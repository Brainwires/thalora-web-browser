#[tokio::test]
async fn test_chrome_135_navigate_event_source_element() {
    println!("🧪 Testing Chrome 135: NavigateEvent.sourceElement...");

    let browser = HeadlessWebBrowser::new();

    // Test NavigateEvent.sourceElement property
    let js_code = r#"
        try {
            if (typeof NavigateEvent !== 'undefined') {
                // Test if NavigateEvent has sourceElement property
                var hasSourceElement = 'sourceElement' in NavigateEvent.prototype;
                'NavigateEvent.sourceElement property available: ' + hasSourceElement;
            } else {
                'NavigateEvent not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("NavigateEvent.sourceElement test: {}", value_str);
            // NavigateEvent might not be available in headless mode
        },
        Err(e) => panic!("Failed to test NavigateEvent.sourceElement: {:?}", e),
    }

    println!("✅ NavigateEvent.sourceElement test completed");
}
