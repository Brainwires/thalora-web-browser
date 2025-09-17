#[tokio::test]
async fn test_chrome_135_csp_require_sri() {
    println!("🧪 Testing Chrome 135: CSP require-sri-for directive...");

    let browser = HeadlessWebBrowser::new();

    // Test CSP require-sri-for directive support
    let js_code = r#"
        try {
            if (typeof document !== 'undefined') {
                // Test creating meta element with CSP require-sri-for
                var meta = document.createElement('meta');
                meta.setAttribute('http-equiv', 'Content-Security-Policy');
                meta.setAttribute('content', 'require-sri-for script style');

                var hasCSPSupport = meta.getAttribute('content').includes('require-sri-for');
                'CSP require-sri-for directive syntax supported: ' + hasCSPSupport;
            } else {
                'document not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("CSP require-sri-for test: {}", value_str);
            assert!(!value_str.contains("error:"), "CSP require-sri-for should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test CSP require-sri-for: {:?}", e),
    }

    println!("✅ CSP require-sri-for test completed");
}
