use serde_json::Value;
use std::collections::HashMap;

use crate::features::ai_memory::AiMemoryHeap;
use crate::protocols::mcp::McpResponse;
use crate::protocols::security::{MAX_CONTENT_LENGTH, MAX_KEY_LENGTH, limit_input_length};

/// Handle storing credentials in AI memory
pub async fn handle_store_credentials(args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
    let key = match credential_key(&args) {
        Some(key) => key,
        None => {
            return McpResponse::error(
                -1,
                "Missing required parameter: service (or key)".to_string(),
            );
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
        .unwrap_or_default();

    // Bind the credential to a site so browser_fill_credential only fills
    // it into pages of that origin.
    let mut additional_data = additional_data;
    if let Some(origin) = args.get("origin").and_then(|v| v.as_str()) {
        match normalize_origin(origin) {
            Some(origin) => {
                additional_data.insert("origin".to_string(), origin);
            }
            None => {
                return McpResponse::error(
                    -32602,
                    format!(
                        "Invalid origin '{}': expected e.g. https://example.com",
                        origin
                    ),
                );
            }
        }
    }

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
    let key = match credential_key(&args) {
        Some(key) => key,
        None => {
            return McpResponse::error(
                -1,
                "Missing required parameter: service (or key)".to_string(),
            );
        }
    };

    // SECURITY: Validate key length
    if let Err(e) = limit_input_length(key, MAX_KEY_LENGTH, "Credential key") {
        return McpResponse::error(-1, format!("Input validation failed: {}", e));
    }

    match ai_memory.get_credentials(key) {
        Ok(Some((service, username, password, additional_data))) => {
            // SECURITY: secrets are not returned into the model's context by
            // default — anything in the context can leak via prompt injection,
            // logs or transcripts. THALORA_EXPOSE_PASSWORDS=true restores the
            // old behaviour for callers that genuinely need the raw secret.
            let response_json = if expose_secrets() {
                serde_json::json!({
                    "service": service,
                    "username": username,
                    "password": password,
                    "additional_data": additional_data,
                    "retrieved_from": "ai_memory_heap"
                })
            } else {
                let mut data_keys: Vec<&String> = additional_data.keys().collect();
                data_keys.sort();
                serde_json::json!({
                    "credential_id": key,
                    "service": service,
                    "username": username,
                    "has_password": !password.is_empty(),
                    "additional_data_keys": data_keys,
                    "secrets_redacted": true,
                    "retrieved_from": "ai_memory_heap"
                })
            };

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

/// Credentials are keyed by an explicit `key`, falling back to `service`
/// (the parameter the tool schemas advertise).
fn credential_key(args: &Value) -> Option<&str> {
    args.get("key")
        .and_then(|v| v.as_str())
        .or_else(|| args.get("service").and_then(|v| v.as_str()))
}

/// `https://example.com/login` -> `https://example.com` (None if not a URL
/// with a host).
pub(crate) fn normalize_origin(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    parsed.host_str()?;
    Some(parsed.origin().ascii_serialization())
}

/// Whether raw secrets may be returned to the caller (opt-in, unsafe).
fn expose_secrets() -> bool {
    std::env::var("THALORA_EXPOSE_PASSWORDS")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origins_are_normalized() {
        assert_eq!(
            normalize_origin("https://Example.com:443/login?x=1").as_deref(),
            Some("https://example.com")
        );
        assert_eq!(
            normalize_origin("http://localhost:8080/").as_deref(),
            Some("http://localhost:8080")
        );
        assert_eq!(normalize_origin("not a url"), None);
        assert_eq!(normalize_origin("data:text/plain,hi"), None);
    }

    #[test]
    fn credential_key_falls_back_to_service() {
        let args = serde_json::json!({"service": "github"});
        assert_eq!(credential_key(&args), Some("github"));
        let args = serde_json::json!({"service": "github", "key": "gh_work"});
        assert_eq!(credential_key(&args), Some("gh_work"));
        assert_eq!(credential_key(&serde_json::json!({})), None);
    }
}
