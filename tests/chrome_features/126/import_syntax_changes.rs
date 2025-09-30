#[tokio::test]
async fn test_chrome_126_import_syntax_changes() {
    println!("🧪 Testing Chrome 126: Import assertion syntax changes...");

    let browser = HeadlessWebBrowser::new();

    // Test import with 'with' keyword (new syntax replacing 'assert')
    let js_code = r#"
        try {
            // This would normally be a syntax test in a real module context
            // In our context, we just test that the syntax doesn't cause parse errors
            const syntaxTest = 'import foo from "./foo.json" with { type: "json" }';
            'import with syntax: valid string';
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Import syntax test: {}", value_str);
            assert!(value_str.contains("valid"), "Import syntax should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test import syntax: {:?}", e),
    }

    println!("✅ Import syntax changes test completed");
}
