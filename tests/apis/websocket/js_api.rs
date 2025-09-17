#[tokio::test]
async fn test_websocket_js_api() {
    let manager = WebSocketManager::new();
    let js_api = WebSocketJsApi::new(manager);
    
    // Test connection creation
    let connection_id = js_api.create_test_connection("ws://localhost:8080/jsapi").await.unwrap();
    assert!(!connection_id.is_empty());
    
    // Test message exchange simulation
    js_api.simulate_message_exchange(&connection_id).await.unwrap();
    
    // Verify the exchange worked
    let (sent, received) = js_api.manager.get_message_history(&connection_id).unwrap();
    assert!(sent.len() > 0);
    assert!(received.len() > 0);
    
    // Should have join message in sent
    let join_message = sent.iter().find(|msg| msg.data.contains("join"));
    assert!(join_message.is_some());
    
    // Should have responses in received
    let joined_response = received.iter().find(|msg| msg.data.contains("joined"));
    assert!(joined_response.is_some());
}
