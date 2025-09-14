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
    tracing_subscriber::fmt::init();
    
    info!("🧠 Synaptic v0.1.0 - Pure Rust headless browser for AI models");
    info!("🔗 Neural connections between AI and the web");
    
    let mut server = McpServer::new();
    server.run().await
}