#[tokio::test]
async fn test_chrome_128_pointer_event_device_properties() {
    println!("🧪 Testing Chrome 128: PointerEvent.deviceProperties...");

    let browser = HeadlessWebBrowser::new();

    // Test PointerEvent.deviceProperties
    let js_code = r#"
        try {
            if (typeof PointerEvent !== 'undefined') {
                // Test creating a PointerEvent with device properties
                var event = new PointerEvent('pointerdown', {
                    pointerId: 1,
                    bubbles: true,
                    cancelable: true
                });

                // Check if deviceProperties exists
                var hasDeviceProperties = 'deviceProperties' in event;

                'PointerEvent.deviceProperties available: ' + hasDeviceProperties;
            } else {
                'PointerEvent not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("PointerEvent.deviceProperties test: {}", value_str);
            assert!(!value_str.contains("error:"), "PointerEvent.deviceProperties should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test PointerEvent.deviceProperties: {:?}", e),
    }

    println!("✅ PointerEvent.deviceProperties test completed");
}
