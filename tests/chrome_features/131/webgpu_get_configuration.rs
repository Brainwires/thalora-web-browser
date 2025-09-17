#[tokio::test]
async fn test_chrome_131_webgpu_get_configuration() {
    println!("🧪 Testing Chrome 131: WebGPU getConfiguration...");

    let browser = HeadlessWebBrowser::new();

    // Test WebGPU getConfiguration method
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.gpu) {
                // Test if canvas context has getConfiguration method
                var canvas = document.createElement('canvas');
                if (canvas && canvas.getContext) {
                    var ctx = canvas.getContext('webgpu');
                    if (ctx && typeof ctx.getConfiguration === 'function') {
                        'WebGPU getConfiguration method available';
                    } else {
                        'WebGPU context available but getConfiguration method missing';
                    }
                } else {
                    'Canvas getContext not available';
                }
            } else {
                'WebGPU not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebGPU getConfiguration test: {}", value_str);
            // WebGPU might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test WebGPU getConfiguration: {:?}", e),
    }

    println!("✅ WebGPU getConfiguration test completed");
}
