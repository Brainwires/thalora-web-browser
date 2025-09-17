use anyhow::Result;
use tracing::info;

// Core modules organized by functionality
pub mod engine;
pub mod apis;
pub mod features;
pub mod protocols;

use protocols::mcp_server::McpServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Configure tracing to write to stderr only (MCP protocol requirement)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    info!("🧠 Thalora v0.1.0 - Pure Rust headless browser for AI models");
    info!("🔗 Neural connections between AI and the web");

    let mut server = McpServer::new();
    server.run().await
}