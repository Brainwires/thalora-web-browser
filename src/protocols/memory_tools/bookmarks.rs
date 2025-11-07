use serde_json::Value;

use crate::protocols::mcp::McpResponse;
use crate::features::ai_memory::AiMemoryHeap;

/// Handle storing a bookmark in AI memory
pub async fn handle_store_bookmark(args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
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

    let url = match args.get("url").and_then(|v| v.as_str()) {
        Some(url) => url,
        None => {
            return McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "Missing required parameter: url"
                })],
                is_error: true,
            };
        }
    };

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

    let description = args.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let content_preview = args.get("content_preview").and_then(|v| v.as_str()).unwrap_or("");

    let tags = args.get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(Vec::new);

    match ai_memory.store_bookmark(key, url, title, description, content_preview, tags) {
        Ok(_) => McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("Bookmark '{}' stored successfully in AI memory heap", title)
            })],
            is_error: false,
        },
        Err(e) => McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("Failed to store bookmark: {}", e)
            })],
            is_error: true,
        }
    }
}
