use thalora::{WebSocketManager, WebSocketJsApi};
use tokio::time::Duration;

async fn test_multiple_connections() {
    let manager = WebSocketManager::new();
    
    // Create multiple connections
    let conn1 = manager.connect("ws://localhost:8080/room1", None).await.unwrap();
    let conn2 = manager.connect("ws://localhost:8080/room2", None).await.unwrap();
    let conn3 = manager.connect("ws://localhost:8080/room3", None).await.unwrap();
    
    // Verify all are active
    let active_connections = manager.get_active_connections();
    assert_eq!(active_connections.len(), 3);
    assert!(active_connections.contains(&conn1));
    assert!(active_connections.contains(&conn2));
    assert!(active_connections.contains(&conn3));
    
    // Send messages on different connections
    manager.send_message(&conn1, "Message from room1", false).await.unwrap();
    manager.send_message(&conn2, "Message from room2", false).await.unwrap();
    
    // Close one connection
    manager.close(&conn2, Some(1000), Some("Leaving room")).await.unwrap();
    
    // Verify active connections updated
    let active_connections = manager.get_active_connections();
    assert_eq!(active_connections.len(), 2);
    assert!(!active_connections.contains(&conn2));
    
    // Clean up
    manager.close(&conn1, None, None).await.unwrap();
    manager.close(&conn3, None, None).await.unwrap();
}
