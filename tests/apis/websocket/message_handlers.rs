use thalora::{WebSocketManager, WebSocketJsApi};
use tokio::time::Duration;

async fn test_message_handlers() {
    let manager = WebSocketManager::new();
    
    // Add a message handler that responds to specific messages
    manager.add_message_handler(|message| {
        if message.data.contains("hello") {
            Ok(Some(thalora::websocket::WebSocketMessage {
                timestamp: tokio::time::Instant::now(),
                message_type: thalora::websocket::MessageType::Text,
                data: "Hello back!".to_string(),
                binary: false,
            }))
        } else {
            Ok(None)
        }
    });
    
    let connection_id = manager.connect("ws://localhost:8080/handled", None).await.unwrap();
    
    // Send message that should trigger handler
    manager.send_message(&connection_id, "hello world", false).await.unwrap();
    
    // Send message that shouldn't trigger handler  
    manager.send_message(&connection_id, "goodbye", false).await.unwrap();
    
    let (sent, _received) = manager.get_message_history(&connection_id).unwrap();
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[0].data, "hello world");
    assert_eq!(sent[1].data, "goodbye");
    
    manager.close(&connection_id, None, None).await.unwrap();
}
