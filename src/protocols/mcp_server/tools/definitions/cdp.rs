use serde_json::Value;

/// JavaScript evaluation tool. The `cdp_` name is historical: the mock CDP
/// domains were removed, and a real CDP endpoint is planned (see
/// docs/ARCHITECTURE.md).
pub(crate) fn get_cdp_tool_definitions() -> Vec<Value> {
    vec![serde_json::json!({
        "name": "cdp_runtime_evaluate",
        "description": "Evaluate JavaScript in a browser session's page (V8) and return the completion value as JSON: undefined is null, a DOM node is its outerHTML, a value JSON cannot express is its string form. A returned promise, or a top-level `await`, is awaited for up to 30 s (statements using `await` return their value with `return`). Fails on an uncaught exception, when the script runs longer than the script timeout (10 s by default), and on a static page (JavaScript disabled).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "JavaScript to evaluate (a script; the value of its last expression statement is the result)"
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
