/// Session management for display server clients
///
/// Tracks connected clients, their browser sessions, and manages lifecycle.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use super::messages::DisplayMessage;

/// Connected display client
pub struct DisplayClient {
    pub(super) id: String,
    pub(super) session_id: String,
    pub(super) sender: UnboundedSender<DisplayMessage>,
}

/// Client session storage and management
pub struct ClientRegistry {
    clients: Arc<RwLock<HashMap<String, DisplayClient>>>,
}

impl ClientRegistry {
    /// Create a new client registry
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new client
    pub(super) fn register(&self, client: DisplayClient) {
        self.clients.write().insert(client.id.clone(), client);
    }

    /// Remove a client by ID
    pub(super) fn remove(&self, client_id: &str) {
        self.clients.write().remove(client_id);
    }

    /// Check if any clients are using a session
    pub fn has_clients_for_session(&self, session_id: &str) -> bool {
        self.clients.read().values().any(|c| c.session_id == session_id)
    }

    /// Get the number of connected clients
    pub fn client_count(&self) -> usize {
        self.clients.read().len()
    }

    /// Send a message to a specific client
    pub fn send_to_client(&self, client_id: &str, msg: DisplayMessage) -> anyhow::Result<()> {
        let clients = self.clients.read();
        if let Some(client) = clients.get(client_id) {
            client.sender.send(msg)?;
        }
        Ok(())
    }
}

impl Clone for ClientRegistry {
    fn clone(&self) -> Self {
        Self {
            clients: Arc::clone(&self.clients),
        }
    }
}

impl Default for ClientRegistry {
    fn default() -> Self {
        Self::new()
    }
}
