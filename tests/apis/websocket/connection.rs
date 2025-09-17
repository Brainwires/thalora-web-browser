#[tokio::test]
async fn test_websocket_connection() {
    let manager = WebSocketManager::new();
    
    // Test connection establishment using public echo server
    let connection_id = manager.connect("wss://echo.websocket.org", None).await.unwrap();
    assert!(!connection_id.is_empty());
    
    // Verify connection state
    let state = manager.get_connection_state(&connection_id).unwrap();
    assert!(matches!(state, ConnectionState::Open));
    
    // Test message sending (this might be where it hangs)
    tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        manager.send_message(&connection_id, "Hello WebSocket", false)
    ).await.expect("send_message timed out").unwrap();
    
    // Test connection closing
    manager.close(&connection_id, Some(1000), Some("Normal closure".to_string())).await.unwrap();
    
    let state = manager.get_connection_state(&connection_id);
    assert!(state.is_err()); // Connection should be removed
}
