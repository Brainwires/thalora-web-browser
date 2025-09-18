#[tokio::test]
async fn test_chrome_124_websocketstream_api() {
    println!("🧪 Testing Chrome 124: WebSocketStream API...");

    println!("🔧 Creating HeadlessWebBrowser...");
    let browser = HeadlessWebBrowser::new();
    println!("🔧 HeadlessWebBrowser created successfully");

    // Test WebSocketStream constructor availability
    println!("🔧 Testing WebSocketStream constructor availability...");
    let result = browser.lock().unwrap().execute_javascript("typeof WebSocketStream").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebSocketStream type: {}", value_str);
            assert!(value_str.contains("function"), "WebSocketStream should be available as constructor, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check WebSocketStream: {:?}", e),
    }

    // Test WebSocketStream has expected methods
    println!("🔧 Testing WebSocketStream prototype...");
    let result = browser.lock().unwrap().execute_javascript("WebSocketStream.prototype.constructor === WebSocketStream").await;
    match result {
        Ok(value) => {
            println!("WebSocketStream prototype check: {:?}", value);
            assert!(format!("{:?}", value).contains("true"), "WebSocketStream prototype should be properly set up");
        },
        Err(e) => panic!("Failed to check WebSocketStream prototype: {:?}", e),
    }

    println!("✅ WebSocketStream API test completed");
}
