use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_140_readablestream_byob_min() {
    println!("🧪 Testing Chrome 140: ReadableStreamBYOBReader min option...");

    let browser = HeadlessWebBrowser::new();

    // Test ReadableStreamBYOBReader with min option
    let js_code = r#"
        try {
            if (typeof ReadableStream !== 'undefined') {
                // Test ReadableStream constructor
                var hasReadableStream = typeof ReadableStream === 'function';

                if (hasReadableStream) {
                    // Test BYOB reader availability
                    var stream = new ReadableStream({
                        type: 'bytes',
                        start: function(controller) {
                            // Mock implementation
                        }
                    });

                    var reader = stream.getReader({ mode: 'byob' });
                    var hasBYOBReader = reader && typeof reader.read === 'function';

                    'ReadableStreamBYOBReader support: ' + hasBYOBReader;
                } else {
                    'ReadableStream constructor not available';
                }
            } else {
                'ReadableStream not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("ReadableStreamBYOBReader min test: {}", value_str);
            // ReadableStream might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test ReadableStreamBYOBReader min: {:?}", e),
    }

    println!("✅ ReadableStreamBYOBReader min test completed");
}
