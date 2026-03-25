/// Display Server Protocol
///
/// WebSocket/SSE server for streaming browser display state to remote clients.
/// This enables the user's browser to act as a "display" for the headless Thalora browser.
///
/// Architecture:
/// ```
/// User Browser ←→ WebSocket ←→ Display Server ←→ Browser Session
/// ```
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::protocols::session_manager::SessionManager;

// Module declarations
mod handlers;
mod messages;
mod server;
mod sessions;

// Public exports
pub use messages::{DisplayCommand, DisplayMessage, ScreencastFrameMetadata, current_timestamp};
pub use server::WebSocketServer;
pub use sessions::ClientRegistry;

/// Display server main struct
pub struct DisplayServer {
    ws_server: WebSocketServer,
    client_registry: ClientRegistry,
}

impl DisplayServer {
    /// Create a new display server
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        let client_registry = ClientRegistry::new();
        let ws_server = WebSocketServer::new(session_manager, client_registry.clone());

        Self {
            ws_server,
            client_registry,
        }
    }

    /// Start the WebSocket server
    pub async fn start(&self, bind_addr: SocketAddr) -> Result<()> {
        self.ws_server.start(bind_addr).await
    }

    /// Broadcast a message to all clients
    pub fn broadcast(&self, msg: DisplayMessage) -> Result<()> {
        self.ws_server.broadcast(msg)
    }

    /// Send a message to a specific client
    pub fn send_to_client(&self, client_id: &str, msg: DisplayMessage) -> Result<()> {
        self.client_registry.send_to_client(client_id, msg)
    }

    /// Check if any clients are using a session
    pub fn has_clients_for_session(&self, session_id: &str) -> bool {
        self.client_registry.has_clients_for_session(session_id)
    }

    /// Get the number of connected clients
    pub fn client_count(&self) -> usize {
        self.client_registry.client_count()
    }
}

impl Clone for DisplayServer {
    fn clone(&self) -> Self {
        Self {
            ws_server: self.ws_server.clone(),
            client_registry: self.client_registry.clone(),
        }
    }
}
