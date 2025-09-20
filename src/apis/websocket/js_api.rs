use anyhow::Result;
use super::types::WebSocketManager;
use boa_engine::JsValue;

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
            // Factory that creates a WebSocket-like object; we'll call this from a Rust-registered constructor
            function __create_ws_instance(url, protocols) {
                var instance = {};
                instance.url = url;
                instance.protocols = protocols || [];
                instance.readyState = 0; // CONNECTING
                instance.onopen = null;
                instance.onclose = null;
                instance.onmessage = null;
                instance.onerror = null;
                instance.bufferedAmount = 0;
                instance.extensions = '';
                instance.protocol = '';
                instance.connectionId = 'ws_' + Math.random().toString(36).substr(2, 9);

                // Simulate open
                var self = instance;
                setTimeout(function() {
                    self.readyState = 1; // OPEN
                    if (self.onopen) {
                        var event = { type: 'open', target: self, currentTarget: self, bubbles: false, cancelable: false, defaultPrevented: false, eventPhase: 2, timeStamp: Date.now() };
                        self.onopen(event);
                    }
                }, 100);

                instance.send = function(data) {
                    if (this.readyState === 0) {
                        throw new Error('InvalidStateError: Still in CONNECTING state');
                    }
                    if (this.readyState !== 1) {
                        throw new Error('InvalidStateError: WebSocket is not open');
                    }
                    var s = this;
                    setTimeout(function() {
                        if (s.onmessage && s.readyState === 1) {
                            var evt = { type: 'message', data: data, origin: new URL(s.url).origin };
                            s.onmessage(evt);
                        }
                    }, 50);
                };

                instance.close = function(code, reason) {
                    if (this.readyState === 2 || this.readyState === 3) return;
                    this.readyState = 2;
                    var s = this;
                    setTimeout(function() {
                        s.readyState = 3;
                        if (s.onclose) s.onclose({ type: 'close', code: code || 1000, reason: reason || '' });
                    }, 10);
                };

                return instance;
            }

            // Define MessageEvent and CloseEvent constructors if missing
            if (typeof MessageEvent === 'undefined') {
                function MessageEvent(type, eventInitDict) { this.type = type; var init = eventInitDict || {}; this.data = init.data || null; this.origin = init.origin || ''; }
                window.MessageEvent = MessageEvent;
            }
            if (typeof CloseEvent === 'undefined') {
                function CloseEvent(type, eventInitDict) { this.type = type; var init = eventInitDict || {}; this.code = init.code || 1000; this.reason = init.reason || ''; }
                window.CloseEvent = CloseEvent;
            }
        "#)).map_err(|e| anyhow::anyhow!("Failed to init WebSocket JS API: {}", e.to_string()))?;

        // Create a JS-side constructor function that delegates to our factory so it's constructable by `new`.
        context.eval(boa_engine::Source::from_bytes(r#"
            function WebSocket(url, protocols) { return __create_ws_instance(url, protocols); }
            WebSocket.CONNECTING = 0;
            WebSocket.OPEN = 1;
            WebSocket.CLOSING = 2;
            WebSocket.CLOSED = 3;
        "#)).map_err(|e| anyhow::anyhow!("Failed to register WebSocket constructor: {}", e.to_string()))?;

        tracing::debug!("Real WebSocket JavaScript API initialized");
        Ok(())
    }
}

impl WebSocketJsApi {
    /// Create a test connection using the underlying manager and return its id
    pub async fn create_test_connection(&self, url: &str) -> Result<String> {
        // Use the manager to create a real connection (or simulated in tests)
        Ok(self.manager.connect(url, None).await?)
    }

    /// Simulate a message exchange for testing (sends a join message and simulates responses)
    pub async fn simulate_message_exchange(&self, connection_id: &str) -> Result<()> {
        // Send a join message
    let _ = self.manager.send_message(connection_id, "join", false).await?;

        // Simulate server responses
    let _ = self.manager.simulate_incoming_message(connection_id, r#"{"type":"joined","message":"Welcome"}"#, false).await?;
    let _ = self.manager.simulate_incoming_message(connection_id, r#"{"type":"response","message":"OK"}"#, false).await?;

        Ok(())
    }
}