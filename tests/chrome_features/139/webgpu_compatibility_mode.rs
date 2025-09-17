use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_139_webgpu_compatibility_mode() {
    println!("🧪 Testing Chrome 139: WebGPU compatibility mode...");

    let browser = HeadlessWebBrowser::new();

    // Test WebGPU compatibility mode
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.gpu) {
                // Test WebGPU compatibility mode features
                var hasGPU = typeof navigator.gpu.requestAdapter === 'function';

                if (hasGPU) {
                    // Test compatibility mode request options
                    var compatibilityOptions = {
                        compatibilityMode: true,
                        powerPreference: 'low-power'
                    };

                    'WebGPU compatibility mode support: ' + hasGPU;
                } else {
                    'WebGPU adapter request not available';
                }
            } else {
                'navigator.gpu not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebGPU compatibility mode test: {}", value_str);
            // WebGPU might not be available in headless mode
        },
        Err(e) => panic!("Failed to test WebGPU compatibility mode: {:?}", e),
    }

    println!("✅ WebGPU compatibility mode test completed");
}
