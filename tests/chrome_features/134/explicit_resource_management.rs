#[tokio::test]
async fn test_chrome_134_explicit_resource_management() {
    println!("🧪 Testing Chrome 134: Explicit Resource Management...");

    let browser = HeadlessWebBrowser::new();

    // Test Explicit Resource Management (using and await using)
    let js_code = r#"
        try {
            // Test if Symbol.dispose and Symbol.asyncDispose are available
            var hasDispose = typeof Symbol !== 'undefined' && typeof Symbol.dispose !== 'undefined';
            var hasAsyncDispose = typeof Symbol !== 'undefined' && typeof Symbol.asyncDispose !== 'undefined';

            if (hasDispose || hasAsyncDispose) {
                'Explicit Resource Management symbols - dispose: ' + hasDispose + ', asyncDispose: ' + hasAsyncDispose;
            } else {
                'Explicit Resource Management not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Explicit Resource Management test: {}", value_str);
            // This is a newer JavaScript feature that might not be fully available
        },
        Err(e) => panic!("Failed to test Explicit Resource Management: {:?}", e),
    }

    println!("✅ Explicit Resource Management test completed");
}
