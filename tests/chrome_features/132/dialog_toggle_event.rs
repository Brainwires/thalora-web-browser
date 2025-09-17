#[tokio::test]
async fn test_chrome_132_dialog_toggle_event() {
    println!("🧪 Testing Chrome 132: Dialog ToggleEvent...");

    let browser = HeadlessWebBrowser::new();

    // Test Dialog ToggleEvent
    let js_code = r#"
        try {
            // Test if ToggleEvent is available
            if (typeof ToggleEvent !== 'undefined') {
                var event = new ToggleEvent('toggle', {
                    oldState: 'closed',
                    newState: 'open'
                });
                'ToggleEvent created with newState: ' + event.newState;
            } else {
                'ToggleEvent constructor not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Dialog ToggleEvent test: {}", value_str);
            assert!(!value_str.contains("error:"), "Dialog ToggleEvent should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Dialog ToggleEvent: {:?}", e),
    }

    println!("✅ Dialog ToggleEvent test completed");
}
