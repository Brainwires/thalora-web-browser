#[tokio::test]
async fn test_chrome_136_canvas_text_lang() {
    println!("🧪 Testing Chrome 136: CanvasTextDrawingStyles lang attribute...");

    let browser = HeadlessWebBrowser::new();

    // Test CanvasTextDrawingStyles lang IDL attribute
    let js_code = r#"
        try {
            if (typeof document !== 'undefined') {
                var canvas = document.createElement('canvas');
                var ctx = canvas.getContext('2d');

                if (ctx) {
                    // Test if lang property exists on canvas context
                    var hasLang = 'lang' in ctx;

                    if (hasLang) {
                        // Test setting language
                        ctx.lang = 'en-US';
                        var langSet = ctx.lang === 'en-US';
                        'CanvasTextDrawingStyles lang attribute: ' + langSet;
                    } else {
                        'CanvasTextDrawingStyles lang attribute not available';
                    }
                } else {
                    'Canvas 2D context not available';
                }
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
            println!("Canvas text lang test: {}", value_str);
            assert!(!value_str.contains("error:"), "Canvas text lang should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Canvas text lang: {:?}", e),
    }

    println!("✅ Canvas text lang test completed");
}
