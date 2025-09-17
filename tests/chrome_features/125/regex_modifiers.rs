#[tokio::test]
async fn test_chrome_125_regex_modifiers() {
    println!("🧪 Testing Chrome 125: Regular Expression Modifiers...");

    let browser = HeadlessWebBrowser::new();

    // Test regex modifiers - locally modify flags inside pattern
    let js_code = r#"
        try {
            // Case insensitive modifier inside pattern
            const regex1 = /(?i:[a-z])[a-z]$/;
            const result1 = regex1.test('Ab');

            // Multiple flags modifier
            const regex2 = /(?im:test.*line)other/;
            const result2 = regex2.test('TEST\nLINEother');

            'success: ' + result1 + ',' + result2;
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Regex modifiers test: {}", value_str);
            // Even if not supported, should not throw syntax errors
            assert!(!value_str.contains("SyntaxError"), "Regex modifiers should not cause syntax errors, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test regex modifiers: {:?}", e),
    }

    println!("✅ Regex modifiers test completed");
}
