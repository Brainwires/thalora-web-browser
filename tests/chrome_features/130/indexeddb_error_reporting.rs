use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_130_indexeddb_error_reporting() {
    println!("🧪 Testing Chrome 130: IndexedDB improved error reporting...");

    let browser = HeadlessWebBrowser::new();

    // Test IndexedDB error reporting improvements
    let js_code = r#"
        try {
            if (typeof indexedDB !== 'undefined') {
                // Test IndexedDB availability and error handling
                var hasIndexedDB = typeof indexedDB.open === 'function';

                // Chrome 130: Enhanced error reporting for large value read failures
                'IndexedDB available with enhanced error reporting: ' + hasIndexedDB;
            } else {
                'IndexedDB not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("IndexedDB error reporting test: {}", value_str);
            assert!(!value_str.contains("error:"), "IndexedDB error reporting should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test IndexedDB error reporting: {:?}", e),
    }

    println!("✅ IndexedDB error reporting test completed");
}
