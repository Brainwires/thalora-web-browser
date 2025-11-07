use serde_json::Value;

use crate::protocols::mcp::McpResponse;
use crate::features::ai_memory::AiMemoryHeap;

/// Handle starting a new session in AI memory
pub async fn handle_start_session(args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
    let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
        Some(session_id) => session_id,
        None => {
            return McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "Missing required parameter: session_id"
                })],
                is_error: true,
            };
        }
    };

    let description = match args.get("description").and_then(|v| v.as_str()) {
        Some(description) => description,
        None => {
            return McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "Missing required parameter: description"
                })],
                is_error: true,
            };
        }
    };

    let objectives = args.get("objectives")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(Vec::new);

    match ai_memory.start_session(session_id, description, objectives) {
        Ok(_) => McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("Session '{}' started successfully in AI memory heap", session_id)
            })],
            is_error: false,
        },
        Err(e) => McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("Failed to start session: {}", e)
            })],
            is_error: true,
        }
    }
}

/// Handle updating session progress in AI memory
pub async fn handle_update_session(args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
    let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
        Some(session_id) => session_id,
        None => {
            return McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "Missing required parameter: session_id"
                })],
                is_error: true,
            };
        }
    };

    if let Some(progress_key) = args.get("progress_key").and_then(|v| v.as_str()) {
        if let Some(progress_value) = args.get("progress_value") {
            match ai_memory.update_session_progress(session_id, progress_key, progress_value.clone()) {
                Ok(_) => McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": format!("Session '{}' progress updated: {} = {:?}", session_id, progress_key, progress_value)
                    })],
                    is_error: false,
                },
                Err(e) => McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": format!("Failed to update session progress: {}", e)
                    })],
                    is_error: true,
                }
            }
        } else {
            McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "Missing required parameter: progress_value"
                })],
                is_error: true,
            }
        }
    } else {
        McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": "No progress_key provided - no updates made"
            })],
            is_error: false,
        }
    }
}
