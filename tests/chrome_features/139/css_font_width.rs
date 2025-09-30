#[tokio::test]
async fn test_chrome_139_css_font_width() {
    println!("🧪 Testing Chrome 139: CSS font-width property...");

    let browser = HeadlessWebBrowser::new();

    // Test CSS font-width property
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                // Test font-width property (CSS Fonts Level 4)
                var supportsFontWidth = CSS.supports('font-width', 'condensed');

                // Test font-stretch as legacy alias
                var supportsFontStretch = CSS.supports('font-stretch', 'condensed');

                'CSS font-width: ' + supportsFontWidth + ', font-stretch legacy: ' + supportsFontStretch;
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
            println!("CSS font-width test: {}", value_str);
            assert!(!value_str.contains("error:"), "CSS font-width should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test CSS font-width: {:?}", e),
    }

    println!("✅ CSS font-width test completed");
}
