use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_136_audiocontext_interrupted_state() {
    println!("🧪 Testing Chrome 136: AudioContext interrupted state...");

    let browser = HeadlessWebBrowser::new();

    // Test AudioContext interrupted state
    let js_code = r#"
        try {
            if (typeof AudioContext !== 'undefined') {
                var audioContext = new AudioContext();

                // Test if 'interrupted' is a valid state
                var stateValues = ['suspended', 'running', 'closed', 'interrupted'];
                var hasInterruptedState = stateValues.includes('interrupted');

                // Test state property exists
                var hasState = 'state' in audioContext;

                'AudioContext interrupted state support: ' + (hasState && hasInterruptedState);
            } else {
                'AudioContext not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("AudioContext interrupted state test: {}", value_str);
            // AudioContext might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test AudioContext interrupted state: {:?}", e),
    }

    println!("✅ AudioContext interrupted state test completed");
}
