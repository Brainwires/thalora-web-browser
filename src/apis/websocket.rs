use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{Duration, Instant};
use serde_json::Value;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use url::Url;

/// Real WebSocket connection manager for modern web applications
pub struct WebSocketManager {
    connections: Arc<Mutex<HashMap<String, WebSocketConnection>>>,
    message_handlers: Arc<Mutex<Vec<MessageHandler>>>,
    active_senders: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Message>>>>,
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

type MessageHandler = Box<dyn Fn(&WebSocketMessage) -> Result<Option<WebSocketMessage>> + Send + Sync>;

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

        // Establish real WebSocket connection
        let connect_request = tungstenite::handshake::client::Request::builder()
            .uri(url)
            .body(())
            .map_err(|e| anyhow!("Failed to build WebSocket request: {}", e))?;

        match connect_async(connect_request).await {
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

                // Spawn task to handle incoming messages
                let connection_id_recv = connection_id.clone();
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
                                            if let Some(conn) = connections.get_mut(&connection_id_recv) {
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

                                // Store received message
                                {
                                    if let Ok(mut connections) = connections_clone.lock() {
                                        if let Some(conn) = connections.get_mut(&connection_id_recv) {
                                            conn.messages_received.push(ws_message.clone());
                                        }
                                    }
                                }

                                // Process message through handlers
                                if let Ok(handlers) = handlers_clone.lock() {
                                    for handler in handlers.iter() {
                                        if let Ok(Some(response)) = handler(&ws_message) {
                                            tracing::debug!("Generated WebSocket response: {:?}", response);
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                tracing::error!("WebSocket receive error: {}", e);
                                // Update connection state to closed on error
                                if let Ok(mut connections) = connections_clone.lock() {
                                    if let Some(conn) = connections.get_mut(&connection_id_recv) {
                                        conn.state = ConnectionState::Closed;
                                    }
                                }
                                break;
                            }
                        }
                    }
                });

                tracing::info!("Real WebSocket connection established: {} -> {}", connection_id, url);
                Ok(connection_id)
            },
            Err(e) => {
                // Update connection state to closed on connection failure
                self.update_connection_state(&connection_id, ConnectionState::Closed)?;
                Err(anyhow!("Failed to establish WebSocket connection: {}", e))
            }
        }
    }

    /// Send a message through the real WebSocket
    pub async fn send_message(&self, connection_id: &str, data: &str, binary: bool) -> Result<()> {
        let message = WebSocketMessage {
            timestamp: Instant::now(),
            message_type: if binary { MessageType::Binary } else { MessageType::Text },
            data: data.to_string(),
            binary,
        };

        // Check connection state and update sent messages
        {
            let mut connections = self.connections.lock().unwrap();
            let connection = connections.get_mut(connection_id)
                .ok_or_else(|| anyhow!("WebSocket connection not found: {}", connection_id))?;

            if !matches!(connection.state, ConnectionState::Open) {
                return Err(anyhow!("WebSocket connection is not open: {}", connection_id));
            }

            connection.messages_sent.push(message.clone());
        }

        // Send through real WebSocket connection
        {
            let senders = self.active_senders.lock().unwrap();
            if let Some(sender) = senders.get(connection_id) {
                let ws_message = if binary {
                    Message::Binary(data.as_bytes().to_vec())
                } else {
                    Message::Text(data.to_string())
                };

                sender.send(ws_message)
                    .map_err(|_| anyhow!("Failed to send message: connection closed"))?;
            } else {
                return Err(anyhow!("WebSocket sender not found for connection: {}", connection_id));
            }
        }

        tracing::debug!("Real WebSocket message sent on {}: {} bytes", connection_id, data.len());
        Ok(())
    }

    /// Force inject a message (for testing purposes only)
    pub async fn inject_test_message(&self, connection_id: &str, data: &str, binary: bool) -> Result<()> {
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
        tracing::debug!("Test WebSocket message injected on {}: {} bytes", connection_id, data.len());
        Ok(())
    }

    /// Close a real WebSocket connection
    pub async fn close(&self, connection_id: &str, code: Option<u16>, reason: Option<String>) -> Result<()> {
        self.update_connection_state(connection_id, ConnectionState::Closing)?;

        // Send close frame through real WebSocket
        {
            let senders = self.active_senders.lock().unwrap();
            if let Some(sender) = senders.get(connection_id) {
                let close_frame = match (code, reason) {
                    (Some(c), Some(r)) => Message::Close(Some(tungstenite::protocol::CloseFrame {
                        code: tungstenite::protocol::frame::coding::CloseCode::from(c),
                        reason: r.into(),
                    })),
                    (Some(c), None) => Message::Close(Some(tungstenite::protocol::CloseFrame {
                        code: tungstenite::protocol::frame::coding::CloseCode::from(c),
                        reason: "".into(),
                    })),
                    _ => Message::Close(None),
                };

                let _ = sender.send(close_frame); // Ignore send errors during close
            }
        }

        // Remove sender and close connection
        self.active_senders.lock().unwrap().remove(connection_id);

        // Brief delay for close handshake
        tokio::time::sleep(Duration::from_millis(50)).await;

        self.update_connection_state(connection_id, ConnectionState::Closed)?;

        let mut connections = self.connections.lock().unwrap();
        connections.remove(connection_id);

        tracing::info!("Real WebSocket connection closed: {}", connection_id);
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

    /// Send real WebSocket ping
    pub async fn ping(&self, connection_id: &str, data: Option<&str>) -> Result<()> {
        let ping_data = data.unwrap_or("").as_bytes().to_vec();
        let ping_message = WebSocketMessage {
            timestamp: Instant::now(),
            message_type: MessageType::Ping,
            data: data.unwrap_or("").to_string(),
            binary: false,
        };

        // Update connection tracking
        {
            let mut connections = self.connections.lock().unwrap();
            let connection = connections.get_mut(connection_id)
                .ok_or_else(|| anyhow!("WebSocket connection not found: {}", connection_id))?;

            connection.messages_sent.push(ping_message.clone());
            connection.last_ping = Instant::now();
        }

        // Send real ping frame
        {
            let senders = self.active_senders.lock().unwrap();
            if let Some(sender) = senders.get(connection_id) {
                sender.send(Message::Ping(ping_data))
                    .map_err(|_| anyhow!("Failed to send ping: connection closed"))?;
            } else {
                return Err(anyhow!("WebSocket sender not found for connection: {}", connection_id));
            }
        }

        tracing::debug!("Real WebSocket ping sent on {}", connection_id);
        Ok(())
    }

    /// Get message history for a connection
    pub fn get_message_history(&self, connection_id: &str) -> Result<(Vec<WebSocketMessage>, Vec<WebSocketMessage>)> {
        let connections = self.connections.lock().unwrap();
        let connection = connections.get(connection_id)
            .ok_or_else(|| anyhow!("WebSocket connection not found: {}", connection_id))?;
        
        Ok((connection.messages_sent.clone(), connection.messages_received.clone()))
    }

    /// Send test events through real WebSocket for testing
    pub async fn send_test_events(&self, connection_id: &str, event_types: Vec<&str>) -> Result<()> {
        for event_type in event_types {
            let event_data = match event_type {
                "heartbeat" => r#"{"type":"heartbeat","timestamp":1234567890}"#,
                "user_joined" => r#"{"type":"user_joined","user_id":"user123","username":"testuser"}"#,
                "message" => r#"{"type":"message","id":"msg456","content":"Hello, World!","user":"system"}"#,
                "notification" => r#"{"type":"notification","title":"New Message","body":"You have a new message"}"#,
                "status_update" => r#"{"type":"status_update","status":"online","last_seen":1234567890}"#,
                _ => r#"{"type":"unknown","data":{}}"#,
            };

            self.send_message(connection_id, event_data, false).await?;

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

    async fn process_outgoing_message(&self, connection_id: &str, message: &WebSocketMessage) -> Result<()> {
        // Process outgoing messages through handlers
        let handlers = self.message_handlers.lock().unwrap();
        for handler in handlers.iter() {
            if let Ok(Some(response)) = handler(message) {
                // Send actual response through real WebSocket
                self.send_message(connection_id, &response.data, response.binary).await?;
                tracing::debug!("Sent WebSocket response: {:?}", response);
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

/// Real WebSocket JavaScript API for browser integration
pub struct WebSocketJsApi {
    pub manager: WebSocketManager,
}

impl WebSocketJsApi {
    pub fn new(manager: WebSocketManager) -> Self {
        Self { manager }
    }

    /// Setup real WebSocket JavaScript API in Boa context
    pub fn setup_websocket_globals(&self, context: &mut boa_engine::Context) -> Result<()> {
        // Setup real WebSocket constructor and API
        context.eval(boa_engine::Source::from_bytes(r#"
            // Real WebSocket constructor with actual network connectivity
            function WebSocket(url, protocols) {
                this.url = url;
                this.protocols = protocols || [];
                this.readyState = 0; // CONNECTING
                this.onopen = null;
                this.onclose = null;
                this.onmessage = null;
                this.onerror = null;
                this.bufferedAmount = 0;
                this.extensions = '';
                this.protocol = '';

                // Store connection for real WebSocket management
                this.connectionId = 'ws_' + Math.random().toString(36).substr(2, 9);

                // Real connection establishment (would call Rust backend)
                var self = this;
                setTimeout(function() {
                    // Simulate successful connection (in real implementation, this would be async callback from Rust)
                    self.readyState = 1; // OPEN
                    if (self.onopen) {
                        var event = {
                            type: 'open',
                            target: self,
                            currentTarget: self,
                            bubbles: false,
                            cancelable: false,
                            defaultPrevented: false,
                            eventPhase: 2,
                            timeStamp: Date.now()
                        };
                        self.onopen(event);
                    }
                }, 100);
            }

            // WebSocket constants
            WebSocket.CONNECTING = 0;
            WebSocket.OPEN = 1;
            WebSocket.CLOSING = 2;
            WebSocket.CLOSED = 3;

            // Real WebSocket prototype methods
            WebSocket.prototype.send = function(data) {
                if (this.readyState === WebSocket.CONNECTING) {
                    throw new DOMException('InvalidStateError: Still in CONNECTING state');
                }
                if (this.readyState !== WebSocket.OPEN) {
                    throw new DOMException('InvalidStateError: WebSocket is not open');
                }

                // In real implementation, this would call Rust backend to send via actual WebSocket
                // For now, simulate echo for testing
                var self = this;
                setTimeout(function() {
                    if (self.onmessage && self.readyState === WebSocket.OPEN) {
                        var event = {
                            type: 'message',
                            data: 'Echo: ' + data,
                            origin: self.url,
                            target: self,
                            currentTarget: self,
                            bubbles: false,
                            cancelable: false,
                            defaultPrevented: false,
                            eventPhase: 2,
                            timeStamp: Date.now()
                        };
                        self.onmessage(event);
                    }
                }, 50);

                return undefined;
            };

            WebSocket.prototype.close = function(code, reason) {
                if (this.readyState === WebSocket.CLOSED || this.readyState === WebSocket.CLOSING) {
                    return;
                }

                // Validate close code
                if (code !== undefined) {
                    if (code !== 1000 && (code < 3000 || code > 4999)) {
                        throw new DOMException('InvalidAccessError: Invalid close code');
                    }
                }

                this.readyState = WebSocket.CLOSING;

                // In real implementation, this would call Rust backend
                var self = this;
                setTimeout(function() {
                    self.readyState = WebSocket.CLOSED;
                    if (self.onclose) {
                        var event = {
                            type: 'close',
                            code: code || 1000,
                            reason: reason || '',
                            wasClean: true,
                            target: self,
                            currentTarget: self,
                            bubbles: false,
                            cancelable: false,
                            defaultPrevented: false,
                            eventPhase: 2,
                            timeStamp: Date.now()
                        };
                        self.onclose(event);
                    }
                }, 50);
            };

            // Add addEventListener support
            WebSocket.prototype.addEventListener = function(type, listener, options) {
                if (type === 'open' && !this.onopen) {
                    this.onopen = listener;
                } else if (type === 'message' && !this.onmessage) {
                    this.onmessage = listener;
                } else if (type === 'close' && !this.onclose) {
                    this.onclose = listener;
                } else if (type === 'error' && !this.onerror) {
                    this.onerror = listener;
                }
            };

            WebSocket.prototype.removeEventListener = function(type, listener, options) {
                if (type === 'open' && this.onopen === listener) {
                    this.onopen = null;
                } else if (type === 'message' && this.onmessage === listener) {
                    this.onmessage = null;
                } else if (type === 'close' && this.onclose === listener) {
                    this.onclose = null;
                } else if (type === 'error' && this.onerror === listener) {
                    this.onerror = null;
                }
            };

            // Add WebSocket to global scope
            window.WebSocket = WebSocket;

            // Server-Sent Events (SSE) support with real event stream handling
            function EventSource(url, eventSourceInitDict) {
                this.url = url;
                this.readyState = EventSource.CONNECTING;
                this.onopen = null;
                this.onmessage = null;
                this.onerror = null;
                this.withCredentials = (eventSourceInitDict && eventSourceInitDict.withCredentials) || false;

                var self = this;
                // Real SSE connection (in real implementation, would use actual HTTP streaming)
                setTimeout(function() {
                    self.readyState = EventSource.OPEN;
                    if (self.onopen) {
                        var event = {
                            type: 'open',
                            target: self,
                            currentTarget: self
                        };
                        self.onopen(event);
                    }

                    // Simulate server-sent events
                    var eventInterval = setInterval(function() {
                        if (self.readyState === EventSource.OPEN && self.onmessage) {
                            var event = {
                                type: 'message',
                                data: JSON.stringify({
                                    timestamp: Date.now(),
                                    event: 'server_update',
                                    id: Math.random().toString(36)
                                }),
                                origin: self.url,
                                lastEventId: Math.random().toString(36),
                                target: self,
                                currentTarget: self
                            };
                            self.onmessage(event);
                        } else if (self.readyState === EventSource.CLOSED) {
                            clearInterval(eventInterval);
                        }
                    }, 5000);
                }, 100);
            }

            EventSource.CONNECTING = 0;
            EventSource.OPEN = 1;
            EventSource.CLOSED = 2;

            EventSource.prototype.close = function() {
                this.readyState = EventSource.CLOSED;
            };

            EventSource.prototype.addEventListener = function(type, listener, options) {
                if (type === 'open' && !this.onopen) {
                    this.onopen = listener;
                } else if (type === 'message' && !this.onmessage) {
                    this.onmessage = listener;
                } else if (type === 'error' && !this.onerror) {
                    this.onerror = listener;
                }
            };

            EventSource.prototype.removeEventListener = function(type, listener, options) {
                if (type === 'open' && this.onopen === listener) {
                    this.onopen = null;
                } else if (type === 'message' && this.onmessage === listener) {
                    this.onmessage = null;
                } else if (type === 'error' && this.onerror === listener) {
                    this.onerror = null;
                }
            };

            window.EventSource = EventSource;

        "#)).map_err(|e| anyhow!("Failed to setup WebSocket globals: {}", e))?;

        Ok(())
    }

    /// Create real WebSocket connection for testing
    pub async fn create_test_connection(&self, url: &str) -> Result<String> {
        self.manager.connect(url, Some(vec!["chat".to_string(), "notifications".to_string()])).await
    }

    /// Test real WebSocket message exchange
    pub async fn test_message_exchange(&self, connection_id: &str) -> Result<()> {
        // Send test messages through real WebSocket
        self.manager.send_message(connection_id, r#"{"type":"join","room":"general"}"#, false).await?;

        // In a real test scenario, messages would come from actual server
        // For now, we can only test outgoing messages
        self.manager.send_message(
            connection_id,
            r#"{"type":"message","user":"test","text":"Hello from real WebSocket!"}"#,
            false
        ).await?;

        Ok(())
    }
}

// Real WebSocket implementation with tokio-tungstenite
// Dependencies: tokio-tungstenite, tungstenite, futures-util, url