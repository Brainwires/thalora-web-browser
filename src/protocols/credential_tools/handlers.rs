use serde_json::Value;
use std::collections::HashMap;

use crate::features::credential_vault::{CredentialEntry, CredentialVault};
use crate::protocols::mcp::McpResponse;
use crate::protocols::security::{MAX_CONTENT_LENGTH, MAX_KEY_LENGTH, limit_input_length};

/// `vault_store_credential`: encrypt and store a credential.
pub fn handle_store_credential(args: &Value, vault: &mut CredentialVault) -> McpResponse {
    let key = match credential_key(args) {
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

    match vault.store_credentials(key, service, username, password, additional_data) {
        Ok(_) => McpResponse::success(serde_json::json!({
            "type": "text",
            "text": format!("Credentials for '{}' stored in the encrypted vault", service)
        })),
        Err(e) => McpResponse::error(-1, format!("Failed to store credentials: {}", e)),
    }
}

/// `vault_list_credentials`: what is stored, never the secrets. With a
/// `service`/`key`, describes just that credential.
pub fn handle_list_credentials(args: &Value, vault: &CredentialVault) -> McpResponse {
    let entries: Vec<Value> = match credential_key(args) {
        Some(key) => {
            if let Err(e) = limit_input_length(key, MAX_KEY_LENGTH, "Credential key") {
                return McpResponse::error(-1, format!("Input validation failed: {}", e));
            }
            vault
                .entry(key)
                .map(|c| describe(key, c))
                .into_iter()
                .collect()
        }
        None => vault
            .list()
            .into_iter()
            .filter_map(|(key, _, _)| vault.entry(key).map(|c| describe(key, c)))
            .collect(),
    };
    McpResponse::success(serde_json::json!({
        "type": "text",
        "text": serde_json::to_string_pretty(&serde_json::json!({ "credentials": entries }))
            .unwrap_or_default()
    }))
}

/// `vault_delete_credential`: remove a stored credential.
pub fn handle_delete_credential(args: &Value, vault: &mut CredentialVault) -> McpResponse {
    let Some(key) = credential_key(args) else {
        return McpResponse::error(
            -1,
            "Missing required parameter: service (or key)".to_string(),
        );
    };
    match vault.remove(key) {
        Ok(true) => McpResponse::success(serde_json::json!({
            "type": "text",
            "text": format!("Deleted credential '{}'", key)
        })),
        Ok(false) => McpResponse::error(-32602, format!("No stored credential named '{}'", key)),
        Err(e) => McpResponse::error(-1, format!("Failed to delete credential: {}", e)),
    }
}

/// Public description of a stored credential. Secrets are never included.
fn describe(key: &str, c: &CredentialEntry) -> Value {
    let mut data_keys: Vec<&String> = c.additional_data.keys().collect();
    data_keys.sort();
    serde_json::json!({
        "credential_id": key,
        "service": c.service,
        "username": c.username,
        "origin": c.additional_data.get("origin"),
        "additional_data_keys": data_keys,
        "last_used": c.last_used,
    })
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
