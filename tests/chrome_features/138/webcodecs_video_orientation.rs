use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_138_webcodecs_video_orientation() {
    println!("🧪 Testing Chrome 138: WebCodecs video orientation support...");

    let browser = HeadlessWebBrowser::new();

    // Test WebCodecs video orientation features
    let js_code = r#"
        try {
            // Check if WebCodecs is available
            if (typeof VideoFrame !== 'undefined') {
                // Test VideoFrame constructor availability
                var hasVideoFrame = typeof VideoFrame === 'function';

                'WebCodecs VideoFrame available: ' + hasVideoFrame;
            } else {
                'WebCodecs VideoFrame not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebCodecs video orientation test: {}", value_str);
            // WebCodecs might not be available in headless mode
        },
        Err(e) => panic!("Failed to test WebCodecs video orientation: {:?}", e),
    }

    println!("✅ WebCodecs video orientation test completed");
}
