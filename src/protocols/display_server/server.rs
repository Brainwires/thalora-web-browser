/// WebSocket server core for display connections
///
/// Handles WebSocket lifecycle: accept connections, upgrade, message routing.

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::protocols::session_manager::{BrowserCommand, BrowserResponse, SessionManager};
use super::handlers::{processing, CommandHandler};
use super::messages::{current_timestamp, DisplayCommand, DisplayMessage};
use super::sessions::{ClientRegistry, DisplayClient};

/// WebSocket connection handler
pub struct WebSocketServer {
    session_manager: Arc<SessionManager>,
    client_registry: ClientRegistry,
    broadcast_tx: broadcast::Sender<DisplayMessage>,
}

impl WebSocketServer {
    /// Create a new WebSocket server
    pub fn new(session_manager: Arc<SessionManager>, client_registry: ClientRegistry) -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);

        Self {
            session_manager,
            client_registry,
            broadcast_tx,
        }
    }

    /// Start listening for WebSocket connections
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
                                let processed_html = processing::process_html(html, &url);

                                let msg = DisplayMessage::HtmlUpdate {
                                    html: processed_html,
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

        self.client_registry.register(client);

        info!("Display client {} connected with session {}", client_id, session_id);

        // Send connected message
        let connected_msg = DisplayMessage::Connected {
            session_id: session_id.clone(),
            timestamp: current_timestamp(),
        };

        ws_sender.send(Message::Text(serde_json::to_string(&connected_msg)?)).await?;

        // Subscribe to broadcast messages
        let mut broadcast_rx = self.broadcast_tx.subscribe();

        // Create command handler for this client
        let handler = CommandHandler::new(
            Arc::clone(&self.session_manager),
            self.client_registry.clone(),
        );

        // Handle incoming messages and outgoing messages concurrently
        loop {
            tokio::select! {
                // Receive messages from client
                Some(msg) = ws_receiver.next() => {
                    match msg {
                        Ok(Message::Text(text)) => {
                            if let Err(e) = handler.handle_client_message(&client_id, &session_id, &text).await {
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
        self.client_registry.remove(&client_id);

        // Close session if no other clients are using it
        if !self.client_registry.has_clients_for_session(&session_id) {
            let _ = self.session_manager.close_session(&session_id).await;
        }

        info!("Display client {} disconnected", client_id);

        Ok(())
    }

    /// Broadcast a message to all clients
    pub fn broadcast(&self, msg: DisplayMessage) -> Result<()> {
        self.broadcast_tx.send(msg)?;
        Ok(())
    }
}

impl Clone for WebSocketServer {
    fn clone(&self) -> Self {
        Self {
            session_manager: Arc::clone(&self.session_manager),
            client_registry: self.client_registry.clone(),
            broadcast_tx: self.broadcast_tx.clone(),
        }
    }
}
