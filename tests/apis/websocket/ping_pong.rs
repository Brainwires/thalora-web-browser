#[tokio::test]
async fn test_websocket_ping_pong() {
    let manager = WebSocketManager::new();
    
    let connection_id = manager.connect("wss://echo.websocket.org", None).await.unwrap();
    
    // Test ping/pong mechanism
    manager.ping(&connection_id, Some("ping data")).await.unwrap();

    // Wait for pong response (WebSocket ping/pong is asynchronous)
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let (sent, received) = manager.get_message_history(&connection_id).unwrap();
    
    // Should have ping in sent messages
    let ping_message = sent.iter().find(|msg| matches!(msg.message_type, MessageType::Ping));
    assert!(ping_message.is_some());
    assert_eq!(ping_message.unwrap().data, "ping data");
    
    // Should have pong in received messages
    let pong_message = received.iter().find(|msg| matches!(msg.message_type, MessageType::Pong));
    assert!(pong_message.is_some());
    assert_eq!(pong_message.unwrap().data, "ping data");
    
    manager.close(&connection_id, None, None).await.unwrap();
}
