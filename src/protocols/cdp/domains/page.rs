use anyhow::Result;
use serde_json::Value;

use super::super::{CdpDomain, CdpEvent};

/// Page domain - Page lifecycle and navigation
#[derive(Debug)]
pub struct PageDomain {
    enabled: bool,
    screencast_enabled: bool,
    screencast_format: String,
    screencast_quality: i32,
    screencast_max_width: Option<i32>,
    screencast_max_height: Option<i32>,
    screencast_frame_counter: i32,
}

impl Default for PageDomain {
    fn default() -> Self {
        Self::new()
    }
}

impl PageDomain {
    pub fn new() -> Self {
        Self {
            enabled: false,
            screencast_enabled: false,
            screencast_format: "png".to_string(),
            screencast_quality: 80,
            screencast_max_width: None,
            screencast_max_height: None,
            screencast_frame_counter: 0,
        }
    }
}

impl CdpDomain for PageDomain {
    fn name(&self) -> &str {
        "Page"
    }

    fn handle_command(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        match method {
            "enable" => {
                self.enabled = true;
                Ok(serde_json::json!({}))
            }
            "disable" => {
                self.enabled = false;
                Ok(serde_json::json!({}))
            }
            "navigate" => {
                let params = params.unwrap_or_default();
                let _url = params.get("url").and_then(|v| v.as_str()).unwrap_or("");

                Ok(serde_json::json!({
                    "frameId": "main_frame",
                    "loaderId": "loader_1"
                }))
            }
            "reload" => Ok(serde_json::json!({})),
            "captureScreenshot" => {
                // Return a minimal base64 encoded 1x1 PNG
                Ok(serde_json::json!({
                    "data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg=="
                }))
            }
            "startScreencast" => {
                let params = params.unwrap_or_default();

                // Parse screencast parameters
                self.screencast_format = params
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("png")
                    .to_string();

                self.screencast_quality =
                    params.get("quality").and_then(|v| v.as_i64()).unwrap_or(80) as i32;

                self.screencast_max_width = params
                    .get("maxWidth")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32);

                self.screencast_max_height = params
                    .get("maxHeight")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32);

                self.screencast_enabled = true;
                self.screencast_frame_counter = 0;

                tracing::info!(
                    "Screencast started: format={}, quality={}, maxWidth={:?}, maxHeight={:?}",
                    self.screencast_format,
                    self.screencast_quality,
                    self.screencast_max_width,
                    self.screencast_max_height
                );

                Ok(serde_json::json!({}))
            }
            "stopScreencast" => {
                self.screencast_enabled = false;
                tracing::info!("Screencast stopped");
                Ok(serde_json::json!({}))
            }
            "screencastFrameAck" => {
                let params = params.unwrap_or_default();
                let _session_id = params
                    .get("sessionId")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);

                // Frame acknowledged by client, can send next frame
                Ok(serde_json::json!({}))
            }
            _ => Err(anyhow::anyhow!("Unknown Page method: {}", method)),
        }
    }

    fn get_events(&mut self) -> Vec<CdpEvent> {
        vec![]
    }
}
