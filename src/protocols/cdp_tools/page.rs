use crate::protocols::cdp::{CdpCommand, CdpMessage, CdpServer};
use crate::protocols::mcp::McpResponse;
use serde_json::Value;

/// Page domain - Page reload, screenshots, navigation, and lifecycle events
pub struct PageTools;

impl PageTools {
    pub fn new() -> Self {
        Self
    }

    pub async fn take_screenshot(
        &mut self,
        args: Value,
        cdp_server: &mut CdpServer,
    ) -> McpResponse {
        let format = args.get("format").and_then(|v| v.as_str()).unwrap_or("png");
        let quality = args.get("quality").and_then(|v| v.as_i64()).unwrap_or(80);
        let full_page = args
            .get("full_page")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut params = serde_json::json!({
            "format": format
        });

        if format == "jpeg" {
            params["quality"] = serde_json::Value::Number(serde_json::Number::from(quality));
        }

        if full_page {
            params["captureBeyondViewport"] = serde_json::Value::Bool(true);
        }

        let command = CdpCommand {
            id: 17,
            method: "Page.captureScreenshot".to_string(),
            params: Some(params),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::error(-1, format!("Screenshot failed: {}", error.message))
                } else if let Some(result) = response.result {
                    if let Some(data) = result.get("data") {
                        McpResponse::success(serde_json::json!({
                            "type": "image",
                            "data": data,
                            "mimeType": format!("image/{}", format)
                        }))
                    } else {
                        McpResponse::success(serde_json::json!({
                            "type": "text",
                            "text": format!("Screenshot captured: {}", result)
                        }))
                    }
                } else {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": "Screenshot capture completed"
                    }))
                }
            }
            Ok(_) => McpResponse::error(-1, "No response from CDP server".to_string()),
            Err(err) => McpResponse::error(-1, format!("CDP screenshot error: {}", err)),
        }
    }

    pub async fn reload_page(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let ignore_cache = args
            .get("ignore_cache")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let command = CdpCommand {
            id: 18,
            method: "Page.reload".to_string(),
            params: Some(serde_json::json!({
                "ignoreCache": ignore_cache
            })),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::error(-1, format!("Page reload failed: {}", error.message))
                } else {
                    let cache_msg = if ignore_cache {
                        " (ignoring cache)"
                    } else {
                        ""
                    };
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": format!("Page reloaded successfully{}", cache_msg)
                    }))
                }
            }
            Ok(_) => McpResponse::error(-1, "No response from CDP server".to_string()),
            Err(err) => McpResponse::error(-1, format!("CDP page reload error: {}", err)),
        }
    }
}
