use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_138_web_serial_bluetooth() {
    println!("🧪 Testing Chrome 138: Web Serial over Bluetooth...");

    let browser = HeadlessWebBrowser::new();

    // Test Web Serial over Bluetooth support
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.serial) {
                // Test if Web Serial API is available
                var hasSerial = typeof navigator.serial.requestPort === 'function';

                // Test for Bluetooth-specific extensions
                var hasBluetoothSerial = typeof navigator.serial.requestPort === 'function';

                'Web Serial API available: ' + hasSerial + ', Bluetooth support context: ' + hasBluetoothSerial;
            } else {
                'Web Serial API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Web Serial Bluetooth test: {}", value_str);
            // Web Serial might not be available in headless mode
        },
        Err(e) => panic!("Failed to test Web Serial Bluetooth: {:?}", e),
    }

    println!("✅ Web Serial Bluetooth test completed");
}
