#[tokio::test]
async fn test_realtime_events_simulation() {
    let manager = WebSocketManager::new();
    
    let connection_id = manager.connect("ws://localhost:8080/events", None).await.unwrap();
    
    // Simulate various real-time events
    let events = vec!["heartbeat", "user_joined", "message", "notification", "status_update"];
    manager.simulate_realtime_events(&connection_id, events).await.unwrap();
    
    let (_sent, received) = manager.get_message_history(&connection_id).unwrap();
    assert_eq!(received.len(), 5);
    
    // Verify event types were properly simulated
    let event_types: Vec<String> = received.iter()
        .filter_map(|msg| {
            serde_json::from_str::<serde_json::Value>(&msg.data)
                .ok()?
                .get("type")?
                .as_str()
                .map(String::from)
        })
        .collect();
    
    assert!(event_types.contains(&"heartbeat".to_string()));
    assert!(event_types.contains(&"user_joined".to_string()));
    assert!(event_types.contains(&"message".to_string()));
    assert!(event_types.contains(&"notification".to_string()));
    assert!(event_types.contains(&"status_update".to_string()));
    
    manager.close(&connection_id, None, None).await.unwrap();
}
