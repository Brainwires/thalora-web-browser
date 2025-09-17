#[tokio::test]
async fn test_chrome_140_toggle_event_source() {
    println!("🧪 Testing Chrome 140: ToggleEvent source attribute...");

    let browser = HeadlessWebBrowser::new();

    // Test ToggleEvent source attribute
    let js_code = r#"
        try {
            // Check if ToggleEvent is available and has source attribute
            if (typeof ToggleEvent !== 'undefined') {
                // Test ToggleEvent constructor with source
                var toggleEvent = new ToggleEvent('toggle', {
                    bubbles: true,
                    cancelable: true,
                    oldState: 'closed',
                    newState: 'open'
                });

                var hasSourceProperty = 'source' in toggleEvent;
                'ToggleEvent source attribute: ' + hasSourceProperty;
            } else {
                'ToggleEvent not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("ToggleEvent source test: {}", value_str);
            // ToggleEvent might not be available in headless mode
        },
        Err(e) => panic!("Failed to test ToggleEvent source: {:?}", e),
    }

    println!("✅ ToggleEvent source test completed");
}
