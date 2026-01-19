use serde_json::Value;
use std::sync::Arc;

use crate::mcp::protocol::{CallToolResult, Tool, ToolInputSchema};
use crate::node::NodeClient;
use crate::tools::ToolError;

pub const NAME: &str = "get_network";

pub fn definition() -> Tool {
    Tool {
        name: NAME.to_string(),
        description: Some(
            "Get the Cardano network the connected node is running on. Returns the network name \
             (mainnet, preprod, preview, or other) and network magic number."
                .to_string(),
        ),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: Some(serde_json::json!({})),
            required: None,
        },
    }
}

pub async fn execute(
    _args: Value,
    node_client: Arc<NodeClient>,
) -> Result<CallToolResult, ToolError> {
    let response = serde_json::json!({
        "network": node_client.network_name(),
        "magic": node_client.network_magic(),
    });

    Ok(CallToolResult::text(
        serde_json::to_string_pretty(&response).unwrap(),
    ))
}
