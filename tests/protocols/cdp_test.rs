// Tests for src/protocols/cdp.rs
#[cfg(test)]
mod cdp_tests {
    use synaptic::protocols::cdp::*;

    #[test]
    fn test_cdp_server_creation() {
        let server = CdpServer::new("http://localhost".to_string(), 9222);
        assert_eq!(server.websocket_url, "http://localhost");
        assert_eq!(server.port, 9222);
    }

    #[test]
    fn test_cdp_command_creation() {
        let command = CdpCommand {
            id: 1,
            method: "Runtime.evaluate".to_string(),
            params: Some(serde_json::json!({"expression": "console.log('test')"})),
            session_id: None,
        };

        assert_eq!(command.method, "Runtime.evaluate");
        assert_eq!(command.id, 1);
    }
}