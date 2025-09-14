use anyhow::Result;
use tracing::info;

mod mcp;
mod browser;
mod renderer;
mod react_processor;
mod websocket;
mod enhanced_dom;
mod ai_memory;
mod cdp;
mod mcp_server;
mod memory_tools;
mod cdp_tools;

use mcp_server::McpServer;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("🧠 Synaptic v0.1.0 - Pure Rust headless browser for AI models");
    info!("🔗 Neural connections between AI and the web");
    
    let mut server = McpServer::new();
    server.run().await
}