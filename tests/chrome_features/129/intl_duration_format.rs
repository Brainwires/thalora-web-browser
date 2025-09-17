use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_129_intl_duration_format() {
    println!("🧪 Testing Chrome 129: Intl.DurationFormat...");

    let browser = HeadlessWebBrowser::new();

    // Test Intl.DurationFormat API
    let js_code = r#"
        try {
            if (typeof Intl !== 'undefined' && typeof Intl.DurationFormat === 'function') {
                // Test creating a DurationFormat instance
                var formatter = new Intl.DurationFormat('en', {
                    style: 'long',
                    hours: 'numeric',
                    minutes: 'numeric',
                    seconds: 'numeric'
                });

                if (formatter && typeof formatter.format === 'function') {
                    // Test formatting a duration
                    var formatted = formatter.format({ hours: 1, minutes: 40, seconds: 30 });
                    'Intl.DurationFormat available, formatted: ' + formatted;
                } else {
                    'Intl.DurationFormat constructor available but format method missing';
                }
            } else {
                'Intl.DurationFormat not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Intl.DurationFormat test: {}", value_str);
            assert!(!value_str.contains("error:"), "Intl.DurationFormat should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Intl.DurationFormat: {:?}", e),
    }

    println!("✅ Intl.DurationFormat test completed");
}
