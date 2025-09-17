#[tokio::test]
async fn test_chrome_125_duplicate_named_groups() {
    println!("🧪 Testing Chrome 125: Duplicate Named Capture Groups...");

    let browser = HeadlessWebBrowser::new();

    // Test duplicate named capture groups in alternatives
    let js_code = r#"
        try {
            // Same named group in different alternatives
            const regex = /(?<year>[0-9]{4})-[0-9]{2}|[0-9]{2}-(?<year>[0-9]{4})/;
            const match1 = regex.exec('2024-12');
            const match2 = regex.exec('12-2024');

            'success: ' + (match1 ? match1.groups.year : 'null') + ',' + (match2 ? match2.groups.year : 'null');
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Duplicate named groups test: {}", value_str);
            // Should not throw syntax errors
            assert!(!value_str.contains("SyntaxError"), "Duplicate named groups should not cause syntax errors, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test duplicate named groups: {:?}", e),
    }

    println!("✅ Duplicate named groups test completed");
}
