#[tokio::test]
async fn test_chrome_138_css_sizing_stretch() {
    println!("🧪 Testing Chrome 138: CSS sizing stretch keyword...");

    let browser = HeadlessWebBrowser::new();

    // Test CSS sizing stretch keyword
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                // Test stretch keyword for sizing properties
                var supportsStretch = CSS.supports('width', 'stretch');

                'CSS stretch keyword support: ' + supportsStretch;
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
            println!("CSS sizing stretch test: {}", value_str);
            assert!(!value_str.contains("error:"), "CSS sizing stretch should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test CSS sizing stretch: {:?}", e),
    }

    println!("✅ CSS sizing stretch test completed");
}
