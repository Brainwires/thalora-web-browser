#[tokio::test]
async fn test_chrome_137_css_if_function() {
    println!("🧪 Testing Chrome 137: CSS if() function...");

    let browser = HeadlessWebBrowser::new();

    // Test CSS if() function support
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                // Test CSS if() function support
                var supportsIf = CSS.supports('color', 'if(true, red, blue)');

                'CSS if() function supported: ' + supportsIf;
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
            println!("CSS if() function test: {}", value_str);
            assert!(!value_str.contains("error:"), "CSS if() function should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test CSS if() function: {:?}", e),
    }

    println!("✅ CSS if() function test completed");
}
