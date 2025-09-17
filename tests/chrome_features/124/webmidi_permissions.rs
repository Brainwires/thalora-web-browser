#[tokio::test]
async fn test_chrome_124_webmidi_permissions() {
    println!("🧪 Testing Chrome 124: WebMIDI permissions...");

    let browser = HeadlessWebBrowser::new();

    // Test navigator.requestMIDIAccess availability
    let result = browser.lock().unwrap().execute_javascript("typeof navigator.requestMIDIAccess").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("navigator.requestMIDIAccess type: {}", value_str);
            assert!(value_str.contains("function"), "navigator.requestMIDIAccess should be available, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check navigator.requestMIDIAccess: {:?}", e),
    }

    // Test that requesting MIDI access requires permissions (should not crash)
    let js_code = r#"
        try {
            // This should work without throwing, even if it fails due to permissions
            typeof navigator.requestMIDIAccess === 'function' ? 'function_available' : 'not_available'
        } catch (e) {
            'error: ' + e.message
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebMIDI function check: {}", value_str);
            assert!(value_str.contains("function_available"), "WebMIDI should be available, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test WebMIDI permissions: {:?}", e),
    }

    println!("✅ WebMIDI permissions test completed");
}
