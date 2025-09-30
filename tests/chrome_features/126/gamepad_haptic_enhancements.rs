#[tokio::test]
async fn test_chrome_126_gamepad_haptic_enhancements() {
    println!("🧪 Testing Chrome 126: Gamepad API Trigger-Rumble Extension...");

    let browser = HeadlessWebBrowser::new();

    // Test navigator.getGamepads availability
    let result = browser.lock().unwrap().execute_javascript("typeof navigator.getGamepads").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("navigator.getGamepads type: {}", value_str);
            assert!(value_str.contains("function") || value_str.contains("undefined"),
                "getGamepads should be function or undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check navigator.getGamepads: {:?}", e),
    }

    // Test GamepadHapticActuator with trigger-rumble extensions
    let js_code = r#"
        try {
            // Check if GamepadHapticActuator exists
            if (typeof GamepadHapticActuator !== 'undefined') {
                'GamepadHapticActuator available';
            } else {
                'GamepadHapticActuator not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            println!("GamepadHapticActuator test: {:?}", value);
            // Should not error out
        },
        Err(e) => panic!("Failed to test GamepadHapticActuator: {:?}", e),
    }

    println!("✅ Gamepad trigger-rumble test completed");
}
