use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "cardano-mcp-server")]
#[command(about = "MCP server for Cardano node and DB-Sync interaction")]
pub struct CliArgs {
    #[arg(short, long, env = "CARDANO_MCP_CONFIG")]
    pub config: Option<PathBuf>,

    #[arg(long, env = "CARDANO_MCP_HOST", default_value = "127.0.0.1")]
    pub host: String,

    #[arg(short, long, env = "CARDANO_MCP_PORT", default_value = "3000")]
    pub port: u16,

    #[arg(long, env = "CARDANO_NODE_SOCKET_PATH")]
    pub node_socket: Option<PathBuf>,

    #[arg(long, env = "CARDANO_NETWORK_MAGIC", default_value = "764824073")]
    pub network_magic: u64,

    #[arg(long, env = "RUST_LOG", default_value = "info")]
    pub log_level: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct FileConfig {
    pub server: Option<ServerConfig>,
    pub node: Option<NodeConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ServerConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Deserialize, Default)]
pub struct NodeConfig {
    pub socket_path: Option<PathBuf>,
    pub network_magic: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub node_socket: Option<PathBuf>,
    pub network_magic: u64,
    pub log_level: String,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();

        let cli = CliArgs::parse();

        let file_config = if let Some(config_path) = &cli.config {
            let contents = std::fs::read_to_string(config_path)?;
            toml::from_str(&contents).unwrap_or_default()
        } else {
            FileConfig::default()
        };

        let server_config = file_config.server.unwrap_or_default();
        let node_config = file_config.node.unwrap_or_default();

        Ok(Config {
            host: if cli.host != "127.0.0.1" {
                cli.host
            } else {
                server_config.host.unwrap_or_else(|| "127.0.0.1".to_string())
            },
            port: if cli.port != 3000 {
                cli.port
            } else {
                server_config.port.unwrap_or(3000)
            },
            node_socket: cli.node_socket.or(node_config.socket_path),
            network_magic: if cli.network_magic != 764824073 {
                cli.network_magic
            } else {
                node_config.network_magic.unwrap_or(764824073)
            },
            log_level: cli.log_level,
        })
    }
}
