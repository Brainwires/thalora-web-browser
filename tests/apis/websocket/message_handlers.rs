#[tokio::test]
async fn test_message_handlers() {
    let manager = WebSocketManager::new();
    
    // Add a message handler that responds to specific messages
    manager.add_message_handler(|message| {
        if message.data.contains("hello") {
            Ok(Some(WebSocketMessage {
                timestamp: tokio::time::Instant::now(),
                message_type: MessageType::Text,
                data: "Hello back!".to_string(),
                binary: false,
            }))
        } else {
            Ok(None)
        }
    });
    
    // Create isolated test server
    let server = create_isolated_test_server().await.unwrap();
    let url = get_server_url(&server);
    let connection_id = manager.connect(&url, None).await.unwrap();
    
    // Send message that should trigger handler
    manager.send_message(&connection_id, "hello world", false).await.unwrap();
    
    // Send message that shouldn't trigger handler  
    manager.send_message(&connection_id, "goodbye", false).await.unwrap();
    
    let (sent, _received) = manager.get_message_history(&connection_id).unwrap();
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[0].data, "hello world");
    assert_eq!(sent[1].data, "goodbye");
    
    manager.close(&connection_id, None, None).await.unwrap();

    // Shutdown the isolated server
    server.shutdown().await;
}
