use serde_json::Value;
use std::sync::Arc;

use crate::mcp::protocol::{CallToolResult, Tool, ToolInputSchema};
use crate::node::NodeClient;
use crate::tools::ToolError;

pub const NAME: &str = "get_tip";

pub fn definition() -> Tool {
    Tool {
        name: NAME.to_string(),
        description: Some(
            "Get the current tip of the Cardano blockchain. Returns the block height (slot number) \
             and block hash of the most recent block known to the connected node."
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
    let tip = node_client.get_tip().await?;

    let response = serde_json::json!({
        "slot": tip.slot,
        "hash": tip.hash,
        "block_number": tip.block_number,
    });

    Ok(CallToolResult::text(
        serde_json::to_string_pretty(&response).unwrap(),
    ))
}
