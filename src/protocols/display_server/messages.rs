/// Message types for Display Server protocol
///
/// Defines all message types sent between Thalora and display clients,
/// including serialization and deserialization.
use serde::{Deserialize, Serialize};

/// Message types sent from Thalora to display clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DisplayMessage {
    /// Initial connection established
    Connected { session_id: String, timestamp: u64 },

    /// HTML content update
    HtmlUpdate {
        html: String,
        url: String,
        title: Option<String>,
        timestamp: u64,
    },

    /// Navigation event
    Navigation { url: String, timestamp: u64 },

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
    Error { message: String, timestamp: u64 },

    /// Heartbeat/keepalive
    Ping { timestamp: u64 },

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
    Navigate { url: String },

    /// Go back in history
    Back,

    /// Go forward in history
    Forward,

    /// Reload current page
    Reload,

    /// Stop loading
    Stop,

    /// Execute JavaScript
    ExecuteScript { script: String },

    /// Click element
    Click { selector: String },

    /// Type text
    Type { selector: String, text: String },

    /// Pong response to ping
    Pong { timestamp: u64 },

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
    ScreencastFrameAck { session_id: i32 },
}

/// Get current timestamp in milliseconds
pub fn current_timestamp() -> u64 {
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
