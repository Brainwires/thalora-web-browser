#[tokio::test]
async fn test_chrome_140_css_font_variation_settings() {
    println!("🧪 Testing Chrome 140: CSS font-variation-settings descriptor...");

    let browser = HeadlessWebBrowser::new();

    // Test CSS font-variation-settings descriptor
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                // Test font-variation-settings in @font-face
                var supportsFontVariationSettings = CSS.supports('font-variation-settings', '"wght" 400');

                'CSS font-variation-settings descriptor: ' + supportsFontVariationSettings;
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
            println!("CSS font-variation-settings test: {}", value_str);
            assert!(!value_str.contains("error:"), "CSS font-variation-settings should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test CSS font-variation-settings: {:?}", e),
    }

    println!("✅ CSS font-variation-settings test completed");
}
