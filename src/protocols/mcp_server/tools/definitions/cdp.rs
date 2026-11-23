use serde_json::Value;

/// JavaScript evaluation tool. The `cdp_` name is historical: the mock CDP
/// domains were removed, and a real CDP endpoint is planned (see
/// docs/ARCHITECTURE.md).
pub(crate) fn get_cdp_tool_definitions() -> Vec<Value> {
    vec![serde_json::json!({
        "name": "cdp_runtime_evaluate",
        "description": "Evaluate a JavaScript expression in a browser session's page and return the result",
        "inputSchema": {
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "JavaScript expression to evaluate"
                },
                "session_id": {
                    "type": "string",
                    "description": "Browser session to evaluate in (default: \"default\")"
                }
            },
            "required": ["expression"]
        }
    })]
}
