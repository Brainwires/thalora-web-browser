use thalora::protocols::{DisplayMessage, DisplayCommand, ScreencastFrameMetadata};
use serde_json;

/// Test DisplayMessage serialization and deserialization
#[test]
fn test_display_message_connected_serialization() {
    let msg = DisplayMessage::Connected {
        session_id: "test-session-123".to_string(),
        timestamp: 1234567890,
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"type\":\"connected\""));
    assert!(json.contains("\"session_id\":\"test-session-123\""));
    assert!(json.contains("\"timestamp\":1234567890"));

    let deserialized: DisplayMessage = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayMessage::Connected { session_id, timestamp } => {
            assert_eq!(session_id, "test-session-123");
            assert_eq!(timestamp, 1234567890);
        }
        _ => panic!("Wrong message type"),
    }
}

/// Test HtmlUpdate message serialization
#[test]
fn test_display_message_html_update_serialization() {
    let msg = DisplayMessage::HtmlUpdate {
        html: "<html><body><h1>Test</h1></body></html>".to_string(),
        url: "https://example.com".to_string(),
        title: Some("Test Page".to_string()),
        timestamp: 1234567890,
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"type\":\"html_update\""));
    assert!(json.contains("https://example.com"));
    assert!(json.contains("Test Page"));

    let deserialized: DisplayMessage = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayMessage::HtmlUpdate { html, url, title, timestamp } => {
            assert!(html.contains("<h1>Test</h1>"));
            assert_eq!(url, "https://example.com");
            assert_eq!(title, Some("Test Page".to_string()));
            assert_eq!(timestamp, 1234567890);
        }
        _ => panic!("Wrong message type"),
    }
}

/// Test Navigation message serialization
#[test]
fn test_display_message_navigation_serialization() {
    let msg = DisplayMessage::Navigation {
        url: "https://example.com/page".to_string(),
        timestamp: 1234567890,
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"type\":\"navigation\""));
    assert!(json.contains("https://example.com/page"));

    let deserialized: DisplayMessage = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayMessage::Navigation { url, timestamp } => {
            assert_eq!(url, "https://example.com/page");
            assert_eq!(timestamp, 1234567890);
        }
        _ => panic!("Wrong message type"),
    }
}

/// Test ConsoleLog message serialization
#[test]
fn test_display_message_console_log_serialization() {
    let msg = DisplayMessage::ConsoleLog {
        level: "info".to_string(),
        message: "Test log message".to_string(),
        timestamp: 1234567890,
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"type\":\"console_log\""));
    assert!(json.contains("\"level\":\"info\""));
    assert!(json.contains("Test log message"));

    let deserialized: DisplayMessage = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayMessage::ConsoleLog { level, message, timestamp } => {
            assert_eq!(level, "info");
            assert_eq!(message, "Test log message");
            assert_eq!(timestamp, 1234567890);
        }
        _ => panic!("Wrong message type"),
    }
}

/// Test NetworkRequest message serialization
#[test]
fn test_display_message_network_request_serialization() {
    let msg = DisplayMessage::NetworkRequest {
        method: "GET".to_string(),
        url: "https://api.example.com/data".to_string(),
        status: Some(200),
        timestamp: 1234567890,
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"type\":\"network_request\""));
    assert!(json.contains("\"method\":\"GET\""));
    assert!(json.contains("\"status\":200"));

    let deserialized: DisplayMessage = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayMessage::NetworkRequest { method, url, status, timestamp } => {
            assert_eq!(method, "GET");
            assert_eq!(url, "https://api.example.com/data");
            assert_eq!(status, Some(200));
            assert_eq!(timestamp, 1234567890);
        }
        _ => panic!("Wrong message type"),
    }
}

/// Test StateUpdate message serialization
#[test]
fn test_display_message_state_update_serialization() {
    let msg = DisplayMessage::StateUpdate {
        can_go_back: true,
        can_go_forward: false,
        loading: true,
        timestamp: 1234567890,
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"type\":\"state_update\""));
    assert!(json.contains("\"can_go_back\":true"));
    assert!(json.contains("\"can_go_forward\":false"));
    assert!(json.contains("\"loading\":true"));

    let deserialized: DisplayMessage = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayMessage::StateUpdate { can_go_back, can_go_forward, loading, timestamp } => {
            assert_eq!(can_go_back, true);
            assert_eq!(can_go_forward, false);
            assert_eq!(loading, true);
            assert_eq!(timestamp, 1234567890);
        }
        _ => panic!("Wrong message type"),
    }
}

/// Test Error message serialization
#[test]
fn test_display_message_error_serialization() {
    let msg = DisplayMessage::Error {
        message: "Navigation failed".to_string(),
        timestamp: 1234567890,
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"type\":\"error\""));
    assert!(json.contains("Navigation failed"));

    let deserialized: DisplayMessage = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayMessage::Error { message, timestamp } => {
            assert_eq!(message, "Navigation failed");
            assert_eq!(timestamp, 1234567890);
        }
        _ => panic!("Wrong message type"),
    }
}

/// Test Ping message serialization
#[test]
fn test_display_message_ping_serialization() {
    let msg = DisplayMessage::Ping {
        timestamp: 1234567890,
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"type\":\"ping\""));
    assert!(json.contains("\"timestamp\":1234567890"));

    let deserialized: DisplayMessage = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayMessage::Ping { timestamp } => {
            assert_eq!(timestamp, 1234567890);
        }
        _ => panic!("Wrong message type"),
    }
}

/// Test ScreencastFrame message serialization
#[test]
fn test_display_message_screencast_frame_serialization() {
    let metadata = ScreencastFrameMetadata {
        offset_top: 0.0,
        page_scale_factor: 1.0,
        device_width: 1920.0,
        device_height: 1080.0,
        scroll_offset_x: 0.0,
        scroll_offset_y: 100.0,
        timestamp: Some(1234567890.0),
    };

    let msg = DisplayMessage::ScreencastFrame {
        data: "base64encodeddata".to_string(),
        metadata: metadata.clone(),
        session_id: 1,
        timestamp: 1234567890,
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"type\":\"screencast_frame\""));
    assert!(json.contains("base64encodeddata"));
    assert!(json.contains("\"session_id\":1"));

    let deserialized: DisplayMessage = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayMessage::ScreencastFrame { data, metadata, session_id, timestamp } => {
            assert_eq!(data, "base64encodeddata");
            assert_eq!(session_id, 1);
            assert_eq!(timestamp, 1234567890);
            assert_eq!(metadata.device_width, 1920.0);
            assert_eq!(metadata.device_height, 1080.0);
        }
        _ => panic!("Wrong message type"),
    }
}

/// Test DisplayCommand Navigate serialization
#[test]
fn test_display_command_navigate_serialization() {
    let cmd = DisplayCommand::Navigate {
        url: "https://example.com".to_string(),
    };

    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("\"type\":\"navigate\""));
    assert!(json.contains("https://example.com"));

    let deserialized: DisplayCommand = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayCommand::Navigate { url } => {
            assert_eq!(url, "https://example.com");
        }
        _ => panic!("Wrong command type"),
    }
}

/// Test DisplayCommand Back serialization
#[test]
fn test_display_command_back_serialization() {
    let cmd = DisplayCommand::Back;

    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("\"type\":\"back\""));

    let deserialized: DisplayCommand = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayCommand::Back => {
            // Success
        }
        _ => panic!("Wrong command type"),
    }
}

/// Test DisplayCommand Forward serialization
#[test]
fn test_display_command_forward_serialization() {
    let cmd = DisplayCommand::Forward;

    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("\"type\":\"forward\""));

    let deserialized: DisplayCommand = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayCommand::Forward => {
            // Success
        }
        _ => panic!("Wrong command type"),
    }
}

/// Test DisplayCommand Reload serialization
#[test]
fn test_display_command_reload_serialization() {
    let cmd = DisplayCommand::Reload;

    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("\"type\":\"reload\""));

    let deserialized: DisplayCommand = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayCommand::Reload => {
            // Success
        }
        _ => panic!("Wrong command type"),
    }
}

/// Test DisplayCommand Stop serialization
#[test]
fn test_display_command_stop_serialization() {
    let cmd = DisplayCommand::Stop;

    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("\"type\":\"stop\""));

    let deserialized: DisplayCommand = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayCommand::Stop => {
            // Success
        }
        _ => panic!("Wrong command type"),
    }
}

/// Test DisplayCommand ExecuteScript serialization
#[test]
fn test_display_command_execute_script_serialization() {
    let cmd = DisplayCommand::ExecuteScript {
        script: "console.log('test')".to_string(),
    };

    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("\"type\":\"execute_script\""));
    assert!(json.contains("console.log('test')"));

    let deserialized: DisplayCommand = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayCommand::ExecuteScript { script } => {
            assert_eq!(script, "console.log('test')");
        }
        _ => panic!("Wrong command type"),
    }
}

/// Test DisplayCommand Click serialization
#[test]
fn test_display_command_click_serialization() {
    let cmd = DisplayCommand::Click {
        selector: "#button".to_string(),
    };

    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("\"type\":\"click\""));
    assert!(json.contains("#button"));

    let deserialized: DisplayCommand = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayCommand::Click { selector } => {
            assert_eq!(selector, "#button");
        }
        _ => panic!("Wrong command type"),
    }
}

/// Test DisplayCommand Type serialization
#[test]
fn test_display_command_type_serialization() {
    let cmd = DisplayCommand::Type {
        selector: "#input".to_string(),
        text: "Hello World".to_string(),
    };

    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("\"type\":\"type\""));
    assert!(json.contains("#input"));
    assert!(json.contains("Hello World"));

    let deserialized: DisplayCommand = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayCommand::Type { selector, text } => {
            assert_eq!(selector, "#input");
            assert_eq!(text, "Hello World");
        }
        _ => panic!("Wrong command type"),
    }
}

/// Test DisplayCommand Pong serialization
#[test]
fn test_display_command_pong_serialization() {
    let cmd = DisplayCommand::Pong {
        timestamp: 1234567890,
    };

    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("\"type\":\"pong\""));
    assert!(json.contains("\"timestamp\":1234567890"));

    let deserialized: DisplayCommand = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayCommand::Pong { timestamp } => {
            assert_eq!(timestamp, 1234567890);
        }
        _ => panic!("Wrong command type"),
    }
}

/// Test DisplayCommand StartScreencast serialization
#[test]
fn test_display_command_start_screencast_serialization() {
    let cmd = DisplayCommand::StartScreencast {
        format: Some("jpeg".to_string()),
        quality: Some(80),
        max_width: Some(1920),
        max_height: Some(1080),
    };

    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("\"type\":\"start_screencast\""));
    assert!(json.contains("\"format\":\"jpeg\""));
    assert!(json.contains("\"quality\":80"));

    let deserialized: DisplayCommand = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayCommand::StartScreencast { format, quality, max_width, max_height } => {
            assert_eq!(format, Some("jpeg".to_string()));
            assert_eq!(quality, Some(80));
            assert_eq!(max_width, Some(1920));
            assert_eq!(max_height, Some(1080));
        }
        _ => panic!("Wrong command type"),
    }
}

/// Test DisplayCommand StartScreencast with defaults
#[test]
fn test_display_command_start_screencast_defaults() {
    let cmd = DisplayCommand::StartScreencast {
        format: None,
        quality: None,
        max_width: None,
        max_height: None,
    };

    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("\"type\":\"start_screencast\""));

    let deserialized: DisplayCommand = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayCommand::StartScreencast { format, quality, max_width, max_height } => {
            assert_eq!(format, None);
            assert_eq!(quality, None);
            assert_eq!(max_width, None);
            assert_eq!(max_height, None);
        }
        _ => panic!("Wrong command type"),
    }
}

/// Test DisplayCommand StopScreencast serialization
#[test]
fn test_display_command_stop_screencast_serialization() {
    let cmd = DisplayCommand::StopScreencast;

    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("\"type\":\"stop_screencast\""));

    let deserialized: DisplayCommand = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayCommand::StopScreencast => {
            // Success
        }
        _ => panic!("Wrong command type"),
    }
}

/// Test DisplayCommand ScreencastFrameAck serialization
#[test]
fn test_display_command_screencast_frame_ack_serialization() {
    let cmd = DisplayCommand::ScreencastFrameAck {
        session_id: 42,
    };

    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("\"type\":\"screencast_frame_ack\""));
    assert!(json.contains("\"session_id\":42"));

    let deserialized: DisplayCommand = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayCommand::ScreencastFrameAck { session_id } => {
            assert_eq!(session_id, 42);
        }
        _ => panic!("Wrong command type"),
    }
}

/// Test ScreencastFrameMetadata serialization
#[test]
fn test_screencast_frame_metadata_serialization() {
    let metadata = ScreencastFrameMetadata {
        offset_top: 10.5,
        page_scale_factor: 1.5,
        device_width: 1920.0,
        device_height: 1080.0,
        scroll_offset_x: 50.0,
        scroll_offset_y: 200.0,
        timestamp: Some(1234567890.5),
    };

    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("\"offsetTop\":10.5"));
    assert!(json.contains("\"pageScaleFactor\":1.5"));
    assert!(json.contains("\"deviceWidth\":1920"));
    assert!(json.contains("\"deviceHeight\":1080"));
    assert!(json.contains("\"scrollOffsetX\":50"));
    assert!(json.contains("\"scrollOffsetY\":200"));
    assert!(json.contains("\"timestamp\":1234567890.5"));

    let deserialized: ScreencastFrameMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.offset_top, 10.5);
    assert_eq!(deserialized.page_scale_factor, 1.5);
    assert_eq!(deserialized.device_width, 1920.0);
    assert_eq!(deserialized.device_height, 1080.0);
    assert_eq!(deserialized.scroll_offset_x, 50.0);
    assert_eq!(deserialized.scroll_offset_y, 200.0);
    assert_eq!(deserialized.timestamp, Some(1234567890.5));
}

/// Test ScreencastFrameMetadata without timestamp
#[test]
fn test_screencast_frame_metadata_no_timestamp() {
    let metadata = ScreencastFrameMetadata {
        offset_top: 0.0,
        page_scale_factor: 1.0,
        device_width: 1920.0,
        device_height: 1080.0,
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        timestamp: None,
    };

    let json = serde_json::to_string(&metadata).unwrap();
    // timestamp field should be omitted when None
    assert!(!json.contains("\"timestamp\""));

    let deserialized: ScreencastFrameMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.timestamp, None);
}

/// Test large HTML content in HtmlUpdate message
#[test]
fn test_display_message_large_html() {
    let large_html = "<html><body>".to_string() + &"<p>Test</p>".repeat(10000) + "</body></html>";

    let msg = DisplayMessage::HtmlUpdate {
        html: large_html.clone(),
        url: "https://example.com".to_string(),
        title: Some("Large Page".to_string()),
        timestamp: 1234567890,
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.len() > 100000); // Ensure it's actually large

    let deserialized: DisplayMessage = serde_json::from_str(&json).unwrap();
    match deserialized {
        DisplayMessage::HtmlUpdate { html, .. } => {
            assert_eq!(html.len(), large_html.len());
        }
        _ => panic!("Wrong message type"),
    }
}

/// Test special characters in messages
#[test]
fn test_display_message_special_characters() {
    let msg = DisplayMessage::ConsoleLog {
        level: "info".to_string(),
        message: "Test with special chars: \"quotes\", 'apostrophes', <tags>, & ampersands, 日本語".to_string(),
        timestamp: 1234567890,
    };

    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: DisplayMessage = serde_json::from_str(&json).unwrap();

    match deserialized {
        DisplayMessage::ConsoleLog { message, .. } => {
            assert!(message.contains("\"quotes\""));
            assert!(message.contains("'apostrophes'"));
            assert!(message.contains("<tags>"));
            assert!(message.contains("& ampersands"));
            assert!(message.contains("日本語"));
        }
        _ => panic!("Wrong message type"),
    }
}

/// Test message roundtrip (serialize -> deserialize -> serialize)
#[test]
fn test_message_roundtrip() {
    let original = DisplayMessage::StateUpdate {
        can_go_back: true,
        can_go_forward: false,
        loading: false,
        timestamp: 1234567890,
    };

    let json1 = serde_json::to_string(&original).unwrap();
    let deserialized: DisplayMessage = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&deserialized).unwrap();

    assert_eq!(json1, json2);
}

/// Test invalid JSON handling
#[test]
fn test_invalid_json_deserialization() {
    let invalid_json = "{\"type\":\"unknown_type\",\"data\":123}";
    let result: Result<DisplayMessage, _> = serde_json::from_str(invalid_json);

    assert!(result.is_err());
}

/// Test missing required fields
#[test]
fn test_missing_required_fields() {
    let incomplete_json = "{\"type\":\"navigate\"}"; // Missing 'url' field
    let result: Result<DisplayCommand, _> = serde_json::from_str(incomplete_json);

    assert!(result.is_err());
}

/// Test NetworkRequest without status
#[test]
fn test_network_request_no_status() {
    let msg = DisplayMessage::NetworkRequest {
        method: "GET".to_string(),
        url: "https://example.com".to_string(),
        status: None,
        timestamp: 1234567890,
    };

    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: DisplayMessage = serde_json::from_str(&json).unwrap();

    match deserialized {
        DisplayMessage::NetworkRequest { status, .. } => {
            assert_eq!(status, None);
        }
        _ => panic!("Wrong message type"),
    }
}

/// Test HtmlUpdate without title
#[test]
fn test_html_update_no_title() {
    let msg = DisplayMessage::HtmlUpdate {
        html: "<html><body>Test</body></html>".to_string(),
        url: "https://example.com".to_string(),
        title: None,
        timestamp: 1234567890,
    };

    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: DisplayMessage = serde_json::from_str(&json).unwrap();

    match deserialized {
        DisplayMessage::HtmlUpdate { title, .. } => {
            assert_eq!(title, None);
        }
        _ => panic!("Wrong message type"),
    }
}
