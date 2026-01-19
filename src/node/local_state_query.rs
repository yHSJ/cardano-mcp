use pallas_network::facades::NodeClient as PallasNodeClient;
use pallas_network::miniprotocols::localstate::queries_v16;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tracing::{debug, info, warn};

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("Node socket not configured")]
    SocketNotConfigured,

    #[error("Failed to connect to node: {0}")]
    ConnectionFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainTip {
    pub slot: u64,
    pub hash: String,
    pub block_number: Option<u64>,
}

pub struct NodeClient {
    socket_path: Option<PathBuf>,
    network_magic: u64,
}

impl NodeClient {
    pub fn new(socket_path: Option<PathBuf>, network_magic: u64) -> Self {
        if let Some(ref path) = socket_path {
            info!("NodeClient configured with socket: {:?}", path);
        } else {
            warn!("NodeClient created without socket path - queries will fail");
        }

        Self {
            socket_path,
            network_magic,
        }
    }

    async fn connect(&self) -> Result<PallasNodeClient, NodeError> {
        let socket_path = self
            .socket_path
            .as_ref()
            .ok_or(NodeError::SocketNotConfigured)?;

        let socket_str = socket_path
            .to_str()
            .ok_or_else(|| NodeError::ConnectionFailed("Invalid socket path".to_string()))?;

        debug!(
            "Connecting to node at {} with network magic {}",
            socket_str, self.network_magic
        );

        PallasNodeClient::connect(socket_str, self.network_magic)
            .await
            .map_err(|e| {
                tracing::error!("Connection failed: {:?}", e);
                NodeError::ConnectionFailed(e.to_string())
            })
    }

    pub async fn get_tip(&self) -> Result<ChainTip, NodeError> {
        debug!("Querying chain tip from node");

        let mut client = self.connect().await?;

        let statequery = client.statequery();
        statequery
            .acquire(None)
            .await
            .map_err(|e| NodeError::ProtocolError(format!("Failed to acquire state: {}", e)))?;

        let chain_point = queries_v16::get_chain_point(statequery)
            .await
            .map_err(|e| NodeError::QueryFailed(format!("Failed to get chain point: {}", e)))?;

        let chain_block_no = queries_v16::get_chain_block_no(statequery)
            .await
            .map_err(|e| {
                NodeError::QueryFailed(format!("Failed to get chain block number: {}", e))
            })?;

        let block_number = chain_block_no.block_number;

        let (slot, hash) = match chain_point {
            pallas_network::miniprotocols::Point::Origin => (0, "origin".to_string()),
            pallas_network::miniprotocols::Point::Specific(slot, hash) => {
                (slot, hex::encode(&hash))
            }
        };

        Ok(ChainTip {
            slot,
            hash,
            block_number: Some(block_number.into()),
        })
    }

    pub fn is_configured(&self) -> bool {
        self.socket_path.is_some()
    }

    pub fn network_magic(&self) -> u64 {
        self.network_magic
    }

    pub fn network_name(&self) -> &'static str {
        match self.network_magic {
            764824073 => "mainnet",
            1 => "preprod",
            2 => "preview",
            _ => "other",
        }
    }
}
