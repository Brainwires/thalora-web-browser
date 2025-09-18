use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::Instant;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    pub id: String,
    pub url: String,
    pub state: ConnectionState,
    pub last_ping: Instant,
    pub messages_sent: Vec<WebSocketMessage>,
    pub messages_received: Vec<WebSocketMessage>,
    pub protocols: Vec<String>,
    pub selected_protocol: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ConnectionState {
    Connecting,
    Open,
    Closing,
    Closed,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WebSocketMessage {
    #[serde(skip)]
    pub timestamp: Instant,
    pub message_type: MessageType,
    pub data: String,
    pub binary: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum MessageType {
    Text,
    Binary,
    Ping,
    Pong,
    Close,
}

pub type MessageHandler = Box<dyn Fn(&WebSocketMessage) -> Result<Option<WebSocketMessage>> + Send + Sync>;

/// Real WebSocket connection manager for modern web applications
pub struct WebSocketManager {
    pub connections: Arc<Mutex<HashMap<String, WebSocketConnection>>>,
    pub message_handlers: Arc<Mutex<Vec<MessageHandler>>>,
    pub active_senders: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Message>>>>,
}

impl Clone for WebSocketManager {
    fn clone(&self) -> Self {
        Self {
            connections: Arc::clone(&self.connections),
            message_handlers: Arc::clone(&self.message_handlers),
            active_senders: Arc::clone(&self.active_senders),
        }
    }
}

impl std::fmt::Debug for WebSocketManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocketManager")
            .field("connections", &self.connections)
            .field("message_handlers", &"<handlers>")
            .finish()
    }
}