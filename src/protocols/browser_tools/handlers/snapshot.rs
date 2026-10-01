use futures::FutureExt;
use serde_json::{Value, json};

use crate::engine::browser::snapshot::SnapshotOptions;
use crate::protocols::browser_tools::core::BrowserTools;
use crate::protocols::mcp::McpResponse;
use crate::protocols::security::sanitize_session_id;

impl BrowserTools {
    /// `browser_snapshot`: compact outline of the current page with element refs.
    pub async fn handle_snapshot(&self, params: Value) -> McpResponse {
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
        }
        let defaults = SnapshotOptions::default();
        let options = SnapshotOptions {
            max_tokens: params
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .map_or(defaults.max_tokens, |t| t.clamp(200, 50_000) as usize),
            interactive_only: params
                .get("interactive_only")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        };
        let focus_ref = params
            .get("focus_ref")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let browser = match self.get_session(session_id) {
            Ok(browser) => browser,
            Err(e) => return McpResponse::error(-32602, e),
        };
        crate::protocols::browser_tools::core::run_in(&browser, move |browser| {
            async move {
                let Ok(mut guard) = browser.lock() else {
                    return McpResponse::error(-1, "Failed to acquire browser lock".to_string());
                };
                match guard.snapshot(options, focus_ref.as_deref()) {
                    Ok(snapshot) => {
                        let url = guard.get_current_url();
                        let header = format!(
                            "url: {}\nrefs: {}{}\n",
                            url.as_deref().unwrap_or("about:blank"),
                            snapshot.ref_count,
                            if snapshot.truncated {
                                " (truncated)"
                            } else {
                                ""
                            }
                        );
                        McpResponse::page_content(
                            url.as_deref(),
                            Value::String(format!("{header}{}", snapshot.text)),
                        )
                    }
                    Err(e) => {
                        let message = e.to_string();
                        let code = if message.starts_with("stale_ref") {
                            -32602
                        } else {
                            -1
                        };
                        McpResponse::error(code, message)
                    }
                }
            }
            .boxed_local()
        })
        .await
    }

    /// `browser_screenshot`: PNG of the current page from the layout engine.
    pub async fn handle_screenshot(&self, params: Value) -> McpResponse {
        use crate::engine::renderer::paint::ScreenshotOptions;
        use base64::Engine;

        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
        }
        let defaults = ScreenshotOptions::default();
        let dimension = |key: &str, default: u32, min: u64, max: u64| {
            params
                .get(key)
                .and_then(|v| v.as_u64())
                .map_or(default, |v| v.clamp(min, max) as u32)
        };
        let options = ScreenshotOptions {
            width: dimension("width", defaults.width, 320, 2560),
            height: dimension("height", defaults.height, 240, 4000),
            full_page: params
                .get("full_page")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        };

        let browser = match self.get_session(session_id) {
            Ok(browser) => browser,
            Err(e) => return McpResponse::error(-32602, e),
        };
        crate::protocols::browser_tools::core::run_in(&browser, move |browser| {
            async move {
                let Ok(mut guard) = browser.lock() else {
                    return McpResponse::error(-1, "Failed to acquire browser lock".to_string());
                };
                match guard.screenshot_png(options) {
                    Ok(png) => {
                        let data = base64::engine::general_purpose::STANDARD.encode(&png);
                        McpResponse::success(json!([
                            {"type": "image", "data": data, "mimeType": "image/png"},
                            {
                                "type": "text",
                                "text": format!(
                                    "Screenshot of {} ({}x{}{}, approximate rendering)",
                                    guard.get_current_url().unwrap_or_default(),
                                    options.width,
                                    options.height,
                                    if options.full_page { ", full page" } else { "" }
                                )
                            }
                        ]))
                    }
                    Err(e) => McpResponse::error(-1, format!("Screenshot failed: {}", e)),
                }
            }
            .boxed_local()
        })
        .await
    }

    /// If `params` has a `ref` from `browser_snapshot`, resolve it to a CSS
    /// selector and store it as `params["selector"]` (a ref takes precedence
    /// over a selector). Stale or unknown refs are reported as errors.
    pub(crate) async fn resolve_ref_param(&self, mut params: Value) -> Result<Value, McpResponse> {
        let Some(element_ref) = params
            .get("ref")
            .and_then(|v| v.as_str())
            .map(str::to_string)
        else {
            return Ok(params);
        };
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();
        let browser = self
            .get_session(&session_id)
            .map_err(|e| McpResponse::error(-32602, e))?;
        let resolved = browser
            .call(move |browser| {
                async move {
                    let guard = browser
                        .lock()
                        .map_err(|_| (-1, "Failed to acquire browser lock".to_string()))?;
                    guard
                        .resolve_ref(&element_ref)
                        .map_err(|e| (-32602, e.to_string()))
                }
                .boxed_local()
            })
            .await
            .map_err(|e| McpResponse::error(-1, format!("Browser session failed: {}", e)))?;
        let selector = resolved.map_err(|(code, message)| McpResponse::error(code, message))?;
        params["selector"] = json!(selector);
        Ok(params)
    }
}
