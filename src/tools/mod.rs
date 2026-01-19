pub mod get_tip;

use serde_json::Value;
use std::sync::Arc;
use thiserror::Error;

use crate::mcp::protocol::{CallToolResult, Tool};
use crate::node::NodeClient;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Node error: {0}")]
    NodeError(#[from] crate::node::NodeError),
}

pub struct ToolRegistry {
    tools: Vec<Tool>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let tools = vec![get_tip::definition()];

        Self { tools }
    }

    pub fn list_tools(&self) -> Vec<Tool> {
        self.tools.clone()
    }

    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Value,
        node_client: Arc<NodeClient>,
    ) -> Result<CallToolResult, ToolError> {
        match name {
            get_tip::NAME => get_tip::execute(arguments, node_client).await,
            _ => Err(ToolError::NotFound(name.to_string())),
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
