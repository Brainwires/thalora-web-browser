/// Display Server Protocol
///
/// WebSocket/SSE server for streaming browser display state to remote clients.
/// This enables the user's browser to act as a "display" for the headless Thalora browser.
///
/// Architecture:
/// ```
/// User Browser ←→ WebSocket ←→ Display Server ←→ Browser Session
/// ```

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::protocols::session_manager::{SessionManager, BrowserCommand, BrowserResponse};

/// Message types sent from Thalora to display clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DisplayMessage {
    /// Initial connection established
    Connected {
        session_id: String,
        timestamp: u64,
    },

    /// HTML content update
    HtmlUpdate {
        html: String,
        url: String,
        title: Option<String>,
        timestamp: u64,
    },

    /// Navigation event
    Navigation {
        url: String,
        timestamp: u64,
    },

    /// Console log message
    ConsoleLog {
        level: String,
        message: String,
        timestamp: u64,
    },

    /// Network request
    NetworkRequest {
        method: String,
        url: String,
        status: Option<u16>,
        timestamp: u64,
    },

    /// Browser state update
    StateUpdate {
        can_go_back: bool,
        can_go_forward: bool,
        loading: bool,
        timestamp: u64,
    },

    /// Error occurred
    Error {
        message: String,
        timestamp: u64,
    },

    /// Heartbeat/keepalive
    Ping {
        timestamp: u64,
    },
}

/// Message types sent from display clients to Thalora
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DisplayCommand {
    /// Navigate to URL
    Navigate {
        url: String,
    },

    /// Go back in history
    Back,

    /// Go forward in history
    Forward,

    /// Reload current page
    Reload,

    /// Stop loading
    Stop,

    /// Execute JavaScript
    ExecuteScript {
        script: String,
    },

    /// Click element
    Click {
        selector: String,
    },

    /// Type text
    Type {
        selector: String,
        text: String,
    },

    /// Pong response to ping
    Pong {
        timestamp: u64,
    },
}

/// Connected display client
struct DisplayClient {
    id: String,
    session_id: String,
    sender: tokio::sync::mpsc::UnboundedSender<DisplayMessage>,
}

/// Display server state
pub struct DisplayServer {
    clients: Arc<RwLock<HashMap<String, DisplayClient>>>,
    session_manager: Arc<SessionManager>,
    broadcast_tx: broadcast::Sender<DisplayMessage>,
}

impl DisplayServer {
    /// Create a new display server
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);

        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            session_manager,
            broadcast_tx,
        }
    }

    /// Start the WebSocket server
    pub async fn start(&self, bind_addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(bind_addr)
            .await
            .context("Failed to bind display server")?;

        info!("Display server listening on {}", bind_addr);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New display client connection from {}", addr);
                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream).await {
                            error!("Display client error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Handle a WebSocket connection
    async fn handle_connection(&self, stream: TcpStream) -> Result<()> {
        let ws_stream = accept_async(stream)
            .await
            .context("Failed to accept WebSocket")?;

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Generate client ID
        let client_id = Uuid::new_v4().to_string();

        // Create message channel for this client
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        // Wait for authentication message with session_id
        let session_id = match ws_receiver.next().await {
            Some(Ok(Message::Text(text))) => {
                let cmd: DisplayCommand = serde_json::from_str(&text)?;
                match cmd {
                    DisplayCommand::Navigate { url } => {
                        // First message is navigation - create or get session
                        let session_id = format!("display-{}", client_id);

                        // Create new browser session
                        self.session_manager.get_or_create_session(
                            &session_id,
                            true, // persistent
                        ).await?;

                        // Navigate to URL
                        let _response = self.session_manager.send_command(
                            &session_id,
                            BrowserCommand::Navigate {
                                url: url.clone(),
                            },
                        ).await?;

                        // Get content after navigation
                        let content_response = self.session_manager.send_command(
                            &session_id,
                            BrowserCommand::GetContent,
                        ).await?;

                        // Send initial HTML
                        if let BrowserResponse::Success { data } = content_response {
                            if let Some(html) = data.get("content").and_then(|v| v.as_str()) {
                                let msg = DisplayMessage::HtmlUpdate {
                                    html: html.to_string(),
                                    url: url.clone(),
                                    title: None,
                                    timestamp: current_timestamp(),
                                };
                                tx.send(msg)?;
                            }
                        }

                        session_id
                    }
                    _ => {
                        warn!("First message must be Navigate command");
                        return Ok(());
                    }
                }
            }
            Some(Ok(_)) => {
                warn!("Expected text message");
                return Ok(());
            }
            Some(Err(e)) => {
                error!("WebSocket error: {}", e);
                return Ok(());
            }
            None => {
                warn!("Connection closed before authentication");
                return Ok(());
            }
        };

        // Register client
        let client = DisplayClient {
            id: client_id.clone(),
            session_id: session_id.clone(),
            sender: tx,
        };

        self.clients.write().insert(client_id.clone(), client);

        info!("Display client {} connected with session {}", client_id, session_id);

        // Send connected message
        let connected_msg = DisplayMessage::Connected {
            session_id: session_id.clone(),
            timestamp: current_timestamp(),
        };

        ws_sender.send(Message::Text(serde_json::to_string(&connected_msg)?)).await?;

        // Subscribe to broadcast messages
        let mut broadcast_rx = self.broadcast_tx.subscribe();

        // Handle incoming messages and outgoing messages concurrently
        loop {
            tokio::select! {
                // Receive messages from client
                Some(msg) = ws_receiver.next() => {
                    match msg {
                        Ok(Message::Text(text)) => {
                            if let Err(e) = self.handle_client_message(&client_id, &session_id, &text).await {
                                error!("Error handling client message: {}", e);
                            }
                        }
                        Ok(Message::Close(_)) => {
                            info!("Client {} closed connection", client_id);
                            break;
                        }
                        Ok(Message::Ping(data)) => {
                            ws_sender.send(Message::Pong(data)).await?;
                        }
                        Err(e) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }

                // Send messages to client
                Some(msg) = rx.recv() => {
                    let json = serde_json::to_string(&msg)?;
                    if let Err(e) = ws_sender.send(Message::Text(json)).await {
                        error!("Failed to send message to client: {}", e);
                        break;
                    }
                }

                // Broadcast messages to all clients
                Ok(msg) = broadcast_rx.recv() => {
                    let json = serde_json::to_string(&msg)?;
                    if let Err(e) = ws_sender.send(Message::Text(json)).await {
                        error!("Failed to broadcast message to client: {}", e);
                        break;
                    }
                }
            }
        }

        // Cleanup
        self.clients.write().remove(&client_id);

        // Close session if no other clients are using it
        if !self.has_clients_for_session(&session_id) {
            let _ = self.session_manager.close_session(&session_id).await;
        }

        info!("Display client {} disconnected", client_id);

        Ok(())
    }

    /// Handle a message from a display client
    async fn handle_client_message(&self, client_id: &str, session_id: &str, text: &str) -> Result<()> {
        let command: DisplayCommand = serde_json::from_str(text)
            .context("Failed to parse display command")?;

        info!("Client {} sent command: {:?}", client_id, command);

        match command {
            DisplayCommand::Navigate { url } => {
                let _response = self.session_manager.send_command(
                    session_id,
                    BrowserCommand::Navigate {
                        url: url.clone(),
                    },
                ).await?;

                // Get content after navigation
                let content_response = self.session_manager.send_command(
                    session_id,
                    BrowserCommand::GetContent,
                ).await?;

                // Send HTML update
                if let BrowserResponse::Success { data } = content_response {
                    if let Some(html) = data.get("content").and_then(|v| v.as_str()) {
                        self.send_to_client(client_id, DisplayMessage::HtmlUpdate {
                            html: html.to_string(),
                            url: url.clone(),
                            title: None,
                            timestamp: current_timestamp(),
                        }).await?;

                        self.send_to_client(client_id, DisplayMessage::Navigation {
                            url,
                            timestamp: current_timestamp(),
                        }).await?;
                    }
                }
            }

            DisplayCommand::Back => {
                // TODO: Implement NavigateBack command in session_manager
                warn!("Navigate back not yet implemented in session manager");
            }

            DisplayCommand::Forward => {
                // TODO: Implement NavigateForward command in session_manager
                warn!("Navigate forward not yet implemented in session manager");
            }

            DisplayCommand::Reload => {
                // TODO: Implement Refresh command in session_manager
                warn!("Reload not yet implemented in session manager");
            }

            DisplayCommand::Stop => {
                // TODO: Implement stop loading
                warn!("Stop loading not yet implemented");
            }

            DisplayCommand::ExecuteScript { script } => {
                let response = self.session_manager.send_command(
                    session_id,
                    BrowserCommand::ExecuteJs { code: script },
                ).await?;

                // Send result as console log
                if let BrowserResponse::Success { data } = response {
                    self.send_to_client(client_id, DisplayMessage::ConsoleLog {
                        level: "info".to_string(),
                        message: format!("Script result: {:?}", data),
                        timestamp: current_timestamp(),
                    }).await?;
                }
            }

            DisplayCommand::Click { selector } => {
                let _response = self.session_manager.send_command(
                    session_id,
                    BrowserCommand::Click { selector },
                ).await?;

                // Refresh content after click
                self.refresh_client_content(client_id, session_id).await?;
            }

            DisplayCommand::Type { selector, text } => {
                let _response = self.session_manager.send_command(
                    session_id,
                    BrowserCommand::Fill { selector, value: text },
                ).await?;
            }

            DisplayCommand::Pong { .. } => {
                // Keepalive response, no action needed
            }
        }

        Ok(())
    }

    /// Refresh client's displayed content
    async fn refresh_client_content(&self, client_id: &str, session_id: &str) -> Result<()> {
        let response = self.session_manager.send_command(
            session_id,
            BrowserCommand::GetContent,
        ).await?;

        if let BrowserResponse::Success { data } = response {
            if let Some(html) = data.get("content").and_then(|v| v.as_str()) {
                let url = data.get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                self.send_to_client(client_id, DisplayMessage::HtmlUpdate {
                    html: html.to_string(),
                    url,
                    title: None,
                    timestamp: current_timestamp(),
                }).await?;
            }
        }

        Ok(())
    }

    /// Send a message to a specific client
    async fn send_to_client(&self, client_id: &str, msg: DisplayMessage) -> Result<()> {
        let clients = self.clients.read();
        if let Some(client) = clients.get(client_id) {
            client.sender.send(msg)?;
        }
        Ok(())
    }

    /// Check if any clients are using a session
    fn has_clients_for_session(&self, session_id: &str) -> bool {
        self.clients.read().values().any(|c| c.session_id == session_id)
    }

    /// Broadcast a message to all clients
    pub fn broadcast(&self, msg: DisplayMessage) -> Result<()> {
        self.broadcast_tx.send(msg)?;
        Ok(())
    }
}

impl Clone for DisplayServer {
    fn clone(&self) -> Self {
        Self {
            clients: Arc::clone(&self.clients),
            session_manager: Arc::clone(&self.session_manager),
            broadcast_tx: self.broadcast_tx.clone(),
        }
    }
}

/// Get current timestamp in milliseconds
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_message_serialization() {
        let msg = DisplayMessage::HtmlUpdate {
            html: "<h1>Test</h1>".to_string(),
            url: "https://example.com".to_string(),
            title: Some("Test Page".to_string()),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("html_update"));
        assert!(json.contains("Test"));
    }

    #[test]
    fn test_display_command_deserialization() {
        let json = r#"{"type":"navigate","url":"https://example.com"}"#;
        let cmd: DisplayCommand = serde_json::from_str(json).unwrap();

        match cmd {
            DisplayCommand::Navigate { url } => {
                assert_eq!(url, "https://example.com");
            }
            _ => panic!("Expected Navigate command"),
        }
    }
}
