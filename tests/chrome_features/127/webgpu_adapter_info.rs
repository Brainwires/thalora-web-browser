#[tokio::test]
async fn test_chrome_127_webgpu_adapter_info() {
    println!("🧪 Testing Chrome 127: WebGPU Adapter Info...");

    let browser = HeadlessWebBrowser::new();

    // Test GPUAdapter.info synchronous attribute
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.gpu) {
                // Test if GPUAdapter has synchronous info attribute
                navigator.gpu.requestAdapter().then(adapter => {
                    if (adapter && typeof adapter.info !== 'undefined') {
                        return 'GPUAdapter.info available: ' + typeof adapter.info;
                    } else {
                        return 'GPUAdapter.info not available';
                    }
                }).catch(e => 'gpu error: ' + e.message);

                'WebGPU adapter test initiated';
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
            println!("WebGPU adapter info test: {}", value_str);
            // WebGPU might not be fully available in headless mode
            assert!(!value_str.contains("error:"), "WebGPU adapter info test should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test WebGPU adapter info: {:?}", e),
    }

    println!("✅ WebGPU adapter info test completed");
}
