#[tokio::test]
async fn test_websocket_messaging() {
    let manager = WebSocketManager::new();
    
    let connection_id = manager.connect("wss://echo.websocket.org", None).await.unwrap();
    
    // Send various message types
    manager.send_message(&connection_id, "Text message", false).await.unwrap();
    manager.send_message(&connection_id, "Binary data", true).await.unwrap();
    
    // Simulate incoming messages
    manager.simulate_incoming_message(&connection_id, r#"{"type":"welcome","message":"Hello!"}"#, false).await.unwrap();
    manager.simulate_incoming_message(&connection_id, r#"{"type":"notification","count":5}"#, false).await.unwrap();
    
    // Check message history
    let (sent, received) = manager.get_message_history(&connection_id).unwrap();
    assert_eq!(sent.len(), 2);
    assert_eq!(received.len(), 2);
    
    // Verify message contents
    assert_eq!(sent[0].data, "Text message");
    assert!(!sent[0].binary);
    assert_eq!(sent[1].data, "Binary data");
    assert!(sent[1].binary);
    
    assert!(received[0].data.contains("welcome"));
    assert!(received[1].data.contains("notification"));
    
    manager.close(&connection_id, None, None).await.unwrap();
}
