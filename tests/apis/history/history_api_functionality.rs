#[tokio::test]
async fn test_history_api_functionality() {
    println!("🧪 Testing History API functionality...");
    let browser = HeadlessWebBrowser::new();
    // Test pushState
    let result = browser.lock().unwrap().execute_javascript("history.pushState({test: 'data'}, 'Test', '/test'); 'success'").await;
    match result {
        Ok(value) => {
            println!("pushState result: {:?}", value);
            assert!(format!("{:?}", value).contains("success"), "pushState should execute without error");
        },
        Err(e) => panic!("pushState failed: {:?}", e),
    }
    // Test state property
    let result = browser.lock().unwrap().execute_javascript("history.state").await;
    match result {
        Ok(value) => {
            println!("history.state: {:?}", value);
            // Should not be undefined
            assert!(!format!("{:?}", value).contains("undefined"), "history.state should be defined after pushState");
        },
        Err(e) => panic!("Failed to get history.state: {:?}", e),
    }
    println!("✅ History API functionality test completed");
}
