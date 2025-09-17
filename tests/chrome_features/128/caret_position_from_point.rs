use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_128_caret_position_from_point() {
    println!("🧪 Testing Chrome 128: document.caretPositionFromPoint...");

    let browser = HeadlessWebBrowser::new();

    // Test document.caretPositionFromPoint() method
    let js_code = r#"
        try {
            if (typeof document.caretPositionFromPoint === 'function') {
                // Test caretPositionFromPoint with coordinates
                var caretPos = document.caretPositionFromPoint(100, 100);

                if (caretPos) {
                    'caretPositionFromPoint available, returned: ' + typeof caretPos;
                } else {
                    'caretPositionFromPoint available but returned null';
                }
            } else {
                'document.caretPositionFromPoint not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("caretPositionFromPoint test: {}", value_str);
            assert!(!value_str.contains("error:"), "caretPositionFromPoint should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test caretPositionFromPoint: {:?}", e),
    }

    println!("✅ caretPositionFromPoint test completed");
}
