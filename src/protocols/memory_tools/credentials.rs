use serde_json::Value;
use std::collections::HashMap;

use crate::features::ai_memory::AiMemoryHeap;
use crate::protocols::mcp::McpResponse;
use crate::protocols::security::{MAX_CONTENT_LENGTH, MAX_KEY_LENGTH, limit_input_length};

/// Handle storing credentials in AI memory
pub async fn handle_store_credentials(args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
    let key = match args.get("key").and_then(|v| v.as_str()) {
        Some(key) => key,
        None => {
            return McpResponse::error(-1, "Missing required parameter: key".to_string());
        }
    };

    // SECURITY: Validate key length
    if let Err(e) = limit_input_length(key, MAX_KEY_LENGTH, "Credential key") {
        return McpResponse::error(-1, format!("Input validation failed: {}", e));
    }

    let service = match args.get("service").and_then(|v| v.as_str()) {
        Some(service) => service,
        None => {
            return McpResponse::error(-1, "Missing required parameter: service".to_string());
        }
    };

    // SECURITY: Validate service length
    if let Err(e) = limit_input_length(service, MAX_KEY_LENGTH, "Service name") {
        return McpResponse::error(-1, format!("Input validation failed: {}", e));
    }

    let username = match args.get("username").and_then(|v| v.as_str()) {
        Some(username) => username,
        None => {
            return McpResponse::error(-1, "Missing required parameter: username".to_string());
        }
    };

    // SECURITY: Validate username length
    if let Err(e) = limit_input_length(username, MAX_KEY_LENGTH, "Username") {
        return McpResponse::error(-1, format!("Input validation failed: {}", e));
    }

    let password = match args.get("password").and_then(|v| v.as_str()) {
        Some(password) => password,
        None => {
            return McpResponse::error(-1, "Missing required parameter: password".to_string());
        }
    };

    // SECURITY: Validate password length (using CONTENT_LENGTH for passwords as they can be long)
    if let Err(e) = limit_input_length(password, MAX_CONTENT_LENGTH, "Password") {
        return McpResponse::error(-1, format!("Input validation failed: {}", e));
    }

    let additional_data: HashMap<String, String> = args
        .get("additional_data")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_else(HashMap::new);

    match ai_memory.store_credentials(key, service, username, password, additional_data) {
        Ok(_) => McpResponse::success(serde_json::json!({
            "type": "text",
            "text": format!("Credentials for '{}' stored securely in AI memory heap", service)
        })),
        Err(e) => McpResponse::error(-1, format!("Failed to store credentials: {}", e)),
    }
}

/// Handle retrieving credentials from AI memory
pub async fn handle_retrieve_credentials(args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
    let key = match args.get("key").and_then(|v| v.as_str()) {
        Some(key) => key,
        None => {
            return McpResponse::error(-1, "Missing required parameter: key".to_string());
        }
    };

    // SECURITY: Validate key length
    if let Err(e) = limit_input_length(key, MAX_KEY_LENGTH, "Credential key") {
        return McpResponse::error(-1, format!("Input validation failed: {}", e));
    }

    match ai_memory.get_credentials(key) {
        Ok(Some((service, username, password, additional_data))) => {
            let response_json = serde_json::json!({
                "service": service,
                "username": username,
                "password": password,
                "additional_data": additional_data,
                "retrieved_from": "ai_memory_heap"
            });

            McpResponse::success(serde_json::json!({
                "type": "text",
                "text": serde_json::to_string_pretty(&response_json).unwrap_or_default()
            }))
        }
        Ok(None) => McpResponse::success(serde_json::json!({
            "type": "text",
            "text": format!("No credentials found for key: {}", key)
        })),
        Err(e) => McpResponse::error(-1, format!("Failed to retrieve credentials: {}", e)),
    }
}
