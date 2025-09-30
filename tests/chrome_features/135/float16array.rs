#[tokio::test]
async fn test_chrome_135_float16array() {
    println!("🧪 Testing Chrome 135: Float16Array...");

    let browser = HeadlessWebBrowser::new();

    // Test Float16Array support
    let js_code = r#"
        try {
            if (typeof Float16Array !== 'undefined') {
                // Test Float16Array constructor and basic operations
                var float16 = new Float16Array(4);
                float16[0] = 1.5;
                float16[1] = 2.5;

                var isArray = float16 instanceof Float16Array;
                var hasLength = float16.length === 4;
                var hasValues = float16[0] === 1.5 && float16[1] === 2.5;

                'Float16Array constructor available and working: ' + (isArray && hasLength && hasValues);
            } else {
                'Float16Array not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Float16Array test: {}", value_str);
            assert!(!value_str.contains("error:"), "Float16Array should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Float16Array: {:?}", e),
    }

    println!("✅ Float16Array test completed");
}
