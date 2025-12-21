use std::{path::PathBuf, sync::Arc};
use anyhow::Result;
use axum::{
    routing::{get, post},
    extract::{Path, State},
    response::{IntoResponse, sse::{Sse, Event}},
    Json, Router,
};
use futures_util::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex};
use tokio_stream::wrappers::BroadcastStream;
use tower_http::{cors::CorsLayer, services::ServeDir};
use uuid::Uuid;

use crate::{config::{RunCfg, TemplateYaml}, run_once};

#[derive(Clone)]
pub struct AppState {
    config_path: PathBuf,
    template_path: PathBuf,
    current_run: Arc<Mutex<Option<String>>>,
    events_tx: broadcast::Sender<RunEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag="type")]
pub enum RunEvent {
    Started { run_id: String },
    Log { run_id: String, msg: String },
    Progress { run_id: String, done: u64, total: u64 },
    Finished { run_id: String },
    Failed { run_id: String, error: String },
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
        .nest_service("/images", ServeDir::new(".")) // we’ll generate absolute paths in list_images
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

    let _ = tx.send(RunEvent::Started { run_id: run_id.clone() });
    let _ = tx.send(RunEvent::Log { run_id: run_id.clone(), msg: "Run spawned".into() });

    // spawn the actual run
    let spawn_run_id = run_id.clone();
    tokio::spawn(async move {
        // NOTE: run_once currently generates its own internal run_id.
        // For now, we treat this API run_id as the “session id”.
        // If you want them identical, we’ll thread run_id into OrchestratorCfg next.
        let res = run_once(cfg_path, tpl_path, None, false).await;
        match res {
            Ok(_) => { let _ = tx.send(RunEvent::Finished { run_id: spawn_run_id }); }
            Err(e) => { let _ = tx.send(RunEvent::Failed { run_id: spawn_run_id, error: format!("{e:#}") }); }
        }
    });

    Ok(Json(StartRunResp { run_id }))
}

async fn run_events(
    State(st): State<AppState>,
    Path(_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let rx = st.events_tx.subscribe();

    let stream = BroadcastStream::new(rx)
        .filter_map(|msg| async move { msg.ok() })
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
        // We’ll serve the folder statically via /images/<full path> is tricky.
        // Simpler: serve out_dir under /images by changing ServeDir root to out_dir later.
        items.push(ImageItem {
            url: format!("http://127.0.0.1:8787/images/{name}"),
            name,
            created_ms: created,
        });
    }

    // IMPORTANT: to make the above URLs work, run the server with cwd = out_dir,
    // OR better: swap ServeDir::new(".") -> ServeDir::new(out_dir) using a nest_service.
    items.sort_by_key(|i| std::cmp::Reverse(i.created_ms));
    Ok(Json(items))
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
