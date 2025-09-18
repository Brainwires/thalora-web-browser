use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use url::Url;
use super::types::*;

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            message_handlers: Arc::new(Mutex::new(Vec::new())),
            active_senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a real WebSocket connection
    pub async fn connect(&self, url: &str, protocols: Option<Vec<String>>) -> Result<String> {
        let connection_id = format!("ws_{}", uuid::Uuid::new_v4().simple());
        let _parsed_url = Url::parse(url).map_err(|e| anyhow!("Invalid WebSocket URL: {}", e))?;

        let mut connection = WebSocketConnection {
            id: connection_id.clone(),
            url: url.to_string(),
            state: ConnectionState::Connecting,
            last_ping: Instant::now(),
            messages_sent: Vec::new(),
            messages_received: Vec::new(),
            protocols: protocols.unwrap_or_default(),
            selected_protocol: None,
        };

        // Store connection in connecting state
        {
            let mut connections = self.connections.lock().unwrap();
            connections.insert(connection_id.clone(), connection.clone());
        }

        // Establish real WebSocket connection using the URL directly
        // tokio-tungstenite's connect_async handles all WebSocket headers automatically
        match connect_async(url).await {
            Ok((ws_stream, response)) => {
                // Update connection state to open
                connection.state = ConnectionState::Open;

                // Check for selected protocol in response
                if let Some(protocol_header) = response.headers().get("sec-websocket-protocol") {
                    if let Ok(protocol_str) = protocol_header.to_str() {
                        connection.selected_protocol = Some(protocol_str.to_string());
                    }
                }

                {
                    let mut connections = self.connections.lock().unwrap();
                    connections.insert(connection_id.clone(), connection);
                }

                // Split the WebSocket stream for concurrent read/write
                let (mut ws_sender, mut ws_receiver) = ws_stream.split();
                let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

                // Store sender for this connection
                {
                    let mut senders = self.active_senders.lock().unwrap();
                    senders.insert(connection_id.clone(), tx);
                }

                // Spawn task to handle outgoing messages
                let connection_id_send = connection_id.clone();
                let senders_clone = Arc::clone(&self.active_senders);
                tokio::spawn(async move {
                    while let Some(message) = rx.recv().await {
                        if let Err(e) = ws_sender.send(message).await {
                            tracing::error!("Failed to send WebSocket message: {}", e);
                            // Remove sender on error
                            senders_clone.lock().unwrap().remove(&connection_id_send);
                            break;
                        }
                    }
                });

                self.spawn_message_receiver(connection_id.clone(), ws_receiver).await;

                tracing::info!("WebSocket connected: {} -> {}", connection_id, url);
                Ok(connection_id)
            }
            Err(e) => {
                // Update connection state to closed on error
                {
                    let mut connections = self.connections.lock().unwrap();
                    if let Some(conn) = connections.get_mut(&connection_id) {
                        conn.state = ConnectionState::Closed;
                    }
                }
                Err(anyhow!("Failed to connect WebSocket: {}", e))
            }
        }
    }

    /// Spawn task to handle incoming messages
    async fn spawn_message_receiver<S>(&self, connection_id: String, mut ws_receiver: S)
    where
        S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin + Send + 'static,
    {
        let connections_clone = Arc::clone(&self.connections);
        let handlers_clone = Arc::clone(&self.message_handlers);

        tokio::spawn(async move {
            while let Some(message_result) = ws_receiver.next().await {
                match message_result {
                    Ok(message) => {
                        let ws_message = match message {
                            Message::Text(text) => WebSocketMessage {
                                timestamp: Instant::now(),
                                message_type: MessageType::Text,
                                data: text,
                                binary: false,
                            },
                            Message::Binary(bytes) => WebSocketMessage {
                                timestamp: Instant::now(),
                                message_type: MessageType::Binary,
                                data: String::from_utf8_lossy(&bytes).to_string(),
                                binary: true,
                            },
                            Message::Ping(data) => WebSocketMessage {
                                timestamp: Instant::now(),
                                message_type: MessageType::Ping,
                                data: String::from_utf8_lossy(&data).to_string(),
                                binary: false,
                            },
                            Message::Pong(data) => WebSocketMessage {
                                timestamp: Instant::now(),
                                message_type: MessageType::Pong,
                                data: String::from_utf8_lossy(&data).to_string(),
                                binary: false,
                            },
                            Message::Close(_) => {
                                // Update connection state to closed
                                if let Ok(mut connections) = connections_clone.lock() {
                                    if let Some(conn) = connections.get_mut(&connection_id) {
                                        conn.state = ConnectionState::Closed;
                                    }
                                }
                                WebSocketMessage {
                                    timestamp: Instant::now(),
                                    message_type: MessageType::Close,
                                    data: String::new(),
                                    binary: false,
                                }
                            },
                            Message::Frame(_) => continue, // Skip raw frames
                        };

                        // Store the received message
                        if let Ok(mut connections) = connections_clone.lock() {
                            if let Some(conn) = connections.get_mut(&connection_id) {
                                conn.messages_received.push(ws_message.clone());
                                conn.last_ping = Instant::now();
                            }
                        }

                        // Process message through handlers
                        if let Ok(handlers) = handlers_clone.lock() {
                            for handler in handlers.iter() {
                                if let Ok(response) = handler(&ws_message) {
                                    if response.is_some() {
                                        tracing::debug!("WebSocket message handler returned response");
                                    }
                                }
                            }
                        }

                        tracing::debug!(
                            "WebSocket message received on {}: {:?}",
                            connection_id,
                            ws_message.message_type
                        );
                    }
                    Err(e) => {
                        tracing::error!("WebSocket receive error on {}: {}", connection_id, e);
                        // Update connection state to closed on error
                        if let Ok(mut connections) = connections_clone.lock() {
                            if let Some(conn) = connections.get_mut(&connection_id) {
                                conn.state = ConnectionState::Closed;
                            }
                        }
                        break;
                    }
                }
            }
            tracing::info!("WebSocket receiver task ended for {}", connection_id);
        });
    }

    /// Send a message through an established WebSocket connection
    pub async fn send_message(&self, connection_id: &str, data: &str) -> Result<()> {
        if let Some(sender) = self.active_senders.lock().unwrap().get(connection_id) {
            let message = Message::Text(data.to_string());

            // Record the sent message
            {
                let mut connections = self.connections.lock().unwrap();
                if let Some(conn) = connections.get_mut(connection_id) {
                    conn.messages_sent.push(WebSocketMessage {
                        timestamp: Instant::now(),
                        message_type: MessageType::Text,
                        data: data.to_string(),
                        binary: false,
                    });
                }
            }

            sender.send(message)
                .map_err(|e| anyhow!("Failed to send message: {}", e))?;

            tracing::debug!("Message sent on {}: {}", connection_id, data);
            Ok(())
        } else {
            Err(anyhow!("No active connection found for ID: {}", connection_id))
        }
    }

    /// Send binary data through WebSocket connection
    pub async fn send_binary(&self, connection_id: &str, data: &[u8]) -> Result<()> {
        if let Some(sender) = self.active_senders.lock().unwrap().get(connection_id) {
            let message = Message::Binary(data.to_vec());

            // Record the sent message
            {
                let mut connections = self.connections.lock().unwrap();
                if let Some(conn) = connections.get_mut(connection_id) {
                    conn.messages_sent.push(WebSocketMessage {
                        timestamp: Instant::now(),
                        message_type: MessageType::Binary,
                        data: String::from_utf8_lossy(data).to_string(),
                        binary: true,
                    });
                }
            }

            sender.send(message)
                .map_err(|e| anyhow!("Failed to send binary data: {}", e))?;

            tracing::debug!("Binary data sent on {}: {} bytes", connection_id, data.len());
            Ok(())
        } else {
            Err(anyhow!("No active connection found for ID: {}", connection_id))
        }
    }

    /// Close a WebSocket connection
    pub async fn close_connection(&self, connection_id: &str) -> Result<()> {
        // Remove sender to stop outgoing messages
        self.active_senders.lock().unwrap().remove(connection_id);

        // Update connection state
        {
            let mut connections = self.connections.lock().unwrap();
            if let Some(conn) = connections.get_mut(connection_id) {
                conn.state = ConnectionState::Closed;
            }
        }

        tracing::info!("WebSocket connection closed: {}", connection_id);
        Ok(())
    }

    /// Get connection status
    pub fn get_connection(&self, connection_id: &str) -> Option<WebSocketConnection> {
        self.connections.lock().unwrap().get(connection_id).cloned()
    }

    /// Get all active connections
    pub fn get_all_connections(&self) -> Vec<WebSocketConnection> {
        self.connections.lock().unwrap().values().cloned().collect()
    }

    /// Add a message handler
    pub fn add_message_handler<F>(&self, handler: F)
    where
        F: Fn(&WebSocketMessage) -> Result<Option<WebSocketMessage>> + Send + Sync + 'static,
    {
        let mut handlers = self.message_handlers.lock().unwrap();
        handlers.push(Box::new(handler));
    }

    /// Clear all message handlers
    pub fn clear_message_handlers(&self) {
        self.message_handlers.lock().unwrap().clear();
    }

    /// Get connection statistics
    pub fn get_connection_stats(&self, connection_id: &str) -> Option<ConnectionStats> {
        if let Some(conn) = self.connections.lock().unwrap().get(connection_id) {
            Some(ConnectionStats {
                connection_id: conn.id.clone(),
                url: conn.url.clone(),
                state: conn.state.clone(),
                messages_sent: conn.messages_sent.len(),
                messages_received: conn.messages_received.len(),
                last_activity: conn.last_ping,
                protocols: conn.protocols.clone(),
                selected_protocol: conn.selected_protocol.clone(),
            })
        } else {
            None
        }
    }

    /// Send ping to keep connection alive
    pub async fn ping(&self, connection_id: &str, data: Option<&str>) -> Result<()> {
        if let Some(sender) = self.active_senders.lock().unwrap().get(connection_id) {
            let ping_data = data.unwrap_or("ping").as_bytes().to_vec();
            let message = Message::Ping(ping_data);

            sender.send(message)
                .map_err(|e| anyhow!("Failed to send ping: {}", e))?;

            // Update last ping time
            {
                let mut connections = self.connections.lock().unwrap();
                if let Some(conn) = connections.get_mut(connection_id) {
                    conn.last_ping = Instant::now();
                }
            }

            tracing::debug!("Ping sent on {}", connection_id);
            Ok(())
        } else {
            Err(anyhow!("No active connection found for ID: {}", connection_id))
        }
    }

    /// Clean up closed connections
    pub fn cleanup_closed_connections(&self) -> usize {
        let mut connections = self.connections.lock().unwrap();
        let mut senders = self.active_senders.lock().unwrap();

        let closed_connections: Vec<String> = connections
            .iter()
            .filter(|(_, conn)| matches!(conn.state, ConnectionState::Closed))
            .map(|(id, _)| id.clone())
            .collect();

        let cleanup_count = closed_connections.len();

        for conn_id in closed_connections {
            connections.remove(&conn_id);
            senders.remove(&conn_id);
        }

        if cleanup_count > 0 {
            tracing::info!("Cleaned up {} closed WebSocket connections", cleanup_count);
        }

        cleanup_count
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub connection_id: String,
    pub url: String,
    pub state: ConnectionState,
    pub messages_sent: usize,
    pub messages_received: usize,
    pub last_activity: Instant,
    pub protocols: Vec<String>,
    pub selected_protocol: Option<String>,
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}