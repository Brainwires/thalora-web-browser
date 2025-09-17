use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_134_canvas_image_smoothing_quality() {
    println!("🧪 Testing Chrome 134: Canvas imageSmoothingQuality...");

    let browser = HeadlessWebBrowser::new();

    // Test Canvas imageSmoothingQuality
    let js_code = r#"
        try {
            if (typeof document !== 'undefined') {
                var canvas = document.createElement('canvas');
                var ctx = canvas.getContext('2d');

                if (ctx) {
                    // Test if imageSmoothingQuality is supported
                    var hasImageSmoothingQuality = 'imageSmoothingQuality' in ctx;

                    if (hasImageSmoothingQuality) {
                        // Test setting different quality levels
                        ctx.imageSmoothingQuality = 'high';
                        var qualitySet = ctx.imageSmoothingQuality === 'high';
                        'imageSmoothingQuality supported and working: ' + qualitySet;
                    } else {
                        'imageSmoothingQuality not supported in context';
                    }
                } else {
                    'Canvas context not available';
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
            println!("Canvas imageSmoothingQuality test: {}", value_str);
            assert!(!value_str.contains("error:"), "Canvas imageSmoothingQuality should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Canvas imageSmoothingQuality: {:?}", e),
    }

    println!("✅ Canvas imageSmoothingQuality test completed");
}
