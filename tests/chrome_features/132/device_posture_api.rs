#[tokio::test]
async fn test_chrome_132_device_posture_api() {
    println!("🧪 Testing Chrome 132: Device Posture API...");

    let browser = HeadlessWebBrowser::new();

    // Test Device Posture API
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.devicePosture) {
                // Test Device Posture API
                var hasDevicePosture = typeof navigator.devicePosture === 'object';
                var posture = navigator.devicePosture.type || 'continuous';
                'Device Posture API available, type: ' + posture;
            } else {
                'Device Posture API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Device Posture API test: {}", value_str);
            // Device Posture API might not be available on non-foldable devices
        },
        Err(e) => panic!("Failed to test Device Posture API: {:?}", e),
    }

    println!("✅ Device Posture API test completed");
}
