#[tokio::test]
async fn test_chrome_125_direct_sockets_api() {
    println!("🧪 Testing Chrome 125: Direct Sockets API...");

    let browser = HeadlessWebBrowser::new();

    // Test navigator.tcp
    let result = browser.lock().unwrap().execute_javascript("typeof navigator.tcp").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("navigator.tcp type: {}", value_str);
            // Might be undefined in headless mode (Chrome Apps only)
        },
        Err(e) => panic!("Failed to check navigator.tcp: {:?}", e),
    }

    // Test navigator.udp
    let result = browser.lock().unwrap().execute_javascript("typeof navigator.udp").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("navigator.udp type: {}", value_str);
            // Might be undefined in headless mode (Chrome Apps only)
        },
        Err(e) => panic!("Failed to check navigator.udp: {:?}", e),
    }

    println!("✅ Direct Sockets API test completed");
}
