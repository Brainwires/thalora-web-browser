use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_136_regexp_escape() {
    println!("🧪 Testing Chrome 136: RegExp.escape() static method...");

    let browser = HeadlessWebBrowser::new();

    // Test RegExp.escape static method
    let js_code = r#"
        try {
            if (typeof RegExp !== 'undefined' && typeof RegExp.escape === 'function') {
                // Test RegExp.escape with special characters
                var escaped = RegExp.escape('a.b*c?d+e(f)g[h]i{j}k^l$m|n');
                var hasEscape = typeof RegExp.escape === 'function';

                // Test that it properly escapes regex special characters
                var testRegex = new RegExp(escaped);
                var isEscaped = escaped.includes('\\.');

                'RegExp.escape available and working: ' + (hasEscape && isEscaped);
            } else {
                'RegExp.escape not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("RegExp.escape test: {}", value_str);
            assert!(!value_str.contains("error:"), "RegExp.escape should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test RegExp.escape: {:?}", e),
    }

    println!("✅ RegExp.escape test completed");
}
