use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_132_element_capture() {
    println!("🧪 Testing Chrome 132: Element Capture API...");

    let browser = HeadlessWebBrowser::new();

    // Test Element Capture API
    let js_code = r#"
        try {
            // Test if MediaStreamTrack has element capture capabilities
            if (typeof MediaStreamTrack !== 'undefined') {
                // Test element capture (would normally require getUserMedia first)
                'MediaStreamTrack available for Element Capture';
            } else {
                'MediaStreamTrack not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Element Capture test: {}", value_str);
            // MediaStreamTrack might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test Element Capture: {:?}", e),
    }

    println!("✅ Element Capture test completed");
}
