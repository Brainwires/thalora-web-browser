use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_124_pageswap_event() {
    println!("🧪 Testing Chrome 124: pageswap event...");

    let browser = HeadlessWebBrowser::new();

    // Test that pageswap event can be listened to
    let result = browser.lock().unwrap().execute_javascript("typeof window.addEventListener").await;
    match result {
        Ok(value) => {
            println!("addEventListener available: {:?}", value);
            assert!(format!("{:?}", value).contains("function"), "addEventListener should be available");
        },
        Err(e) => panic!("Failed to check addEventListener: {:?}", e),
    }

    // Test pageswap event registration (should not throw)
    let js_code = r#"
        try {
            window.addEventListener('pageswap', function(event) {
                // Event handler for pageswap
            });
            'success'
        } catch (e) {
            'error: ' + e.message
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("pageswap event registration: {}", value_str);
            assert!(value_str.contains("success"), "pageswap event should be registerable, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test pageswap event: {:?}", e),
    }

    println!("✅ pageswap event test completed");
}
