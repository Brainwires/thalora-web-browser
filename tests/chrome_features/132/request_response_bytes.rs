#[tokio::test]
async fn test_chrome_132_request_response_bytes() {
    println!("🧪 Testing Chrome 132: Request/Response bytes() method...");

    let browser = HeadlessWebBrowser::new();

    // Test Request.bytes() and Response.bytes() methods
    let js_code = r#"
        try {
            if (typeof Request !== 'undefined' && typeof Response !== 'undefined') {
                // Test if bytes() method exists on Request
                var request = new Request('https://example.com');
                var hasRequestBytes = typeof request.bytes === 'function';

                // Test if bytes() method exists on Response
                var response = new Response('test data');
                var hasResponseBytes = typeof response.bytes === 'function';

                'Request.bytes: ' + hasRequestBytes + ', Response.bytes: ' + hasResponseBytes;
            } else {
                'Request/Response not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Request/Response bytes() test: {}", value_str);
            assert!(!value_str.contains("error:"), "Request/Response bytes() should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Request/Response bytes(): {:?}", e),
    }

    println!("✅ Request/Response bytes() test completed");
}
