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
    // Create isolated test server
    let server = create_isolated_test_server().await.unwrap();
    let url = get_server_url(&server);
    let connection_id = manager.connect(&url, None).await.unwrap();
    manager.close(&connection_id, None, None).await.unwrap();

    // Should fail to send message to closed connection
    let result = manager.send_message(&connection_id, "test", false).await;
    assert!(result.is_err());

    // Shutdown the isolated server
    server.shutdown().await;
}
