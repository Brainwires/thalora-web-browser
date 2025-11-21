use thalora::protocols::{CdpCommand, CdpMessage, CdpResponse, CdpError};
use serde_json::json;

/// Test CdpCommand serialization
#[test]
fn test_cdp_command_serialization() {
    let command = CdpCommand {
        id: 1,
        method: "Runtime.enable".to_string(),
        params: None,
        session_id: None,
    };

    let json_str = serde_json::to_string(&command).unwrap();
    assert!(json_str.contains("\"id\":1"));
    assert!(json_str.contains("\"method\":\"Runtime.enable\""));
}

/// Test CdpCommand with params serialization
#[test]
fn test_cdp_command_with_params() {
    let command = CdpCommand {
        id: 2,
        method: "Runtime.evaluate".to_string(),
        params: Some(json!({
            "expression": "2 + 2",
            "returnByValue": true
        })),
        session_id: None,
    };

    let json_str = serde_json::to_string(&command).unwrap();
    assert!(json_str.contains("\"method\":\"Runtime.evaluate\""));
    assert!(json_str.contains("2 + 2"));
    assert!(json_str.contains("returnByValue"));
}

/// Test CdpCommand with session_id
#[test]
fn test_cdp_command_with_session() {
    let command = CdpCommand {
        id: 3,
        method: "Page.navigate".to_string(),
        params: Some(json!({
            "url": "https://example.com"
        })),
        session_id: Some("session-123".to_string()),
    };

    let json_str = serde_json::to_string(&command).unwrap();
    assert!(json_str.contains("\"sessionId\":\"session-123\""));
}

/// Test CdpCommand deserialization
#[test]
fn test_cdp_command_deserialization() {
    let json_str = r#"{"id":1,"method":"Runtime.enable"}"#;
    let command: CdpCommand = serde_json::from_str(json_str).unwrap();

    assert_eq!(command.id, 1);
    assert_eq!(command.method, "Runtime.enable");
    assert_eq!(command.params, None);
    assert_eq!(command.session_id, None);
}

/// Test CdpResponse success serialization
#[test]
fn test_cdp_response_success() {
    let response = CdpResponse {
        id: 1,
        result: Some(json!({"type": "undefined"})),
        error: None,
    };

    let json_str = serde_json::to_string(&response).unwrap();
    assert!(json_str.contains("\"id\":1"));
    assert!(json_str.contains("undefined"));
}

/// Test CdpResponse error serialization
#[test]
fn test_cdp_response_error() {
    let error = CdpError {
        code: -32000,
        message: "Runtime not enabled".to_string(),
        data: None,
    };

    let response = CdpResponse {
        id: 1,
        result: None,
        error: Some(error),
    };

    let json_str = serde_json::to_string(&response).unwrap();
    assert!(json_str.contains("\"code\":-32000"));
    assert!(json_str.contains("Runtime not enabled"));
}

/// Test CdpError serialization
#[test]
fn test_cdp_error_serialization() {
    let error = CdpError {
        code: -32600,
        message: "Invalid request".to_string(),
        data: Some(json!({"details": "Missing method"})),
    };

    let json_str = serde_json::to_string(&error).unwrap();
    assert!(json_str.contains("\"code\":-32600"));
    assert!(json_str.contains("Invalid request"));
    assert!(json_str.contains("Missing method"));
}

/// Test CdpMessage command variant
#[test]
fn test_cdp_message_command() {
    let command = CdpCommand {
        id: 1,
        method: "Runtime.enable".to_string(),
        params: None,
        session_id: None,
    };

    let message = CdpMessage::Command(command);

    match message {
        CdpMessage::Command(cmd) => {
            assert_eq!(cmd.method, "Runtime.enable");
        }
        _ => panic!("Wrong message type"),
    }
}

/// Test CdpMessage response variant
#[test]
fn test_cdp_message_response() {
    let response = CdpResponse {
        id: 1,
        result: Some(json!({"success": true})),
        error: None,
    };

    let message = CdpMessage::Response(response);

    match message {
        CdpMessage::Response(resp) => {
            assert_eq!(resp.id, 1);
            assert!(resp.error.is_none());
        }
        _ => panic!("Wrong message type"),
    }
}

/// Test Runtime.enable command
#[test]
fn test_runtime_enable_command() {
    let command = CdpCommand {
        id: 1,
        method: "Runtime.enable".to_string(),
        params: None,
        session_id: None,
    };

    assert_eq!(command.method, "Runtime.enable");
    assert!(command.params.is_none());
}

/// Test Runtime.evaluate command
#[test]
fn test_runtime_evaluate_command() {
    let command = CdpCommand {
        id: 2,
        method: "Runtime.evaluate".to_string(),
        params: Some(json!({
            "expression": "console.log('test')",
            "returnByValue": true
        })),
        session_id: None,
    };

    assert_eq!(command.method, "Runtime.evaluate");
    assert!(command.params.is_some());

    let params = command.params.unwrap();
    assert_eq!(params["expression"], "console.log('test')");
    assert_eq!(params["returnByValue"], true);
}

/// Test Debugger.enable command
#[test]
fn test_debugger_enable_command() {
    let command = CdpCommand {
        id: 3,
        method: "Debugger.enable".to_string(),
        params: None,
        session_id: None,
    };

    assert_eq!(command.method, "Debugger.enable");
}

/// Test Debugger.setBreakpoint command
#[test]
fn test_debugger_set_breakpoint_command() {
    let command = CdpCommand {
        id: 4,
        method: "Debugger.setBreakpoint".to_string(),
        params: Some(json!({
            "location": {
                "scriptId": "1",
                "lineNumber": 10
            }
        })),
        session_id: None,
    };

    assert_eq!(command.method, "Debugger.setBreakpoint");
    let params = command.params.unwrap();
    assert_eq!(params["location"]["lineNumber"], 10);
}

/// Test DOM.querySelector command
#[test]
fn test_dom_query_selector_command() {
    let command = CdpCommand {
        id: 5,
        method: "DOM.querySelector".to_string(),
        params: Some(json!({
            "nodeId": 1,
            "selector": "#myElement"
        })),
        session_id: None,
    };

    assert_eq!(command.method, "DOM.querySelector");
    let params = command.params.unwrap();
    assert_eq!(params["selector"], "#myElement");
}

/// Test DOM.getDocument command
#[test]
fn test_dom_get_document_command() {
    let command = CdpCommand {
        id: 6,
        method: "DOM.getDocument".to_string(),
        params: None,
        session_id: None,
    };

    assert_eq!(command.method, "DOM.getDocument");
}

/// Test Network.enable command
#[test]
fn test_network_enable_command() {
    let command = CdpCommand {
        id: 7,
        method: "Network.enable".to_string(),
        params: None,
        session_id: None,
    };

    assert_eq!(command.method, "Network.enable");
}

/// Test Network.setRequestInterception command
#[test]
fn test_network_set_request_interception_command() {
    let command = CdpCommand {
        id: 8,
        method: "Network.setRequestInterception".to_string(),
        params: Some(json!({
            "patterns": [
                {"urlPattern": "*"}
            ]
        })),
        session_id: None,
    };

    assert_eq!(command.method, "Network.setRequestInterception");
    let params = command.params.unwrap();
    assert!(params["patterns"].is_array());
}

/// Test Page.navigate command
#[test]
fn test_page_navigate_command() {
    let command = CdpCommand {
        id: 9,
        method: "Page.navigate".to_string(),
        params: Some(json!({
            "url": "https://example.com"
        })),
        session_id: None,
    };

    assert_eq!(command.method, "Page.navigate");
    let params = command.params.unwrap();
    assert_eq!(params["url"], "https://example.com");
}

/// Test Page.reload command
#[test]
fn test_page_reload_command() {
    let command = CdpCommand {
        id: 10,
        method: "Page.reload".to_string(),
        params: None,
        session_id: None,
    };

    assert_eq!(command.method, "Page.reload");
}

/// Test Page.getLayoutMetrics command
#[test]
fn test_page_get_layout_metrics_command() {
    let command = CdpCommand {
        id: 11,
        method: "Page.getLayoutMetrics".to_string(),
        params: None,
        session_id: None,
    };

    assert_eq!(command.method, "Page.getLayoutMetrics");
}

/// Test Console.enable command
#[test]
fn test_console_enable_command() {
    let command = CdpCommand {
        id: 12,
        method: "Console.enable".to_string(),
        params: None,
        session_id: None,
    };

    assert_eq!(command.method, "Console.enable");
}

/// Test Performance.enable command
#[test]
fn test_performance_enable_command() {
    let command = CdpCommand {
        id: 13,
        method: "Performance.enable".to_string(),
        params: None,
        session_id: None,
    };

    assert_eq!(command.method, "Performance.enable");
}

/// Test Target.createTarget command
#[test]
fn test_target_create_target_command() {
    let command = CdpCommand {
        id: 14,
        method: "Target.createTarget".to_string(),
        params: Some(json!({
            "url": "about:blank"
        })),
        session_id: None,
    };

    assert_eq!(command.method, "Target.createTarget");
    let params = command.params.unwrap();
    assert_eq!(params["url"], "about:blank");
}

/// Test successful JavaScript evaluation response
#[test]
fn test_javascript_evaluation_response_success() {
    let response = CdpResponse {
        id: 2,
        result: Some(json!({
            "result": {
                "type": "number",
                "value": 4
            }
        })),
        error: None,
    };

    assert_eq!(response.id, 2);
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    assert_eq!(result["result"]["type"], "number");
    assert_eq!(result["result"]["value"], 4);
}

/// Test JavaScript evaluation error response
#[test]
fn test_javascript_evaluation_response_error() {
    let error = CdpError {
        code: -32000,
        message: "SyntaxError: Unexpected token".to_string(),
        data: None,
    };

    let response = CdpResponse {
        id: 2,
        result: None,
        error: Some(error),
    };

    assert!(response.result.is_none());
    assert!(response.error.is_some());

    let err = response.error.unwrap();
    assert_eq!(err.code, -32000);
    assert!(err.message.contains("SyntaxError"));
}

/// Test DOM query response
#[test]
fn test_dom_query_response() {
    let response = CdpResponse {
        id: 5,
        result: Some(json!({
            "nodeId": 42
        })),
        error: None,
    };

    let result = response.result.unwrap();
    assert_eq!(result["nodeId"], 42);
}

/// Test Network request interception response
#[test]
fn test_network_interception_response() {
    let response = CdpResponse {
        id: 8,
        result: Some(json!({})),
        error: None,
    };

    assert!(response.error.is_none());
}

/// Test multiple CDP commands in sequence
#[test]
fn test_multiple_commands_sequence() {
    let commands = vec![
        CdpCommand {
            id: 1,
            method: "Runtime.enable".to_string(),
            params: None,
            session_id: None,
        },
        CdpCommand {
            id: 2,
            method: "Debugger.enable".to_string(),
            params: None,
            session_id: None,
        },
        CdpCommand {
            id: 3,
            method: "Network.enable".to_string(),
            params: None,
            session_id: None,
        },
    ];

    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].method, "Runtime.enable");
    assert_eq!(commands[1].method, "Debugger.enable");
    assert_eq!(commands[2].method, "Network.enable");
}

/// Test CDP command ID incrementation
#[test]
fn test_command_id_incrementation() {
    let mut id = 1;

    let cmd1 = CdpCommand {
        id,
        method: "Test1".to_string(),
        params: None,
        session_id: None,
    };
    id += 1;

    let cmd2 = CdpCommand {
        id,
        method: "Test2".to_string(),
        params: None,
        session_id: None,
    };
    id += 1;

    let cmd3 = CdpCommand {
        id,
        method: "Test3".to_string(),
        params: None,
        session_id: None,
    };

    assert_eq!(cmd1.id, 1);
    assert_eq!(cmd2.id, 2);
    assert_eq!(cmd3.id, 3);
}

/// Test CDP error codes
#[test]
fn test_cdp_error_codes() {
    let parse_error = CdpError {
        code: -32700,
        message: "Parse error".to_string(),
        data: None,
    };

    let invalid_request = CdpError {
        code: -32600,
        message: "Invalid Request".to_string(),
        data: None,
    };

    let method_not_found = CdpError {
        code: -32601,
        message: "Method not found".to_string(),
        data: None,
    };

    assert_eq!(parse_error.code, -32700);
    assert_eq!(invalid_request.code, -32600);
    assert_eq!(method_not_found.code, -32601);
}

/// Test complex nested parameters
#[test]
fn test_complex_nested_parameters() {
    let command = CdpCommand {
        id: 100,
        method: "Target.setAutoAttach".to_string(),
        params: Some(json!({
            "autoAttach": true,
            "waitForDebuggerOnStart": false,
            "flatten": true,
            "filter": [
                {"type": "page"},
                {"type": "worker"}
            ]
        })),
        session_id: None,
    };

    let params = command.params.unwrap();
    assert_eq!(params["autoAttach"], true);
    assert_eq!(params["waitForDebuggerOnStart"], false);
    assert!(params["filter"].is_array());
    assert_eq!(params["filter"][0]["type"], "page");
}

/// Test large result payload
#[test]
fn test_large_result_payload() {
    let large_data = "x".repeat(10000);

    let response = CdpResponse {
        id: 1,
        result: Some(json!({
            "data": large_data
        })),
        error: None,
    };

    let result = response.result.unwrap();
    assert_eq!(result["data"].as_str().unwrap().len(), 10000);
}

/// Test empty params
#[test]
fn test_empty_params() {
    let command = CdpCommand {
        id: 1,
        method: "Page.reload".to_string(),
        params: Some(json!({})),
        session_id: None,
    };

    assert!(command.params.is_some());
    let params = command.params.unwrap();
    assert!(params.is_object());
}
