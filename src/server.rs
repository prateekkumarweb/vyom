use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use serde::Deserialize;

use crate::FileStorage;

#[derive(Clone)]
pub struct AppState {
    storage: Arc<FileStorage>,
}

#[derive(Deserialize)]
struct PutFileRequest {
    content: String,
    mime_type: Option<String>,
}

impl AppState {
    #[must_use]
    pub fn new(storage: FileStorage) -> Self {
        Self {
            storage: Arc::new(storage),
        }
    }
}

async fn health_check() -> &'static str {
    "vyom storage server is running"
}

async fn list_files(State(state): State<AppState>) -> Response {
    state.storage.all_files().map_or_else(
        |_| StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        |files| {
            let file_list = files.join("\n");
            (StatusCode::OK, file_list).into_response()
        },
    )
}

async fn get_file(Path(filename): Path<String>, State(state): State<AppState>) -> Response {
    match state.storage.get_file(&filename).await {
        Ok(Some((data, metadata))) => {
            let content_type = metadata.mime_type().unwrap_or("application/octet-stream");
            (StatusCode::OK, [(header::CONTENT_TYPE, content_type)], data).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn put_file(
    Path(filename): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<PutFileRequest>,
) -> Response {
    let content_bytes = request.content.into_bytes();
    let cursor = std::io::Cursor::new(content_bytes);

    match state
        .storage
        .put_file(&filename, cursor, request.mime_type)
        .await
    {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn delete_file(Path(filename): Path<String>, State(state): State<AppState>) -> Response {
    match state.storage.del_file(&filename) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/files", get(list_files))
        .route("/files/{filename}", get(get_file))
        .route("/files/{filename}", post(put_file))
        .route("/files/{filename}", delete(delete_file))
        .with_state(state)
}

pub async fn start_server(
    root_dir: &str,
    chunk_size: usize,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing storage at: {root_dir}");
    let storage = FileStorage::new(root_dir, chunk_size).await?;
    let state = AppState::new(storage);
    let app = create_router(state);

    let bind_addr = format!("0.0.0.0:{port}");
    println!("Starting vyom server on {bind_addr}");
    println!("API endpoints:");
    println!("  GET / - Health check");
    println!("  GET /health - Health check");
    println!("  GET /files - List all files (newline separated)");
    println!("  GET /files/{{filename}} - Get file content (raw bytes)");
    println!(
        "  POST /files/{{filename}} - Store file (JSON: {{\"content\": \"...\", \"mime_type\": \"...\"}})"
    );
    println!("  DELETE /files/{{filename}} - Delete file");

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
