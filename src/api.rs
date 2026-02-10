use std::{path::{Component, PathBuf}, sync::Arc};
use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::{sse::{Event, Sse}, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex};
use tokio_stream::wrappers::BroadcastStream;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use crate::{auth::{self, UserResponse}, config::{Mode, RunCfg, TemplateYaml}, cost_tracking, events::RunEvent, run_once};
use anyhow::Context;

#[derive(Clone)]
pub struct AppState {
    config_path: PathBuf,
    template_path: PathBuf,
    current_run: Arc<Mutex<Option<String>>>,
    events_tx: broadcast::Sender<RunEvent>,
    pool: sqlx::PgPool,
}


pub async fn serve(bind: String, config_path: PathBuf, template_path: PathBuf, pool: sqlx::PgPool) -> Result<()> {
    // Validate config and output directory at startup
    let cfg_txt = tokio::fs::read_to_string(&config_path)
        .await
        .context(format!("Failed to read config file: {}", config_path.display()))?;
    let cfg: RunCfg = serde_yaml::from_str(&cfg_txt)
        .context("Failed to parse config YAML")?;

    // Validate output directory
    crate::validate_output_dir(&cfg.out_dir)
        .await
        .context("Output directory validation failed")?;

    println!("✅ Output directory validated: {}", cfg.out_dir.display());

    let (tx, _rx) = broadcast::channel::<RunEvent>(256);

    let state = AppState {
        config_path,
        template_path,
        current_run: Arc::new(Mutex::new(None)),
        events_tx: tx,
        pool,
    };

    let app = Router::new()
        .route("/api/config", get(get_config).put(put_config))
        .route("/api/config/validate", post(validate_config))
        .route("/api/template", get(get_template).put(put_template))
        .route("/api/run", post(start_run))
        .route("/api/run/current", get(get_current_run))
        .route("/api/run/{id}/events", get(run_events))
        .route("/api/images", get(list_images))
        .route("/images/{name}", get(get_image))
        .route("/api/register", post(register))
        .route("/api/login", post(login))
        .route("/api/cost/summary", get(cost_summary))
        .route("/api/cost/estimate", post(cost_estimate))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    println!("✅ adgen API listening on http://{bind}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn get_config(State(st): State<AppState>) -> Result<Json<RunCfg>, ApiErr> {
    let txt = tokio::fs::read_to_string(&st.config_path).await.map_err(ApiErr::from)?;
    let cfg: RunCfg = serde_yaml::from_str(&txt).map_err(ApiErr::from)?;
    Ok(Json(cfg))
}

async fn put_config(State(st): State<AppState>, Json(cfg): Json<RunCfg>) -> Result<impl IntoResponse, ApiErr> {
    let out = serde_yaml::to_string(&cfg).map_err(ApiErr::from)?;
    tokio::fs::write(&st.config_path, out).await.map_err(ApiErr::from)?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn get_template(State(st): State<AppState>) -> Result<Json<TemplateYaml>, ApiErr> {
    let txt = tokio::fs::read_to_string(&st.template_path).await.map_err(ApiErr::from)?;
    let tpl: TemplateYaml = serde_yaml::from_str(&txt).map_err(ApiErr::from)?;
    Ok(Json(tpl))
}

async fn put_template(State(st): State<AppState>, Json(tpl): Json<TemplateYaml>) -> Result<impl IntoResponse, ApiErr> {
    let out = serde_yaml::to_string(&tpl).map_err(ApiErr::from)?;
    tokio::fs::write(&st.template_path, out).await.map_err(ApiErr::from)?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct RegisterReq {
    email: String,
    password: String,
    name: Option<String>,
}

#[derive(Deserialize)]
struct LoginReq {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct ValidateConfigReq {
    config: RunCfg,
    template: TemplateYaml,
}

#[derive(Serialize)]
struct ValidationError {
    field: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggestion: Option<String>,
}

#[derive(Serialize)]
struct ValidationResult {
    valid: bool,
    errors: Vec<ValidationError>,
    warnings: Vec<String>,
}

async fn validate_config(
    State(_st): State<AppState>,
    Json(req): Json<ValidateConfigReq>,
) -> Json<ValidationResult> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Validate concurrency
    if req.config.orchestrator.concurrency == 0 {
        errors.push(ValidationError {
            field: "orchestrator.concurrency".to_string(),
            message: "Concurrency must be greater than 0".to_string(),
            suggestion: Some("Set concurrency to at least 1".to_string()),
        });
    }

    // Validate target_images
    if req.config.orchestrator.target_images == 0 {
        errors.push(ValidationError {
            field: "orchestrator.target_images".to_string(),
            message: "Target images must be greater than 0".to_string(),
            suggestion: None,
        });
    }

    // Validate output directory
    if let Err(e) = crate::validate_output_dir(&req.config.out_dir).await {
        errors.push(ValidationError {
            field: "out_dir".to_string(),
            message: format!("Output directory error: {}", e),
            suggestion: Some("Ensure directory exists and is writable".to_string()),
        });
    }

    // Validate API key for OpenAI provider
    if req.config.provider.kind == "openai" {
        let key_env = req.config.provider.api_key_env.as_deref().unwrap_or("OPENAI_API_KEY");
        if std::env::var(key_env).is_err() {
            errors.push(ValidationError {
                field: "provider.api_key_env".to_string(),
                message: format!("Environment variable {} not set", key_env),
                suggestion: Some(format!("Run: export {}=sk-...", key_env)),
            });
        }
    }

    // Validate template by prompt mode
    match &req.template.mode {
        Mode::AdTemplate(tpl) => {
            if tpl.styles.is_empty() {
                errors.push(ValidationError {
                    field: "mode.AdTemplate.styles".to_string(),
                    message: "At least one style is required".to_string(),
                    suggestion: None,
                });
            }

            if tpl.brand.trim().is_empty() {
                errors.push(ValidationError {
                    field: "mode.AdTemplate.brand".to_string(),
                    message: "Brand cannot be empty".to_string(),
                    suggestion: None,
                });
            }

            if tpl.product.trim().is_empty() {
                errors.push(ValidationError {
                    field: "mode.AdTemplate.product".to_string(),
                    message: "Product cannot be empty".to_string(),
                    suggestion: None,
                });
            }
        }
        Mode::GeneralPrompt(prompt) => {
            if prompt.prompt.trim().is_empty() {
                errors.push(ValidationError {
                    field: "mode.GeneralPrompt.prompt".to_string(),
                    message: "Prompt cannot be empty".to_string(),
                    suggestion: None,
                });
            }
        }
    }

    // Warnings
    if req.config.orchestrator.concurrency as u32 > req.config.orchestrator.rate_per_min {
        warnings.push(format!(
            "Concurrency ({}) exceeds rate limit ({}/min) - may cause burst throttling",
            req.config.orchestrator.concurrency, req.config.orchestrator.rate_per_min
        ));
    }

    if req.config.orchestrator.rate_per_min > 60 {
        warnings.push("High rate limit may cause API throttling".to_string());
    }

    Json(ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    })
}

#[derive(Serialize)]
struct StartRunResp { run_id: String }

async fn start_run(State(st): State<AppState>) -> Result<Json<StartRunResp>, ApiErr> {
    // Check if a run is already in progress
    {
        let current = st.current_run.lock().await;
        if let Some(existing_id) = &*current {
            return Err(ApiErr::run_already_active(existing_id));
        }
    }

    // create run id
    let run_id = format!("run-{}", Uuid::new_v4());

    // mark current run
    *st.current_run.lock().await = Some(run_id.clone());

    let tx = st.events_tx.clone();
    let cfg_path = st.config_path.clone();
    let tpl_path = st.template_path.clone();
    let current_run_ref = st.current_run.clone();

    // spawn the actual run (brief delay lets the frontend SSE subscriber connect)
    let spawn_run_id = run_id.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let result = run_once(cfg_path, tpl_path, None, false, Some(spawn_run_id), Some(tx)).await;

        // Clear current run on completion or failure
        *current_run_ref.lock().await = None;

        if let Err(e) = result {
            eprintln!("run error: {e:#}");
        }
    });

    Ok(Json(StartRunResp { run_id }))
}

#[derive(Serialize)]
struct CurrentRunResp { run_id: Option<String> }

async fn get_current_run(State(st): State<AppState>) -> Json<CurrentRunResp> {
    let current = st.current_run.lock().await;
    Json(CurrentRunResp { run_id: current.clone() })
}

pub async fn run_events(
    State(st): State<AppState>,
    Path(run_id): Path<String>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let rx = st.events_tx.subscribe();

    let stream = BroadcastStream::new(rx)
        .filter_map(|msg| async move { msg.ok() })
        .filter(move |evt: &RunEvent| {
            // keep only events for this run_id
            let ok = match evt {
                RunEvent::Started { run_id: id, .. } => id == &run_id,
                RunEvent::Log { run_id: id, .. } => id == &run_id,
                RunEvent::Progress { run_id: id, .. } => id == &run_id,
                RunEvent::Finished { run_id: id } => id == &run_id,
                RunEvent::Failed { run_id: id, .. } => id == &run_id,
            };
            futures_util::future::ready(ok)
        })
        .map(|evt| {
            let json = serde_json::to_string(&evt).unwrap();
            Ok(Event::default().event("message").data(json))
        });

    Sse::new(stream)
}
async fn register(
    State(st): State<AppState>,
    Json(req): Json<RegisterReq>,
) -> Result<(StatusCode, Json<UserResponse>), ApiErr> {
    let email = req.email.trim().to_lowercase();
    if email.is_empty() {
        return Err(ApiErr::bad_request("Email is required"));
    }
    if req.password.len() < 8 {
        return Err(ApiErr::bad_request("Password must be at least 8 characters"));
    }

    let password = req.password;
    let hashed = tokio::task::spawn_blocking(move || auth::hash_password(&password))
        .await
        .map_err(|e| ApiErr::internal(e))?
        .map_err(|e| ApiErr::internal(e))?;

    let row = sqlx::query_as::<_, auth::UserRow>(
        "INSERT INTO users (email, password, name) VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(&email)
    .bind(&hashed)
    .bind(&req.name)
    .fetch_one(&st.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.code().as_deref() == Some("23505") {
                return ApiErr::conflict("A user with this email already exists");
            }
        }
        ApiErr::internal(e)
    })?;

    Ok((StatusCode::CREATED, Json(UserResponse::from(row))))
}

async fn login(
    State(st): State<AppState>,
    Json(req): Json<LoginReq>,
) -> Result<Json<UserResponse>, ApiErr> {
    let email = req.email.trim().to_lowercase();

    let row = sqlx::query_as::<_, auth::UserRow>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&email)
    .fetch_optional(&st.pool)
    .await
    .map_err(ApiErr::internal)?;

    let user = match row {
        Some(u) => u,
        None => return Err(ApiErr::unauthorized()),
    };

    let stored_hash = user.password.clone();
    let password = req.password;
    let valid = tokio::task::spawn_blocking(move || auth::verify_password(&password, &stored_hash))
        .await
        .map_err(|e| ApiErr::internal(e))?
        .map_err(|e| ApiErr::internal(e))?;

    if !valid {
        return Err(ApiErr::unauthorized());
    }

    Ok(Json(UserResponse::from(user)))
}

async fn cost_summary(State(st): State<AppState>) -> Result<Json<cost_tracking::CostSummary>, ApiErr> {
    let txt = tokio::fs::read_to_string(&st.config_path).await.map_err(ApiErr::from)?;
    let cfg: RunCfg = serde_yaml::from_str(&txt).map_err(ApiErr::from)?;
    let summary = cost_tracking::compute_cost_summary(&cfg.out_dir)
        .await
        .map_err(ApiErr::from)?;
    Ok(Json(summary))
}

#[derive(Deserialize)]
struct CostEstimateReq {
    target_images: u64,
    price_per_image: f64,
}

#[derive(Serialize)]
struct CostEstimateResp {
    estimated_cost: f64,
}

async fn cost_estimate(Json(req): Json<CostEstimateReq>) -> Json<CostEstimateResp> {
    Json(CostEstimateResp {
        estimated_cost: cost_tracking::estimate_cost(req.target_images, req.price_per_image),
    })
}

#[derive(Serialize)]
struct ImageItem { name: String, url: String, created_ms: u128 }

async fn list_images(State(st): State<AppState>) -> Result<Json<Vec<ImageItem>>, ApiErr> {
    // read config to know out_dir
    let txt = tokio::fs::read_to_string(&st.config_path).await.map_err(ApiErr::from)?;
    let cfg: RunCfg = serde_yaml::from_str(&txt).map_err(ApiErr::from)?;
    let out_dir = cfg.out_dir;

    let mut items = vec![];
    let mut rd = tokio::fs::read_dir(&out_dir).await.map_err(ApiErr::from)?;
    while let Some(ent) = rd.next_entry().await.map_err(ApiErr::from)? {
        let path = ent.path();
        if path.extension().and_then(|s| s.to_str()) != Some("png") { continue; }
        let meta = ent.metadata().await.map_err(ApiErr::from)?;
        let created = meta.modified().ok()
            .and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_millis())
            .unwrap_or(0);

        let name = path.file_name().unwrap().to_string_lossy().to_string();
        items.push(ImageItem {
            url: format!("/images/{name}"),
            name,
            created_ms: created,
        });
    }

    items.sort_by_key(|i| std::cmp::Reverse(i.created_ms));
    Ok(Json(items))
}

async fn get_image(
    State(st): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if !is_safe_filename(&name) {
        return (StatusCode::BAD_REQUEST, "invalid filename").into_response();
    }

    let cfg_txt = match tokio::fs::read_to_string(&st.config_path).await {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("read config failed: {e}")).into_response(),
    };

    let cfg: RunCfg = match serde_yaml::from_str(&cfg_txt) {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("parse config failed: {e}")).into_response(),
    };

    let path: PathBuf = cfg.out_dir.join(&name);

    let bytes = match tokio::fs::read(&path).await {
        Ok(b) => b,
        Err(_) => return (StatusCode::NOT_FOUND, "not found").into_response(),
    };

    let content_type = content_type_for(&name);

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, HeaderValue::from_static(content_type))],
        bytes,
    )
        .into_response()
}

fn is_safe_filename(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let p = std::path::Path::new(name);
    let mut comps = p.components();
    matches!((comps.next(), comps.next()), (Some(Component::Normal(_)), None))
}

fn content_type_for(name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else {
        "application/octet-stream"
    }
}

#[derive(Debug)]
struct ApiErr {
    status: StatusCode,
    code: String,
    message: String,
    suggestion: Option<String>,
}

impl ApiErr {
    fn internal(e: impl std::fmt::Display) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error".to_string(),
            message: format!("Internal error: {}", e),
            suggestion: None,
        }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "bad_request".to_string(),
            message: message.into(),
            suggestion: None,
        }
    }

    fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: "conflict".to_string(),
            message: message.into(),
            suggestion: None,
        }
    }

    fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "unauthorized".to_string(),
            message: "Invalid email or password".to_string(),
            suggestion: None,
        }
    }

    fn run_already_active(run_id: &str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: "run_already_active".to_string(),
            message: format!("A run is already in progress: {}", run_id),
            suggestion: Some("Wait for the current run to complete, or navigate to Run Monitor to view progress.".to_string()),
        }
    }
}

impl<E: Into<anyhow::Error>> From<E> for ApiErr {
    fn from(e: E) -> Self {
        Self::internal(e.into())
    }
}

impl IntoResponse for ApiErr {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            error: String,
            code: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            suggestion: Option<String>,
        }
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
                code: self.code,
                suggestion: self.suggestion,
            }),
        )
            .into_response()
    }
}
