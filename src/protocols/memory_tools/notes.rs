use serde_json::Value;

use crate::protocols::mcp::McpResponse;
use crate::features::ai_memory::{AiMemoryHeap, NotePriority};
use crate::protocols::security::{limit_input_length, MAX_KEY_LENGTH, MAX_CONTENT_LENGTH};

/// Handle storing a note in AI memory
pub async fn handle_store_note(args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
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

    // SECURITY: Validate key length
    if let Err(e) = limit_input_length(key, MAX_KEY_LENGTH, "Note key") {
        return McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("Input validation failed: {}", e)
            })],
            is_error: true,
        };
    }

    let title = match args.get("title").and_then(|v| v.as_str()) {
        Some(title) => title,
        None => {
            return McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "Missing required parameter: title"
                })],
                is_error: true,
            };
        }
    };

    // SECURITY: Validate title length
    if let Err(e) = limit_input_length(title, MAX_KEY_LENGTH, "Note title") {
        return McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("Input validation failed: {}", e)
            })],
            is_error: true,
        };
    }

    let content = match args.get("content").and_then(|v| v.as_str()) {
        Some(content) => content,
        None => {
            return McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "Missing required parameter: content"
                })],
                is_error: true,
            };
        }
    };

    // SECURITY: Validate content length
    if let Err(e) = limit_input_length(content, MAX_CONTENT_LENGTH, "Note content") {
        return McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("Input validation failed: {}", e)
            })],
            is_error: true,
        };
    }

    let category = args.get("category").and_then(|v| v.as_str()).unwrap_or("general");

    let tags = args.get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(Vec::new);

    let priority = args.get("priority")
        .and_then(|v| v.as_str())
        .and_then(|s| match s {
            "Low" => Some(NotePriority::Low),
            "Medium" => Some(NotePriority::Medium),
            "High" => Some(NotePriority::High),
            "Critical" => Some(NotePriority::Critical),
            _ => None,
        })
        .unwrap_or(NotePriority::Medium);

    match ai_memory.store_note(key, title, content, category, tags, priority) {
        Ok(_) => McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("Note '{}' stored successfully in AI memory heap", title)
            })],
            is_error: false,
        },
        Err(e) => McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("Failed to store note: {}", e)
            })],
            is_error: true,
        }
    }
}
