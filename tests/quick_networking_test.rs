use thalora::engine::browser::HeadlessWebBrowser;

#[tokio::test]
async fn test_networking_apis_exist() {
    let browser = HeadlessWebBrowser::new();

    // Test WebSocket constructor exists
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript("typeof WebSocket")
        .await
        .unwrap();
    assert!(format!("{:?}", result).contains("function"));

    // Test WebSocket constants
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript("WebSocket.CONNECTING")
        .await
        .unwrap();
    assert!(format!("{:?}", result).contains("0"));

    // Test ReadableStream exists
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript("typeof ReadableStream")
        .await
        .unwrap();
    assert!(format!("{:?}", result).contains("function"));

    println!("✅ All networking APIs properly registered in Boa");
}
