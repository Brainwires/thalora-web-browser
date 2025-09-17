    #[tokio::test]
    async fn test_websocket_manager_creation() {
        let manager = WebSocketManager::new();
        let connections = manager.get_active_connections();
        assert!(connections.is_empty());
    }
