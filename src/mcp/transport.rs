use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use futures::stream::Stream;
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, info};

use crate::mcp::protocol::{JsonRpcRequest, JsonRpcResponse};
use crate::mcp::server::McpServer;

pub struct AppState {
    pub mcp_server: Arc<McpServer>,
    pub sse_tx: broadcast::Sender<String>,
}

impl AppState {
    pub fn new(mcp_server: Arc<McpServer>) -> Self {
        let (sse_tx, _) = broadcast::channel(100);
        Self { mcp_server, sse_tx }
    }
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/mcp", post(handle_post).get(handle_sse))
        .route("/health", get(handle_health))
        .layer(cors)
        .with_state(state)
}

async fn handle_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "server": "cardano-mcp-server"
    }))
}

async fn handle_post(
    State(state): State<Arc<AppState>>,
    Json(request): Json<JsonRpcRequest>,
) -> Response {
    debug!("Received request: {:?}", request);

    let id = request.id.clone();

    match state.mcp_server.handle_request(request).await {
        Ok(result) => {
            let response = JsonRpcResponse::new(id, result);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(error_response) => (StatusCode::OK, Json(error_response)).into_response(),
    }
}

async fn handle_sse(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("New SSE connection established");

    let rx = state.sse_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(data) => Some(Ok(Event::default().data(data))),
        Err(_) => None,
    });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("ping"),
    )
}
