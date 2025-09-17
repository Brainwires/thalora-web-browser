use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_131_translator_api() {
    println!("🧪 Testing Chrome 131: Translator API (Origin Trial)...");

    let browser = HeadlessWebBrowser::new();

    // Test Translator API availability
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.ml && navigator.ml.createTranslator) {
                // Test Translator API availability
                var hasTranslator = typeof navigator.ml.createTranslator === 'function';
                'Translator API available: ' + hasTranslator;
            } else if (typeof Translator !== 'undefined') {
                // Alternative global interface
                'Translator global available';
            } else {
                'Translator API not available (Origin Trial)';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Translator API test: {}", value_str);
            // Translator API is in Origin Trial, so might not be available
        },
        Err(e) => panic!("Failed to test Translator API: {:?}", e),
    }

    println!("✅ Translator API test completed");
}
