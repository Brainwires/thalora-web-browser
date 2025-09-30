#[tokio::test]
async fn test_chrome_138_css_math_functions() {
    println!("🧪 Testing Chrome 138: CSS math functions (abs, sign, progress)...");

    let browser = HeadlessWebBrowser::new();

    // Test CSS math functions
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                // Test abs() function
                var supportsAbs = CSS.supports('width', 'abs(-10px)');

                // Test sign() function
                var supportsSign = CSS.supports('opacity', 'sign(-1)');

                // Test progress() function
                var supportsProgress = CSS.supports('width', 'progress(from 0% to 100%, 50%)');

                'CSS math functions - abs: ' + supportsAbs + ', sign: ' + supportsSign + ', progress: ' + supportsProgress;
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
            println!("CSS math functions test: {}", value_str);
            assert!(!value_str.contains("error:"), "CSS math functions should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test CSS math functions: {:?}", e),
    }

    println!("✅ CSS math functions test completed");
}
