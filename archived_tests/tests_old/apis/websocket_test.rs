// Tests for src/apis/websocket.rs
#[cfg(test)]
mod websocket_tests {
    use thalora::apis::websocket::*;

    #[tokio::test]
    async fn test_websocket_manager_creation() {
        let manager = WebSocketManager::new();
        let connections = manager.get_active_connections();
        assert!(connections.is_empty());
    }

    #[tokio::test]
    async fn test_websocket_connection() {
        let manager = WebSocketManager::new();
        // Test connection simulation since we can't make real connections in tests
        let result = manager.simulate_incoming_message("test", "hello", false).await;
        assert!(result.is_err()); // Should fail with no connection
    }
}