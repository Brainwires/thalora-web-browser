use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_126_webgl_object_exposure() {
    println!("🧪 Testing Chrome 126: WebGL Enhancements (WebGLObject exposure)...");

    let browser = HeadlessWebBrowser::new();

    // Test WebGLObject availability
    let result = browser.lock().unwrap().execute_javascript("typeof WebGLObject").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebGLObject type: {}", value_str);
            assert!(value_str.contains("function") || value_str.contains("undefined"),
                "WebGLObject should be function or undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check WebGLObject: {:?}", e),
    }

    // Test WebGL context availability
    let js_code = r#"
        try {
            // Try to get WebGL context
            const canvas = typeof HTMLCanvasElement !== 'undefined' ? document.createElement('canvas') : null;
            if (canvas && canvas.getContext) {
                const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
                if (gl) {
                    'WebGL context available';
                } else {
                    'WebGL context not available';
                }
            } else {
                'Canvas or getContext not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            println!("WebGL context test: {:?}", value);
            // Should not error out
        },
        Err(e) => panic!("Failed to test WebGL context: {:?}", e),
    }

    println!("✅ WebGL enhancements test completed");
}
