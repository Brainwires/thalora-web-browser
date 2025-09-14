use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{Duration, Instant};
use serde_json::Value;

/// WebSocket connection simulation for modern web applications
pub struct WebSocketManager {
    connections: Arc<Mutex<HashMap<String, WebSocketConnection>>>,
    message_handlers: Arc<Mutex<Vec<MessageHandler>>>,
}

impl Clone for WebSocketManager {
    fn clone(&self) -> Self {
        Self {
            connections: Arc::clone(&self.connections),
            message_handlers: Arc::clone(&self.message_handlers),
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

#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    pub id: String,
    pub url: String,
    pub state: ConnectionState,
    pub last_ping: Instant,
    pub messages_sent: Vec<WebSocketMessage>,
    pub messages_received: Vec<WebSocketMessage>,
    pub protocols: Vec<String>,
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

type MessageHandler = Box<dyn Fn(&WebSocketMessage) -> Result<Option<WebSocketMessage>> + Send + Sync>;

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            message_handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a new WebSocket connection simulation
    pub async fn connect(&self, url: &str, protocols: Option<Vec<String>>) -> Result<String> {
        let connection_id = format!("ws_{}", uuid::Uuid::new_v4().simple());
        
        let connection = WebSocketConnection {
            id: connection_id.clone(),
            url: url.to_string(),
            state: ConnectionState::Connecting,
            last_ping: Instant::now(),
            messages_sent: Vec::new(),
            messages_received: Vec::new(),
            protocols: protocols.unwrap_or_default(),
        };

        // Simulate connection establishment
        tokio::time::sleep(Duration::from_millis(100)).await;

        let mut connections = self.connections.lock().unwrap();
        connections.insert(connection_id.clone(), connection);

        // Simulate successful connection
        self.update_connection_state(&connection_id, ConnectionState::Open)?;

        tracing::info!("WebSocket connection established: {} -> {}", connection_id, url);
        Ok(connection_id)
    }

    /// Send a message through the WebSocket
    pub async fn send_message(&self, connection_id: &str, data: &str, binary: bool) -> Result<()> {
        let message = WebSocketMessage {
            timestamp: Instant::now(),
            message_type: if binary { MessageType::Binary } else { MessageType::Text },
            data: data.to_string(),
            binary,
        };

        {
            let mut connections = self.connections.lock().unwrap();
            let connection = connections.get_mut(connection_id)
                .ok_or_else(|| anyhow!("WebSocket connection not found: {}", connection_id))?;

            if !matches!(connection.state, ConnectionState::Open) {
                return Err(anyhow!("WebSocket connection is not open: {}", connection_id));
            }

            connection.messages_sent.push(message.clone());
        }

        // Simulate message processing and potential response
        self.process_outgoing_message(connection_id, &message).await?;

        tracing::debug!("WebSocket message sent on {}: {} bytes", connection_id, data.len());
        Ok(())
    }

    /// Simulate receiving a message
    pub async fn simulate_incoming_message(&self, connection_id: &str, data: &str, binary: bool) -> Result<()> {
        let message = WebSocketMessage {
            timestamp: Instant::now(),
            message_type: if binary { MessageType::Binary } else { MessageType::Text },
            data: data.to_string(),
            binary,
        };

        {
            let mut connections = self.connections.lock().unwrap();
            let connection = connections.get_mut(connection_id)
                .ok_or_else(|| anyhow!("WebSocket connection not found: {}", connection_id))?;

            connection.messages_received.push(message.clone());
        }

        self.process_incoming_message(connection_id, &message).await?;
        tracing::debug!("WebSocket message received on {}: {} bytes", connection_id, data.len());
        Ok(())
    }

    /// Close a WebSocket connection
    pub async fn close(&self, connection_id: &str, code: Option<u16>, reason: Option<&str>) -> Result<()> {
        self.update_connection_state(connection_id, ConnectionState::Closing)?;

        // Simulate closing handshake
        tokio::time::sleep(Duration::from_millis(50)).await;

        self.update_connection_state(connection_id, ConnectionState::Closed)?;

        let mut connections = self.connections.lock().unwrap();
        connections.remove(connection_id);

        tracing::info!("WebSocket connection closed: {} (code: {:?}, reason: {:?})", 
                      connection_id, code, reason);
        Ok(())
    }

    /// Get connection status
    pub fn get_connection_state(&self, connection_id: &str) -> Result<ConnectionState> {
        let connections = self.connections.lock().unwrap();
        let connection = connections.get(connection_id)
            .ok_or_else(|| anyhow!("WebSocket connection not found: {}", connection_id))?;
        Ok(connection.state.clone())
    }

    /// Get all active connections
    pub fn get_active_connections(&self) -> Vec<String> {
        let connections = self.connections.lock().unwrap();
        connections.iter()
            .filter(|(_, conn)| matches!(conn.state, ConnectionState::Open))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Add a message handler for processing WebSocket messages
    pub fn add_message_handler<F>(&self, handler: F)
    where
        F: Fn(&WebSocketMessage) -> Result<Option<WebSocketMessage>> + Send + Sync + 'static,
    {
        let mut handlers = self.message_handlers.lock().unwrap();
        handlers.push(Box::new(handler));
    }

    /// Simulate WebSocket ping/pong mechanism
    pub async fn ping(&self, connection_id: &str, data: Option<&str>) -> Result<()> {
        let ping_message = WebSocketMessage {
            timestamp: Instant::now(),
            message_type: MessageType::Ping,
            data: data.unwrap_or("").to_string(),
            binary: false,
        };

        {
            let mut connections = self.connections.lock().unwrap();
            let connection = connections.get_mut(connection_id)
                .ok_or_else(|| anyhow!("WebSocket connection not found: {}", connection_id))?;

            connection.messages_sent.push(ping_message.clone());
            connection.last_ping = Instant::now();
        }

        // Simulate automatic pong response
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let pong_message = WebSocketMessage {
            timestamp: Instant::now(),
            message_type: MessageType::Pong,
            data: ping_message.data,
            binary: false,
        };

        {
            let mut connections = self.connections.lock().unwrap();
            let connection = connections.get_mut(connection_id)
                .ok_or_else(|| anyhow!("WebSocket connection not found: {}", connection_id))?;

            connection.messages_received.push(pong_message);
        }

        tracing::debug!("WebSocket ping/pong completed on {}", connection_id);
        Ok(())
    }

    /// Get message history for a connection
    pub fn get_message_history(&self, connection_id: &str) -> Result<(Vec<WebSocketMessage>, Vec<WebSocketMessage>)> {
        let connections = self.connections.lock().unwrap();
        let connection = connections.get(connection_id)
            .ok_or_else(|| anyhow!("WebSocket connection not found: {}", connection_id))?;
        
        Ok((connection.messages_sent.clone(), connection.messages_received.clone()))
    }

    /// Simulate real-time WebSocket events for modern web apps
    pub async fn simulate_realtime_events(&self, connection_id: &str, event_types: Vec<&str>) -> Result<()> {
        for event_type in event_types {
            let event_data = match event_type {
                "heartbeat" => r#"{"type":"heartbeat","timestamp":1234567890}"#,
                "user_joined" => r#"{"type":"user_joined","user_id":"user123","username":"testuser"}"#,
                "message" => r#"{"type":"message","id":"msg456","content":"Hello, World!","user":"system"}"#,
                "notification" => r#"{"type":"notification","title":"New Message","body":"You have a new message"}"#,
                "status_update" => r#"{"type":"status_update","status":"online","last_seen":1234567890}"#,
                _ => r#"{"type":"unknown","data":{}}"#,
            };

            self.simulate_incoming_message(connection_id, event_data, false).await?;
            
            // Add realistic delay between events
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Ok(())
    }

    fn update_connection_state(&self, connection_id: &str, state: ConnectionState) -> Result<()> {
        let mut connections = self.connections.lock().unwrap();
        let connection = connections.get_mut(connection_id)
            .ok_or_else(|| anyhow!("WebSocket connection not found: {}", connection_id))?;
        
        connection.state = state;
        Ok(())
    }

    async fn process_outgoing_message(&self, _connection_id: &str, message: &WebSocketMessage) -> Result<()> {
        // Process outgoing messages through handlers
        let handlers = self.message_handlers.lock().unwrap();
        for handler in handlers.iter() {
            if let Ok(Some(response)) = handler(message) {
                // In a real implementation, this would send the response back
                tracing::debug!("Generated WebSocket response: {:?}", response);
            }
        }
        Ok(())
    }

    async fn process_incoming_message(&self, _connection_id: &str, message: &WebSocketMessage) -> Result<()> {
        // Process incoming messages (could trigger JavaScript events, etc.)
        tracing::debug!("Processing incoming WebSocket message: {:?}", message);
        
        // Parse JSON messages for special handling
        if !message.binary {
            if let Ok(json_data) = serde_json::from_str::<Value>(&message.data) {
                if let Some(msg_type) = json_data.get("type").and_then(|v| v.as_str()) {
                    match msg_type {
                        "heartbeat" => tracing::debug!("Heartbeat received"),
                        "user_joined" => tracing::debug!("User joined event"),
                        "message" => tracing::debug!("Chat message received"),
                        _ => tracing::debug!("Unknown message type: {}", msg_type),
                    }
                }
            }
        }

        Ok(())
    }
}

/// WebSocket JavaScript API simulation for browser integration
pub struct WebSocketJsApi {
    pub manager: WebSocketManager,
}

impl WebSocketJsApi {
    pub fn new(manager: WebSocketManager) -> Self {
        Self { manager }
    }

    /// Setup WebSocket JavaScript API in Boa context
    pub fn setup_websocket_globals(&self, context: &mut boa_engine::Context) -> Result<()> {
        // Setup WebSocket constructor and API using simple JavaScript approach
        context.eval(boa_engine::Source::from_bytes(r#"
            // WebSocket constructor simulation
            function WebSocket(url, protocols) {
                this.url = url;
                this.protocols = protocols || [];
                this.readyState = 0; // CONNECTING
                this.onopen = null;
                this.onclose = null;
                this.onmessage = null;
                this.onerror = null;
                
                // Simulate connection establishment
                var self = this;
                setTimeout(function() {
                    self.readyState = 1; // OPEN
                    if (self.onopen) {
                        self.onopen({type: 'open'});
                    }
                }, 100);
                
                // Store connection for tracking
                this.connectionId = 'ws_' + Math.random().toString(36).substr(2, 9);
            }
            
            // WebSocket constants
            WebSocket.CONNECTING = 0;
            WebSocket.OPEN = 1;
            WebSocket.CLOSING = 2;
            WebSocket.CLOSED = 3;
            
            // WebSocket prototype methods
            WebSocket.prototype.send = function(data) {
                if (this.readyState !== 1) {
                    throw new Error('WebSocket is not open');
                }
                console.log('WebSocket send:', data);
                
                // Simulate echo response for testing
                var self = this;
                setTimeout(function() {
                    if (self.onmessage) {
                        self.onmessage({
                            type: 'message',
                            data: 'Echo: ' + data,
                            origin: self.url
                        });
                    }
                }, 50);
            };
            
            WebSocket.prototype.close = function(code, reason) {
                this.readyState = 2; // CLOSING
                var self = this;
                setTimeout(function() {
                    self.readyState = 3; // CLOSED
                    if (self.onclose) {
                        self.onclose({
                            type: 'close',
                            code: code || 1000,
                            reason: reason || ''
                        });
                    }
                }, 50);
            };
            
            // Add WebSocket to global scope
            window.WebSocket = WebSocket;
            
            // Server-Sent Events (SSE) support
            function EventSource(url) {
                this.url = url;
                this.readyState = 0; // CONNECTING
                this.onopen = null;
                this.onmessage = null;
                this.onerror = null;
                
                var self = this;
                setTimeout(function() {
                    self.readyState = 1; // OPEN
                    if (self.onopen) {
                        self.onopen({type: 'open'});
                    }
                    
                    // Simulate periodic events
                    setInterval(function() {
                        if (self.onmessage && self.readyState === 1) {
                            self.onmessage({
                                type: 'message',
                                data: JSON.stringify({
                                    timestamp: Date.now(),
                                    event: 'server_update'
                                }),
                                origin: self.url
                            });
                        }
                    }, 5000);
                }, 100);
            }
            
            EventSource.CONNECTING = 0;
            EventSource.OPEN = 1;
            EventSource.CLOSED = 2;
            
            EventSource.prototype.close = function() {
                this.readyState = 2; // CLOSED
            };
            
            window.EventSource = EventSource;
            
        "#)).map_err(|e| anyhow!("Failed to setup WebSocket globals: {}", e))?;

        Ok(())
    }

    /// Simulate WebSocket connection for testing
    pub async fn create_test_connection(&self, url: &str) -> Result<String> {
        self.manager.connect(url, Some(vec!["chat".to_string(), "notifications".to_string()])).await
    }

    /// Simulate WebSocket message exchange
    pub async fn simulate_message_exchange(&self, connection_id: &str) -> Result<()> {
        // Send some test messages
        self.manager.send_message(connection_id, r#"{"type":"join","room":"general"}"#, false).await?;
        
        // Simulate server responses
        self.manager.simulate_incoming_message(
            connection_id, 
            r#"{"type":"joined","room":"general","users":["user1","user2"]}"#, 
            false
        ).await?;
        
        self.manager.simulate_incoming_message(
            connection_id, 
            r#"{"type":"message","user":"user1","text":"Hello everyone!"}"#, 
            false
        ).await?;

        Ok(())
    }
}

// Add UUID dependency to Cargo.toml for connection IDs
// uuid = "1.0"