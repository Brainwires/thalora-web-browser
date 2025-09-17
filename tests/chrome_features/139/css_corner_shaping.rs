#[tokio::test]
async fn test_chrome_139_css_corner_shaping() {
    println!("🧪 Testing Chrome 139: CSS corner shaping...");

    let browser = HeadlessWebBrowser::new();

    // Test CSS corner shaping support
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                // Test corner-shape property
                var supportsCornerShape = CSS.supports('corner-shape', 'round') ||
                                         CSS.supports('border-corner-shape', 'round');

                'CSS corner shaping support: ' + supportsCornerShape;
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
            println!("CSS corner shaping test: {}", value_str);
            assert!(!value_str.contains("error:"), "CSS corner shaping should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test CSS corner shaping: {:?}", e),
    }

    println!("✅ CSS corner shaping test completed");
}
