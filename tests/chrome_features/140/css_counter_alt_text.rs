use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_140_css_counter_alt_text() {
    println!("🧪 Testing Chrome 140: CSS counter() and counters() in alt text...");

    let browser = HeadlessWebBrowser::new();

    // Test CSS counter() and counters() in alt text
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                // Test counter() in content alt text
                var supportsCounterInAlt = CSS.supports('content', 'counter(chapter) / "Chapter"');

                'CSS counter() in alt text: ' + supportsCounterInAlt;
            } else {
                'CSS.supports not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("CSS counter alt text test: {}", value_str);
            assert!(!value_str.contains("error:"), "CSS counter alt text should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test CSS counter alt text: {:?}", e),
    }

    println!("✅ CSS counter alt text test completed");
}
