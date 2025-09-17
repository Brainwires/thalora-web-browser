use thalora::{WebSocketManager, WebSocketJsApi};
use tokio::time::Duration;

async fn test_websocket_ping_pong() {
    let manager = WebSocketManager::new();
    
    let connection_id = manager.connect("ws://localhost:8080/ping", None).await.unwrap();
    
    // Test ping/pong mechanism
    manager.ping(&connection_id, Some("ping data")).await.unwrap();
    
    let (sent, received) = manager.get_message_history(&connection_id).unwrap();
    
    // Should have ping in sent messages
    let ping_message = sent.iter().find(|msg| matches!(msg.message_type, thalora::websocket::MessageType::Ping));
    assert!(ping_message.is_some());
    assert_eq!(ping_message.unwrap().data, "ping data");
    
    // Should have pong in received messages
    let pong_message = received.iter().find(|msg| matches!(msg.message_type, thalora::websocket::MessageType::Pong));
    assert!(pong_message.is_some());
    assert_eq!(pong_message.unwrap().data, "ping data");
    
    manager.close(&connection_id, None, None).await.unwrap();
}
