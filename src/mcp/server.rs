use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::mcp::protocol::*;
use crate::node::NodeClient;
use crate::tools::ToolRegistry;

pub struct McpServer {
    server_info: ServerInfo,
    capabilities: ServerCapabilities,
    tool_registry: Arc<ToolRegistry>,
    node_client: Arc<NodeClient>,
    initialized: std::sync::atomic::AtomicBool,
}

impl McpServer {
    pub fn new(tool_registry: Arc<ToolRegistry>, node_client: Arc<NodeClient>) -> Self {
        Self {
            server_info: ServerInfo {
                name: "cardano-mcp-server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                resources: None,
                prompts: None,
            },
            tool_registry,
            node_client,
            initialized: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub async fn handle_request(&self, request: JsonRpcRequest) -> Result<Value, JsonRpcErrorResponse> {
        debug!("Handling request: {}", request.method);

        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id, request.params).await,
            "initialized" => self.handle_initialized(request.id).await,
            "ping" => self.handle_ping(request.id).await,
            "tools/list" => self.handle_tools_list(request.id).await,
            "tools/call" => self.handle_tools_call(request.id, request.params).await,
            "resources/list" => self.handle_resources_list(request.id).await,
            _ => Err(JsonRpcErrorResponse::method_not_found(
                request.id,
                &request.method,
            )),
        }
    }

    async fn handle_initialize(
        &self,
        id: Option<RequestId>,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcErrorResponse> {
        let params: InitializeParams = params
            .map(|p| serde_json::from_value(p))
            .transpose()
            .map_err(|e| JsonRpcErrorResponse::invalid_params(id.clone(), e.to_string()))?
            .ok_or_else(|| {
                JsonRpcErrorResponse::invalid_params(id.clone(), "Missing initialize params")
            })?;

        info!(
            "Client connecting: {} v{}",
            params.client_info.name, params.client_info.version
        );

        let result = InitializeResult {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: self.capabilities.clone(),
            server_info: self.server_info.clone(),
        };

        Ok(serde_json::to_value(result).unwrap())
    }

    async fn handle_initialized(&self, _id: Option<RequestId>) -> Result<Value, JsonRpcErrorResponse> {
        self.initialized
            .store(true, std::sync::atomic::Ordering::SeqCst);
        info!("MCP session initialized");
        Ok(Value::Object(serde_json::Map::new()))
    }

    async fn handle_ping(&self, _id: Option<RequestId>) -> Result<Value, JsonRpcErrorResponse> {
        Ok(Value::Object(serde_json::Map::new()))
    }

    async fn handle_tools_list(&self, _id: Option<RequestId>) -> Result<Value, JsonRpcErrorResponse> {
        let tools = self.tool_registry.list_tools();
        let result = ListToolsResult {
            tools,
            next_cursor: None,
        };
        Ok(serde_json::to_value(result).unwrap())
    }

    async fn handle_tools_call(
        &self,
        id: Option<RequestId>,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcErrorResponse> {
        let params: CallToolParams = params
            .map(|p| serde_json::from_value(p))
            .transpose()
            .map_err(|e| JsonRpcErrorResponse::invalid_params(id.clone(), e.to_string()))?
            .ok_or_else(|| {
                JsonRpcErrorResponse::invalid_params(id.clone(), "Missing tool call params")
            })?;

        debug!("Calling tool: {}", params.name);

        let result = self
            .tool_registry
            .call_tool(&params.name, params.arguments, self.node_client.clone())
            .await;

        match result {
            Ok(tool_result) => Ok(serde_json::to_value(tool_result).unwrap()),
            Err(e) => {
                error!("Tool error: {}", e);
                Ok(serde_json::to_value(CallToolResult::error(e.to_string())).unwrap())
            }
        }
    }

    async fn handle_resources_list(
        &self,
        _id: Option<RequestId>,
    ) -> Result<Value, JsonRpcErrorResponse> {
        let result = ListResourcesResult {
            resources: vec![],
            next_cursor: None,
        };
        Ok(serde_json::to_value(result).unwrap())
    }
}
