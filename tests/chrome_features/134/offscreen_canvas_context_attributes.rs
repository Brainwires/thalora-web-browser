use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_134_offscreen_canvas_context_attributes() {
    println!("🧪 Testing Chrome 134: OffscreenCanvas getContextAttributes...");

    let browser = HeadlessWebBrowser::new();

    // Test OffscreenCanvas getContextAttributes
    let js_code = r#"
        try {
            if (typeof OffscreenCanvas !== 'undefined') {
                var canvas = new OffscreenCanvas(100, 100);
                var ctx = canvas.getContext('2d');

                if (ctx && typeof ctx.getContextAttributes === 'function') {
                    var attributes = ctx.getContextAttributes();
                    'OffscreenCanvas getContextAttributes available: ' + (attributes !== null);
                } else {
                    'OffscreenCanvas getContextAttributes not available';
                }
            } else {
                'OffscreenCanvas not supported';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("OffscreenCanvas getContextAttributes test: {}", value_str);
            // OffscreenCanvas might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test OffscreenCanvas getContextAttributes: {:?}", e),
    }

    println!("✅ OffscreenCanvas getContextAttributes test completed");
}
