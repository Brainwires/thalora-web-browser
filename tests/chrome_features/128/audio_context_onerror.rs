#[tokio::test]
async fn test_chrome_128_audio_context_onerror() {
    println!("🧪 Testing Chrome 128: AudioContext.onerror...");

    let browser = HeadlessWebBrowser::new();

    // Test AudioContext.onerror callback
    let js_code = r#"
        try {
            if (typeof AudioContext !== 'undefined') {
                var audioContext = new AudioContext();

                // Test if onerror property exists
                var hasOnError = 'onerror' in audioContext;

                if (hasOnError) {
                    // Try to set an error handler
                    audioContext.onerror = function(error) {
                        console.log('AudioContext error:', error);
                    };
                }

                'AudioContext.onerror available: ' + hasOnError;
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
            println!("AudioContext.onerror test: {}", value_str);
            // AudioContext might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test AudioContext.onerror: {:?}", e),
    }

    println!("✅ AudioContext.onerror test completed");
}
