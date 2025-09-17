use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_135_command_attributes() {
    println!("🧪 Testing Chrome 135: Command and commandfor attributes...");

    let browser = HeadlessWebBrowser::new();

    // Test command and commandfor attributes
    let js_code = r#"
        try {
            if (typeof document !== 'undefined') {
                var button = document.createElement('button');

                // Test setting command and commandfor attributes
                button.setAttribute('command', 'show-popover');
                button.setAttribute('commandfor', 'my-element');

                var hasCommand = button.getAttribute('command') === 'show-popover';
                var hasCommandFor = button.getAttribute('commandfor') === 'my-element';

                'Command attributes supported: ' + (hasCommand && hasCommandFor);
            } else {
                'document not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Command attributes test: {}", value_str);
            assert!(!value_str.contains("error:"), "Command attributes should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test command attributes: {:?}", e),
    }

    println!("✅ Command attributes test completed");
}
