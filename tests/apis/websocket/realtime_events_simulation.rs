#[tokio::test]
async fn test_realtime_events_simulation() {
    let manager = WebSocketManager::new();

    // Get global test server URL (starts server if needed)
    let url = get_test_server_url().await;

    let connection_id = manager.connect(&url, None).await.unwrap();
    
    // Simulate various real-time events
    let events = vec!["heartbeat", "user_joined", "message", "notification", "status_update"];
    manager.simulate_realtime_events(&connection_id, events).await.unwrap();
    
    let (_sent, received) = manager.get_message_history(&connection_id).unwrap();
    // Filter only the JSON event messages (ignore server identification messages)
    let json_messages: Vec<_> = received.iter()
        .filter(|msg| serde_json::from_str::<serde_json::Value>(&msg.data).is_ok())
        .collect();

    // We expect exactly 5 simulated JSON messages
    assert_eq!(json_messages.len(), 5, "Expected 5 JSON event messages, got {} total messages", received.len());
    
    // Verify event types were properly simulated (using only JSON messages)
    let event_types: Vec<String> = json_messages.iter()
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
