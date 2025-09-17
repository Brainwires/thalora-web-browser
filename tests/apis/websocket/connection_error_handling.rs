#[tokio::test]
async fn test_connection_error_handling() {
    let manager = WebSocketManager::new();
    
    // Test sending message to non-existent connection
    let result = manager.send_message("invalid_id", "test", false).await;
    assert!(result.is_err());
    
    // Test getting state of non-existent connection
    let result = manager.get_connection_state("invalid_id");
    assert!(result.is_err());
    
    // Test closing non-existent connection
    let result = manager.close("invalid_id", None, None).await;
    assert!(result.is_err());
    
    // Create connection then test operations on closed connection
    let connection_id = manager.connect("ws://localhost:8080/test", None).await.unwrap();
    manager.close(&connection_id, None, None).await.unwrap();
    
    // Should fail to send message to closed connection
    let result = manager.send_message(&connection_id, "test", false).await;
    assert!(result.is_err());
}
