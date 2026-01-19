mod config;
mod mcp;
mod node;
mod tools;

use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::mcp::{create_router, AppState, McpServer};
use crate::node::NodeClient;
use crate::tools::ToolRegistry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.log_level.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting cardano-mcp-server v{}", env!("CARGO_PKG_VERSION"));

    let node_client = Arc::new(NodeClient::new(
        config.node_socket.clone(),
        config.network_magic,
    ));

    if !node_client.is_configured() {
        warn!(
            "No node socket configured. Set CARDANO_NODE_SOCKET_PATH to enable node queries."
        );
    }

    let tool_registry = Arc::new(ToolRegistry::new());
    info!("Registered {} tools", tool_registry.list_tools().len());

    let mcp_server = Arc::new(McpServer::new(tool_registry, node_client));

    let app_state = Arc::new(AppState::new(mcp_server));
    let router = create_router(app_state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    info!("Listening on http://{}", addr);
    info!("MCP endpoint: POST http://{}/mcp", addr);
    info!("SSE endpoint: GET http://{}/mcp", addr);
    info!("Health check: GET http://{}/health", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
