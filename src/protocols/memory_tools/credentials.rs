use serde_json::Value;
use std::collections::HashMap;

use crate::protocols::mcp::McpResponse;
use crate::features::ai_memory::AiMemoryHeap;

/// Handle storing credentials in AI memory
pub async fn handle_store_credentials(args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
    let key = match args.get("key").and_then(|v| v.as_str()) {
        Some(key) => key,
        None => {
            return McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "Missing required parameter: key"
                })],
                is_error: true,
            };
        }
    };

    let service = match args.get("service").and_then(|v| v.as_str()) {
        Some(service) => service,
        None => {
            return McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "Missing required parameter: service"
                })],
                is_error: true,
            };
        }
    };

    let username = match args.get("username").and_then(|v| v.as_str()) {
        Some(username) => username,
        None => {
            return McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "Missing required parameter: username"
                })],
                is_error: true,
            };
        }
    };

    let password = match args.get("password").and_then(|v| v.as_str()) {
        Some(password) => password,
        None => {
            return McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "Missing required parameter: password"
                })],
                is_error: true,
            };
        }
    };

    let additional_data: HashMap<String, String> = args.get("additional_data")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_else(HashMap::new);

    match ai_memory.store_credentials(key, service, username, password, additional_data) {
        Ok(_) => McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("Credentials for '{}' stored securely in AI memory heap", service)
            })],
            is_error: false,
        },
        Err(e) => McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("Failed to store credentials: {}", e)
            })],
            is_error: true,
        }
    }
}

/// Handle retrieving credentials from AI memory
pub async fn handle_retrieve_credentials(args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
    let key = match args.get("key").and_then(|v| v.as_str()) {
        Some(key) => key,
        None => {
            return McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "Missing required parameter: key"
                })],
                is_error: true,
            };
        }
    };

    match ai_memory.get_credentials(key) {
        Ok(Some((service, username, password, additional_data))) => {
            let response_json = serde_json::json!({
                "service": service,
                "username": username,
                "password": password,
                "additional_data": additional_data,
                "retrieved_from": "ai_memory_heap"
            });

            McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": serde_json::to_string_pretty(&response_json).unwrap_or_default()
                })],
                is_error: false,
            }
        },
        Ok(None) => McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("No credentials found for key: {}", key)
            })],
            is_error: false,
        },
        Err(e) => McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("Failed to retrieve credentials: {}", e)
            })],
            is_error: true,
        }
    }
}
