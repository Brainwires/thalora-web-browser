use thalora::mcp::{McpRequest, McpResponse, ToolCall};
use serde_json::{json, Value};

#[test]
fn test_mcp_request_deserialization() {
    let initialize_json = r#"
    {
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {}
        }
    }
    "#;

    let request: McpRequest = serde_json::from_str(initialize_json).unwrap();
    match request {
        McpRequest::Initialize { params } => {
            assert!(params.is_object());
        }
        _ => panic!("Expected Initialize request"),
    }

    let list_tools_json = r#"
    {
        "method": "tools/list"
    }
    "#;

    let request: McpRequest = serde_json::from_str(list_tools_json).unwrap();
    match request {
        McpRequest::ListTools => {
            // Success
        }
        _ => panic!("Expected ListTools request"),
    }

    let call_tool_json = r#"
    {
        "method": "tools/call",
        "params": {
            "name": "scrape_url",
            "arguments": {
                "url": "https://example.com",
                "wait_for_js": true
            }
        }
    }
    "#;

    let request: McpRequest = serde_json::from_str(call_tool_json).unwrap();
    match request {
        McpRequest::CallTool { params } => {
            assert_eq!(params.name, "scrape_url");
            assert_eq!(params.arguments["url"], "https://example.com");
            assert_eq!(params.arguments["wait_for_js"], true);
        }
        _ => panic!("Expected CallTool request"),
    }
}

#[test]
fn test_mcp_response_serialization() {
    let initialize_response = McpResponse::Initialize {
        protocol_version: "2024-11-05".to_string(),
        capabilities: json!({"tools": {}}),
        server_info: json!({
            "name": "brainwires-scraper",
            "version": "0.1.0"
        }),
    };

    let json_str = serde_json::to_string(&initialize_response).unwrap();
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    
    assert_eq!(parsed["protocolVersion"], "2024-11-05");
    assert_eq!(parsed["serverInfo"]["name"], "brainwires-scraper");

    let list_tools_response = McpResponse::ListTools {
        tools: vec![
            json!({
                "name": "scrape_url",
                "description": "Scrape content from a URL",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "url": {"type": "string"}
                    },
                    "required": ["url"]
                }
            })
        ],
    };

    let json_str = serde_json::to_string(&list_tools_response).unwrap();
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    
    assert!(parsed["tools"].is_array());
    assert_eq!(parsed["tools"][0]["name"], "scrape_url");

    let tool_result_response = McpResponse::ToolResult {
        content: vec![json!({
            "type": "text",
            "text": "Scraped content"
        })],
        is_error: false,
    };

    let json_str = serde_json::to_string(&tool_result_response).unwrap();
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    
    assert!(parsed["content"].is_array());
    assert_eq!(parsed["isError"], false);
    assert_eq!(parsed["content"][0]["text"], "Scraped content");

    let error_response = McpResponse::Error {
        error: "Something went wrong".to_string(),
    };

    let json_str = serde_json::to_string(&error_response).unwrap();
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    
    assert_eq!(parsed["error"], "Something went wrong");
}

#[test]
fn test_tool_call_structure() {
    let tool_call = ToolCall {
        name: "extract_data".to_string(),
        arguments: json!({
            "html": "<html><body><h1>Test</h1></body></html>",
            "selectors": {
                "title": "h1"
            }
        }),
    };

    assert_eq!(tool_call.name, "extract_data");
    assert!(tool_call.arguments["html"].is_string());
    assert!(tool_call.arguments["selectors"].is_object());

    let json_str = serde_json::to_string(&tool_call).unwrap();
    let parsed: ToolCall = serde_json::from_str(&json_str).unwrap();
    
    assert_eq!(parsed.name, tool_call.name);
    assert_eq!(parsed.arguments, tool_call.arguments);
}

#[test]
fn test_invalid_mcp_requests() {
    // Invalid method
    let invalid_json = r#"
    {
        "method": "unknown/method"
    }
    "#;

    let result = serde_json::from_str::<McpRequest>(invalid_json);
    assert!(result.is_err());

    // Missing required field
    let invalid_json = r#"
    {
        "method": "tools/call"
    }
    "#;

    let result = serde_json::from_str::<McpRequest>(invalid_json);
    assert!(result.is_err());

    // Invalid JSON
    let invalid_json = r#"
    {
        "method": "tools/call",
        "params": {
            "name": "test"
            // Missing comma and closing brace
        }
    "#;

    let result = serde_json::from_str::<McpRequest>(invalid_json);
    assert!(result.is_err());
}

#[test] 
fn test_scrape_url_arguments() {
    let args = json!({
        "url": "https://example.com",
        "wait_for_js": true,
        "selector": ".content",
        "extract_links": false,
        "extract_images": true
    });

    // Test argument extraction
    assert_eq!(args["url"].as_str().unwrap(), "https://example.com");
    assert_eq!(args["wait_for_js"].as_bool().unwrap(), true);
    assert_eq!(args["selector"].as_str().unwrap(), ".content");
    assert_eq!(args["extract_links"].as_bool().unwrap(), false);
    assert_eq!(args["extract_images"].as_bool().unwrap(), true);
}

#[test]
fn test_extract_data_arguments() {
    let args = json!({
        "html": "<div><h1>Title</h1><p>Content</p></div>",
        "selectors": {
            "title": "h1",
            "content": "p"
        }
    });

    assert!(args["html"].is_string());
    assert!(args["selectors"].is_object());
    
    let selectors = args["selectors"].as_object().unwrap();
    assert_eq!(selectors["title"].as_str().unwrap(), "h1");
    assert_eq!(selectors["content"].as_str().unwrap(), "p");
}