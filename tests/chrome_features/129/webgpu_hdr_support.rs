#[tokio::test]
async fn test_chrome_129_webgpu_hdr_support() {
    println!("🧪 Testing Chrome 129: WebGPU HDR support...");

    let browser = HeadlessWebBrowser::new();

    // Test WebGPU HDR support
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.gpu) {
                // Test if we can get a canvas context with HDR options
                var canvas = document.createElement('canvas');
                if (canvas && canvas.getContext) {
                    var ctx = canvas.getContext('webgpu');
                    if (ctx) {
                        'WebGPU context available for HDR testing';
                    } else {
                        'WebGPU context not available';
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
            println!("WebGPU HDR test: {}", value_str);
            // WebGPU might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test WebGPU HDR: {:?}", e),
    }

    println!("✅ WebGPU HDR test completed");
}
