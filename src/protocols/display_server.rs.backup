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

    /// CDP Screencast frame
    /// This is the efficient display streaming method used by services like Browserless
    ScreencastFrame {
        /// Base64-encoded frame data (JPEG or PNG)
        data: String,
        /// Metadata about the frame
        metadata: ScreencastFrameMetadata,
        /// Session ID for frame acknowledgment
        session_id: i32,
        timestamp: u64,
    },
}

/// Screencast frame metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreencastFrameMetadata {
    /// Offset from the top of the page in CSS pixels
    #[serde(rename = "offsetTop")]
    pub offset_top: f64,
    /// Page scale factor
    #[serde(rename = "pageScaleFactor")]
    pub page_scale_factor: f64,
    /// Width of device screen in CSS pixels
    #[serde(rename = "deviceWidth")]
    pub device_width: f64,
    /// Height of device screen in CSS pixels
    #[serde(rename = "deviceHeight")]
    pub device_height: f64,
    /// Width of scrollbar in CSS pixels
    #[serde(rename = "scrollOffsetX")]
    pub scroll_offset_x: f64,
    /// Height of scrollbar in CSS pixels
    #[serde(rename = "scrollOffsetY")]
    pub scroll_offset_y: f64,
    /// Timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<f64>,
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

    /// Start CDP screencast
    StartScreencast {
        /// Image format: "png" or "jpeg"
        format: Option<String>,
        /// Image quality (0-100) for JPEG
        quality: Option<i32>,
        /// Maximum width in pixels
        max_width: Option<i32>,
        /// Maximum height in pixels
        max_height: Option<i32>,
    },

    /// Stop CDP screencast
    StopScreencast,

    /// Acknowledge screencast frame (required for next frame to be sent)
    ScreencastFrameAck {
        session_id: i32,
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
                                // Strip security meta tags, inject proxy script, and rewrite image URLs
                                let cleaned_html = strip_security_meta_tags(html);
                                let with_proxy = inject_proxy_script(&cleaned_html, &url);
                                let processed_html = rewrite_image_urls(&with_proxy, &url);

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
                // Send navigation event BEFORE starting navigation to indicate loading
                self.send_to_client(client_id, DisplayMessage::Navigation {
                    url: url.clone(),
                    timestamp: current_timestamp(),
                }).await?;

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

                // Send HTML update (which will set loading=false on client)
                if let BrowserResponse::Success { data } = content_response {
                    if let Some(html) = data.get("content").and_then(|v| v.as_str()) {
                        // Strip security meta tags, inject proxy script, and rewrite image URLs
                        let cleaned_html = strip_security_meta_tags(html);
                        let with_proxy = inject_proxy_script(&cleaned_html, &url);
                        let processed_html = rewrite_image_urls(&with_proxy, &url);

                        self.send_to_client(client_id, DisplayMessage::HtmlUpdate {
                            html: processed_html,
                            url: url.clone(),
                            title: None,
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

            DisplayCommand::StartScreencast { format, quality, max_width, max_height } => {
                // TODO: Integrate with CDP Page.startScreencast
                info!("Starting screencast: format={:?}, quality={:?}, max_width={:?}, max_height={:?}",
                    format, quality, max_width, max_height);

                // For now, send an acknowledgment
                self.send_to_client(client_id, DisplayMessage::ConsoleLog {
                    level: "info".to_string(),
                    message: format!(
                        "Screencast started with format={:?}, quality={:?}",
                        format.as_deref().unwrap_or("png"),
                        quality.unwrap_or(80)
                    ),
                    timestamp: current_timestamp(),
                }).await?;
            }

            DisplayCommand::StopScreencast => {
                // TODO: Integrate with CDP Page.stopScreencast
                info!("Stopping screencast");

                self.send_to_client(client_id, DisplayMessage::ConsoleLog {
                    level: "info".to_string(),
                    message: "Screencast stopped".to_string(),
                    timestamp: current_timestamp(),
                }).await?;
            }

            DisplayCommand::ScreencastFrameAck { session_id } => {
                // TODO: Acknowledge frame receipt and allow next frame to be sent
                debug!("Screencast frame {} acknowledged", session_id);
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

                // Strip security meta tags, inject proxy script, and rewrite image URLs
                let cleaned_html = strip_security_meta_tags(html);
                let with_proxy = inject_proxy_script(&cleaned_html, &url);
                let processed_html = rewrite_image_urls(&with_proxy, &url);

                self.send_to_client(client_id, DisplayMessage::HtmlUpdate {
                    html: processed_html,
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

/// Strip security meta tags that prevent iframe embedding
fn strip_security_meta_tags(html: &str) -> String {
    use regex::Regex;

    let mut cleaned = html.to_string();

    // Remove X-Frame-Options meta tags
    let xframe_regex = Regex::new(r#"(?i)<meta[^>]*http-equiv=["']?x-frame-options["']?[^>]*>"#).unwrap();
    cleaned = xframe_regex.replace_all(&cleaned, "").to_string();

    // Remove CSP meta tags
    let csp_regex = Regex::new(r#"(?i)<meta[^>]*http-equiv=["']?content-security-policy["']?[^>]*>"#).unwrap();
    cleaned = csp_regex.replace_all(&cleaned, "").to_string();

    // Remove frame-ancestors directives from inline CSP
    let frame_ancestors_regex = Regex::new(r#"(?i)frame-ancestors\s+[^;]+;?"#).unwrap();
    cleaned = frame_ancestors_regex.replace_all(&cleaned, "").to_string();

    cleaned
}

/// Rewrite image URLs to use proxy
fn rewrite_image_urls(html: &str, base_url: &str) -> String {
    use regex::Regex;

    // Regex to match src attributes in img tags
    let img_regex = Regex::new(r#"<img([^>]*)\s+src=["']([^"']+)["']([^>]*)>"#).unwrap();

    img_regex.replace_all(html, |caps: &regex::Captures| {
        let before_src = &caps[1];
        let src = &caps[2];
        let after_src = &caps[3];

        // Convert relative URLs to absolute
        let absolute_url = if src.starts_with("http://") || src.starts_with("https://") {
            src.to_string()
        } else if src.starts_with("//") {
            format!("https:{}", src)
        } else if src.starts_with('/') {
            // Get base origin
            if let Ok(base) = url::Url::parse(base_url) {
                format!("{}://{}{}", base.scheme(), base.host_str().unwrap_or(""), src)
            } else {
                src.to_string()
            }
        } else {
            // Relative path
            if let Ok(base) = url::Url::parse(base_url) {
                base.join(src).map(|u| u.to_string()).unwrap_or_else(|_| src.to_string())
            } else {
                src.to_string()
            }
        };

        // URL encode the absolute URL for the proxy
        // Use absolute URL for proxy to work in sandboxed iframe
        let encoded_url = urlencoding::encode(&absolute_url);
        let proxy_url = format!("https://local.brainwires.net/api/thalora-display/proxy-image?url={}", encoded_url);

        format!(r#"<img{} src="{}"{}>"#, before_src, proxy_url, after_src)
    }).to_string()
}

/// Inject proxy script to intercept fetch/XHR requests
fn inject_proxy_script(html: &str, base_url: &str) -> String {
    // Script to intercept fetch and XMLHttpRequest
    let proxy_script = format!(r#"
<script>
(function() {{
    const PROXY_URL = 'https://local.brainwires.net/api/thalora-display/proxy-fetch';
    const BASE_URL = '{}';

    // Helper to make absolute URLs
    function makeAbsolute(url) {{
        try {{
            // Already absolute
            if (url.startsWith('http://') || url.startsWith('https://')) {{
                return url;
            }}
            // Protocol-relative
            if (url.startsWith('//')) {{
                return 'https:' + url;
            }}
            // Create absolute URL using base
            const base = new URL(BASE_URL);
            return new URL(url, base).href;
        }} catch (e) {{
            console.error('Failed to make absolute URL:', url, e);
            return url;
        }}
    }}

    // Intercept fetch
    const originalFetch = window.fetch;
    window.fetch = function(resource, options) {{
        let url;
        if (typeof resource === 'string') {{
            url = resource;
        }} else if (resource instanceof Request) {{
            url = resource.url;
        }} else {{
            url = resource;
        }}

        // Make URL absolute
        const absoluteUrl = makeAbsolute(url);

        // Only proxy external requests (not blob: or data:)
        if (absoluteUrl.startsWith('http://') || absoluteUrl.startsWith('https://')) {{
            const proxyUrl = PROXY_URL + '?url=' + encodeURIComponent(absoluteUrl);
            console.log('🔄 Proxying fetch:', absoluteUrl);
            return originalFetch(proxyUrl, options);
        }}

        return originalFetch(resource, options);
    }};

    // Intercept XMLHttpRequest
    const OriginalXHR = window.XMLHttpRequest;
    window.XMLHttpRequest = function() {{
        const xhr = new OriginalXHR();
        const originalOpen = xhr.open;

        xhr.open = function(method, url, ...args) {{
            // Make URL absolute
            const absoluteUrl = makeAbsolute(url);

            // Only proxy external requests
            if (absoluteUrl.startsWith('http://') || absoluteUrl.startsWith('https://')) {{
                const proxyUrl = PROXY_URL + '?url=' + encodeURIComponent(absoluteUrl);
                console.log('🔄 Proxying XHR:', absoluteUrl);
                return originalOpen.call(this, method, proxyUrl, ...args);
            }}

            return originalOpen.call(this, method, url, ...args);
        }};

        return xhr;
    }};

    // Suppress History API SecurityErrors by wrapping at the deepest level
    // Save original native methods BEFORE any page scripts load
    const HistoryProto = History.prototype;
    const nativePushState = HistoryProto.pushState;
    const nativeReplaceState = HistoryProto.replaceState;

    // Create error-suppressing wrapper
    const createSafeWrapper = (nativeMethod) => {{
        return function(...args) {{
            try {{
                return nativeMethod.apply(this, args);
            }} catch (e) {{
                // Silently ignore SecurityError in sandboxed iframe
                if (e.name !== 'SecurityError') {{
                    throw e;
                }}
                // Log suppressed error for debugging
                console.debug('🔇 Suppressed History API SecurityError:', e.message);
                return undefined;
            }}
        }};
    }};

    // Replace with safe wrappers using Object.defineProperty to lock them
    Object.defineProperty(HistoryProto, 'pushState', {{
        value: createSafeWrapper(nativePushState),
        writable: true,
        enumerable: true,
        configurable: true
    }});

    Object.defineProperty(HistoryProto, 'replaceState', {{
        value: createSafeWrapper(nativeReplaceState),
        writable: true,
        enumerable: true,
        configurable: true
    }});
}})();
</script>
"#, base_url);

    // Inject script IMMEDIATELY after <html> tag (before <head>)
    // This ensures our script runs BEFORE any other scripts load
    if html.contains("<html>") {
        // Find the position right after <html> tag
        if let Some(pos) = html.find(">") {
            if html[..pos+1].to_lowercase().contains("<html") {
                // Insert right after the <html> opening tag
                let (before, after) = html.split_at(pos + 1);
                return format!("{}{}{}", before, proxy_script, after);
            }
        }
        // Fallback: replace <html> tag
        html.replace("<html>", &format!("<html>{}", proxy_script))
    } else if html.contains("<head>") {
        // No <html> tag, inject after <head>
        html.replace("<head>", &format!("<head>{}", proxy_script))
    } else {
        // No structure, prepend
        format!("{}{}", proxy_script, html)
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
