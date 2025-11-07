/// Message handlers for display server commands
///
/// Processes client commands and coordinates with browser sessions.

use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::protocols::session_manager::{BrowserCommand, BrowserResponse, SessionManager};
use super::messages::{current_timestamp, DisplayCommand, DisplayMessage};
use super::sessions::ClientRegistry;

/// HTML content processing utilities
pub(super) mod processing {
    use regex::Regex;

    /// Strip security meta tags that prevent iframe embedding
    pub fn strip_security_meta_tags(html: &str) -> String {
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
    pub fn rewrite_image_urls(html: &str, base_url: &str) -> String {
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
    pub fn inject_proxy_script(html: &str, base_url: &str) -> String {
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

    /// Process HTML for display: strip security headers, inject proxy, rewrite URLs
    pub fn process_html(html: &str, url: &str) -> String {
        let cleaned_html = strip_security_meta_tags(html);
        let with_proxy = inject_proxy_script(&cleaned_html, url);
        rewrite_image_urls(&with_proxy, url)
    }
}

/// Command handler for display server
pub struct CommandHandler {
    session_manager: Arc<SessionManager>,
    client_registry: ClientRegistry,
}

impl CommandHandler {
    /// Create a new command handler
    pub fn new(session_manager: Arc<SessionManager>, client_registry: ClientRegistry) -> Self {
        Self {
            session_manager,
            client_registry,
        }
    }

    /// Handle a message from a display client
    pub async fn handle_client_message(
        &self,
        client_id: &str,
        session_id: &str,
        text: &str,
    ) -> Result<()> {
        let command: DisplayCommand = serde_json::from_str(text)
            .context("Failed to parse display command")?;

        info!("Client {} sent command: {:?}", client_id, command);

        match command {
            DisplayCommand::Navigate { url } => {
                self.handle_navigate(client_id, session_id, &url).await?;
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
                self.handle_execute_script(client_id, session_id, &script).await?;
            }

            DisplayCommand::Click { selector } => {
                self.handle_click(client_id, session_id, &selector).await?;
            }

            DisplayCommand::Type { selector, text } => {
                self.handle_type(session_id, &selector, &text).await?;
            }

            DisplayCommand::Pong { .. } => {
                // Keepalive response, no action needed
            }

            DisplayCommand::StartScreencast { format, quality, max_width, max_height } => {
                self.handle_start_screencast(client_id, format, quality, max_width, max_height).await?;
            }

            DisplayCommand::StopScreencast => {
                self.handle_stop_screencast(client_id).await?;
            }

            DisplayCommand::ScreencastFrameAck { session_id } => {
                // TODO: Acknowledge frame receipt and allow next frame to be sent
                debug!("Screencast frame {} acknowledged", session_id);
            }
        }

        Ok(())
    }

    /// Handle navigation command
    async fn handle_navigate(&self, client_id: &str, session_id: &str, url: &str) -> Result<()> {
        // Send navigation event BEFORE starting navigation to indicate loading
        self.client_registry.send_to_client(client_id, DisplayMessage::Navigation {
            url: url.to_string(),
            timestamp: current_timestamp(),
        })?;

        let _response = self.session_manager.send_command(
            session_id,
            BrowserCommand::Navigate {
                url: url.to_string(),
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
                let processed_html = processing::process_html(html, url);

                self.client_registry.send_to_client(client_id, DisplayMessage::HtmlUpdate {
                    html: processed_html,
                    url: url.to_string(),
                    title: None,
                    timestamp: current_timestamp(),
                })?;
            }
        }

        Ok(())
    }

    /// Handle execute script command
    async fn handle_execute_script(&self, client_id: &str, session_id: &str, script: &str) -> Result<()> {
        let response = self.session_manager.send_command(
            session_id,
            BrowserCommand::ExecuteJs { code: script.to_string() },
        ).await?;

        // Send result as console log
        if let BrowserResponse::Success { data } = response {
            self.client_registry.send_to_client(client_id, DisplayMessage::ConsoleLog {
                level: "info".to_string(),
                message: format!("Script result: {:?}", data),
                timestamp: current_timestamp(),
            })?;
        }

        Ok(())
    }

    /// Handle click command
    async fn handle_click(&self, client_id: &str, session_id: &str, selector: &str) -> Result<()> {
        let _response = self.session_manager.send_command(
            session_id,
            BrowserCommand::Click { selector: selector.to_string() },
        ).await?;

        // Refresh content after click
        self.refresh_client_content(client_id, session_id).await?;

        Ok(())
    }

    /// Handle type command
    async fn handle_type(&self, session_id: &str, selector: &str, text: &str) -> Result<()> {
        let _response = self.session_manager.send_command(
            session_id,
            BrowserCommand::Fill {
                selector: selector.to_string(),
                value: text.to_string()
            },
        ).await?;

        Ok(())
    }

    /// Handle start screencast command
    async fn handle_start_screencast(
        &self,
        client_id: &str,
        format: Option<String>,
        quality: Option<i32>,
        max_width: Option<i32>,
        max_height: Option<i32>,
    ) -> Result<()> {
        // TODO: Integrate with CDP Page.startScreencast
        info!("Starting screencast: format={:?}, quality={:?}, max_width={:?}, max_height={:?}",
            format, quality, max_width, max_height);

        // For now, send an acknowledgment
        self.client_registry.send_to_client(client_id, DisplayMessage::ConsoleLog {
            level: "info".to_string(),
            message: format!(
                "Screencast started with format={:?}, quality={:?}",
                format.as_deref().unwrap_or("png"),
                quality.unwrap_or(80)
            ),
            timestamp: current_timestamp(),
        })?;

        Ok(())
    }

    /// Handle stop screencast command
    async fn handle_stop_screencast(&self, client_id: &str) -> Result<()> {
        // TODO: Integrate with CDP Page.stopScreencast
        info!("Stopping screencast");

        self.client_registry.send_to_client(client_id, DisplayMessage::ConsoleLog {
            level: "info".to_string(),
            message: "Screencast stopped".to_string(),
            timestamp: current_timestamp(),
        })?;

        Ok(())
    }

    /// Refresh client's displayed content
    pub async fn refresh_client_content(&self, client_id: &str, session_id: &str) -> Result<()> {
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

                let processed_html = processing::process_html(html, &url);

                self.client_registry.send_to_client(client_id, DisplayMessage::HtmlUpdate {
                    html: processed_html,
                    url,
                    title: None,
                    timestamp: current_timestamp(),
                })?;
            }
        }

        Ok(())
    }
}
