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
use serde::Serialize;
use tokio::sync::{broadcast, Mutex};
use tokio_stream::wrappers::BroadcastStream;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use crate::{config::{RunCfg, TemplateYaml}, events::RunEvent, run_once};

#[derive(Clone)]
pub struct AppState {
    config_path: PathBuf,
    template_path: PathBuf,
    current_run: Arc<Mutex<Option<String>>>,
    events_tx: broadcast::Sender<RunEvent>,
}


pub async fn serve(bind: String, config_path: PathBuf, template_path: PathBuf) -> Result<()> {
    let (tx, _rx) = broadcast::channel::<RunEvent>(256);

    let state = AppState {
        config_path,
        template_path,
        current_run: Arc::new(Mutex::new(None)),
        events_tx: tx,
    };

    let app = Router::new()
        .route("/api/config", get(get_config).put(put_config))
        .route("/api/template", get(get_template).put(put_template))
        .route("/api/run", post(start_run))
        .route("/api/run/{id}/events", get(run_events))
        .route("/api/images", get(list_images))
        .route("/images/{name}", get(get_image))
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

#[derive(Serialize)]
struct StartRunResp { run_id: String }

async fn start_run(State(st): State<AppState>) -> Result<Json<StartRunResp>, ApiErr> {
    // create run id
    let run_id = format!("run-{}", Uuid::new_v4());

    // mark current run
    *st.current_run.lock().await = Some(run_id.clone());

    let tx = st.events_tx.clone();
    let cfg_path = st.config_path.clone();
    let tpl_path = st.template_path.clone();

    // spawn the actual run (brief delay lets the frontend SSE subscriber connect)
    let spawn_run_id = run_id.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if let Err(e) = run_once(cfg_path, tpl_path, None, false, Some(spawn_run_id), Some(tx)).await {
            eprintln!("run error: {e:#}");
        }
    });

    Ok(Json(StartRunResp { run_id }))
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
            url: format!("http://127.0.0.1:8787/images/{name}"),
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
struct ApiErr(anyhow::Error);
impl<E: Into<anyhow::Error>> From<E> for ApiErr {
    fn from(e: E) -> Self { ApiErr(e.into()) }
}
impl IntoResponse for ApiErr {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}
