mod ws;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, Router},
};
use std::{path::PathBuf, sync::Arc, time::Instant};
use tower::{ServiceBuilder, limit::ConcurrencyLimitLayer};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tracing::info;
use worlds_shared::WorldGenerator;

#[derive(Clone)]
struct AppState {
    generator: Arc<WorldGenerator>,
    start_time: Instant,
    assets_dir: PathBuf,
    ws_state: ws::WsState,
}

impl AppState {
    fn new(seed: u32, assets_dir: PathBuf) -> Self {
        Self {
            generator: Arc::new(WorldGenerator::new(seed)),
            start_time: Instant::now(),
            assets_dir,
            ws_state: ws::WsState {
                rooms: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            },
        }
    }
}

async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let uptime = state.start_time.elapsed();
    serde_json::json!({
        "status": "ok",
        "uptime_seconds": uptime.as_secs(),
    }).to_string()
}

async fn get_chunk(
    State(state): State<AppState>,
    Path((x, y, z)): Path<(i32, i32, i32)>,
) -> impl IntoResponse {
    let generator = state.generator.clone();
    
    let chunk = tokio::task::spawn_blocking(move || {
        generator.generate_chunk(x, y, z)
    }).await;
    
    match chunk {
        Ok(c) => serde_json::to_string(&c).unwrap_or_else(|_| "{}".to_string()),
        Err(_) => serde_json::json!({"error": "generation_failed"}).to_string(),
    }
}

async fn serve_index(State(state): State<AppState>) -> impl IntoResponse {
    let path = state.assets_dir.join("index.html");
    match tokio::fs::read_to_string(&path).await {
        Ok(s) => axum::response::Html(s),
        Err(_) => axum::response::Html("Worlds Server".to_string()),
    }
}

async fn serve_asset(State(state): State<AppState>, Path(path): Path<String>) -> impl IntoResponse {
    let clean = path.trim_start_matches('/');
    let file_path = state.assets_dir.join(clean);
    
    if !file_path.exists() || !file_path.is_file() {
        return (axum::http::StatusCode::NOT_FOUND, "Not found").into_response();
    }
    
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let mime = match ext {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css",
        "js" => "application/javascript",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    };
    
    let bytes = match tokio::fs::read(&file_path).await {
        Ok(b) => b,
        Err(_) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Read error").into_response(),
    };
    
    let body = axum::body::Body::from(bytes);
    let mut res = axum::http::Response::new(body);
    res.headers_mut().insert(axum::http::header::CONTENT_TYPE, mime.parse().unwrap());
    res.headers_mut().insert(
        axum::http::HeaderName::from_static("cross-origin-opener-policy"),
        "same-origin".parse().unwrap(),
    );
    res.headers_mut().insert(
        axum::http::HeaderName::from_static("cross-origin-embedder-policy"),
        "credentialless".parse().unwrap(),
    );
    res
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    
    let mut assets_dir = std::env::current_exe()?.parent().map(|p| p.join("assets")).unwrap_or_else(|| PathBuf::from("assets"));
    if !assets_dir.exists() { assets_dir = PathBuf::from("server/assets"); }
    if !assets_dir.exists() { return Err(format!("Assets not found: {:?}", assets_dir).into()); }
    
    info!("Assets: {:?}", assets_dir);
    info!("Iniciando Worlds Server...");
    let state = AppState::new(42, assets_dir);
    
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/health", get(health_check))
        .route("/api/chunk/{x}/{y}/{z}", get(get_chunk))
        .route("/ws", get(ws::ws_handler))
        .route("/{*path}", get(serve_asset))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()).layer(CompressionLayer::new()).layer(ConcurrencyLimitLayer::new(1000)).layer(cors))
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Servidor en http://0.0.0.0:3000");
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
