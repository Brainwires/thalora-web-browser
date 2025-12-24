use serde_json::Value;
use chrono::Utc;

use crate::protocols::mcp::McpResponse;
use crate::features::ai_memory::{AiMemoryHeap, ResearchEntry};
use crate::protocols::security::{limit_input_length, MAX_KEY_LENGTH, MAX_CONTENT_LENGTH};

/// Handle storing research data in AI memory
pub async fn handle_store_research(args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
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

    // SECURITY: Validate key length to prevent DoS attacks
    if let Err(e) = limit_input_length(key, MAX_KEY_LENGTH, "Research key") {
        return McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("Input validation failed: {}", e)
            })],
            is_error: true,
        };
    }

    let topic = match args.get("topic").and_then(|v| v.as_str()) {
        Some(topic) => topic,
        None => {
            return McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "Missing required parameter: topic"
                })],
                is_error: true,
            };
        }
    };

    // SECURITY: Validate topic length
    if let Err(e) = limit_input_length(topic, MAX_CONTENT_LENGTH, "Topic") {
        return McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("Input validation failed: {}", e)
            })],
            is_error: true,
        };
    }

    let summary = match args.get("summary").and_then(|v| v.as_str()) {
        Some(summary) => summary,
        None => {
            return McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "Missing required parameter: summary"
                })],
                is_error: true,
            };
        }
    };

    // SECURITY: Validate summary length
    if let Err(e) = limit_input_length(summary, MAX_CONTENT_LENGTH, "Summary") {
        return McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("Input validation failed: {}", e)
            })],
            is_error: true,
        };
    }

    let findings = args.get("findings")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(Vec::new);

    let sources = args.get("sources")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(Vec::new);

    let tags = args.get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(Vec::new);

    let confidence_score = args.get("confidence_score")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);

    let related_topics = args.get("related_topics")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(Vec::new);

    let research_entry = ResearchEntry {
        topic: topic.to_string(),
        summary: summary.to_string(),
        findings,
        sources,
        tags,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        confidence_score,
        related_topics,
    };

    match ai_memory.store_research(key, research_entry) {
        Ok(_) => McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("Research entry '{}' stored successfully in AI memory heap", key)
            })],
            is_error: false,
        },
        Err(e) => McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("Failed to store research entry: {}", e)
            })],
            is_error: true,
        }
    }
}
