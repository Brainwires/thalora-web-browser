// Tests for src/protocols/mcp.rs
#[cfg(test)]
mod mcp_tests {
    use thalora::protocols::mcp::*;
    use serde_json::json;

    #[test]
    fn test_mcp_request_parsing() {
        let json_str = r#"{"method": "tools/call", "id": 1, "params": {"name": "scrape_url", "arguments": {"url": "https://example.com"}}}"#;
        let request: Result<McpRequest, _> = serde_json::from_str(json_str);
        assert!(request.is_ok());

        let req = request.unwrap();
        assert_eq!(req.method, "tools/call");
        assert_eq!(req.id, Some(json!(1)));
    }

    #[test]
    fn test_mcp_response_creation() {
        let response = McpResponse {
            id: Some(json!(1)),
            result: Some(json!({"success": true})),
            error: None,
        };

        let serialized = serde_json::to_string(&response);
        assert!(serialized.is_ok());
    }
}