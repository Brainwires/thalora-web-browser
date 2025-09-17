use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_138_css_env_font_scale() {
    println!("🧪 Testing Chrome 138: CSS env() font scale...");

    let browser = HeadlessWebBrowser::new();

    // Test CSS env() font scale variable
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                // Test env() font scale support
                var supportsFontScale = CSS.supports('font-size', 'env(font-scale)');

                'CSS env(font-scale) support: ' + supportsFontScale;
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
            println!("CSS env font scale test: {}", value_str);
            assert!(!value_str.contains("error:"), "CSS env font scale should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test CSS env font scale: {:?}", e),
    }

    println!("✅ CSS env font scale test completed");
}
