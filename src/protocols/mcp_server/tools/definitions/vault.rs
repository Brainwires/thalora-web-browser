use serde_json::Value;

/// Credential vault tools. No tool here returns a stored secret.
pub(crate) fn get_vault_tool_definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "vault_store_credential",
            "description": "Store a site credential in the encrypted vault (requires THALORA_MASTER_PASSWORD). Secrets are never returned by any tool; use browser_fill_credential to fill them into a page of the stored origin.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "service": {
                        "type": "string",
                        "description": "Service name or identifier for the credentials"
                    },
                    "key": {
                        "type": "string",
                        "description": "Storage key (optional; defaults to service)"
                    },
                    "origin": {
                        "type": "string",
                        "description": "Site the credential belongs to, e.g. https://github.com. Required for browser_fill_credential, which only fills into pages of this origin."
                    },
                    "username": {
                        "type": "string",
                        "description": "Username or identifier"
                    },
                    "password": {
                        "type": "string",
                        "description": "Password or secret value"
                    },
                    "additional_data": {
                        "type": "object",
                        "description": "Additional credential data (API keys, tokens, etc.)"
                    },
                    "tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Optional tags for categorization"
                    }
                },
                "required": ["service", "username", "password"]
            }
        }),
        serde_json::json!({
            "name": "vault_list_credentials",
            "description": "List stored credentials (service, username, origin, last use). Secret values are never returned. Pass `service` (or `key`) to describe a single credential.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "service": {
                        "type": "string",
                        "description": "Only describe this credential (service name or storage key)"
                    },
                    "key": {
                        "type": "string",
                        "description": "Storage key (alternative to service)"
                    }
                }
            }
        }),
        serde_json::json!({
            "name": "vault_delete_credential",
            "description": "Delete a stored credential from the vault.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "service": {
                        "type": "string",
                        "description": "Credential to delete (service name or storage key)"
                    },
                    "key": {
                        "type": "string",
                        "description": "Storage key (alternative to service)"
                    }
                },
                "required": ["service"]
            }
        }),
        serde_json::json!({
            "name": "browser_fill_credential",
            "description": "Fill a stored credential into the current page without revealing it: the password (and optionally the username) is typed into the given fields server-side and never returned. Only works on pages whose origin matches the origin the credential was stored with.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "service": {
                        "type": "string",
                        "description": "Credential to use (service name or storage key)"
                    },
                    "password_ref": {
                        "type": "string",
                        "description": "Snapshot ref of the password field"
                    },
                    "password_selector": {
                        "type": "string",
                        "description": "CSS selector of the password field (alternative to password_ref)"
                    },
                    "username_ref": {
                        "type": "string",
                        "description": "Snapshot ref of the username field (optional)"
                    },
                    "username_selector": {
                        "type": "string",
                        "description": "CSS selector of the username field (optional)"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID (optional)"
                    }
                },
                "required": ["service"]
            }
        }),
    ]
}
