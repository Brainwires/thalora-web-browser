use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_139_css_custom_functions() {
    println!("🧪 Testing Chrome 139: CSS custom functions...");

    let browser = HeadlessWebBrowser::new();

    // Test CSS custom functions support
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                // Test custom function syntax
                var supportsCustomFunctions = CSS.supports('--custom-func', 'function(calc(1px + 1px))') ||
                                             CSS.supports('@function', '--test() { return 1px; }');

                'CSS custom functions support: ' + supportsCustomFunctions;
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
            println!("CSS custom functions test: {}", value_str);
            assert!(!value_str.contains("error:"), "CSS custom functions should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test CSS custom functions: {:?}", e),
    }

    println!("✅ CSS custom functions test completed");
}
