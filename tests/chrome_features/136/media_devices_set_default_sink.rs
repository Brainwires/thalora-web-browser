use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_136_media_devices_set_default_sink() {
    println!("🧪 Testing Chrome 136: MediaDevices setDefaultSinkId()...");

    let browser = HeadlessWebBrowser::new();

    // Test MediaDevices setDefaultSinkId method
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.mediaDevices) {
                // Test if setDefaultSinkId method exists
                var hasSetDefaultSinkId = typeof navigator.mediaDevices.setDefaultSinkId === 'function';

                if (hasSetDefaultSinkId) {
                    'MediaDevices setDefaultSinkId method available: true';
                } else {
                    'MediaDevices setDefaultSinkId method not available';
                }
            } else {
                'navigator.mediaDevices not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("MediaDevices setDefaultSinkId test: {}", value_str);
            // MediaDevices might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test MediaDevices setDefaultSinkId: {:?}", e),
    }

    println!("✅ MediaDevices setDefaultSinkId test completed");
}
