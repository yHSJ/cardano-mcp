# cardano-mcp

An MCP server for Cardano node interaction via LLMs.

## Requirements

- A running cardano-node with accessible socket

## Build

```
cargo build --release
```

## Run

```
CARDANO_NODE_SOCKET_PATH=/path/to/node.socket \
CARDANO_NETWORK_MAGIC=764824073 \
./target/release/cardano-mcp-server
```

Network magic values: mainnet `764824073`, preprod `1`, preview `2`

## Usage

The server exposes an MCP endpoint at `http://127.0.0.1:3000/mcp`.

### Available Tools

- `get_tip` - Get the current chain tip (slot, block hash, block number)

## Configuration

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `CARDANO_NODE_SOCKET_PATH` | - | Path to cardano-node socket |
| `CARDANO_NETWORK_MAGIC` | `764824073` | Network magic number |
| `CARDANO_MCP_HOST` | `127.0.0.1` | Server bind address |
| `CARDANO_MCP_PORT` | `3000` | Server port |
