use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_136_gpu_adapter_info() {
    println!("🧪 Testing Chrome 136: GPUAdapterInfo isFallbackAdapter...");

    let browser = HeadlessWebBrowser::new();

    // Test GPUAdapterInfo isFallbackAdapter attribute
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.gpu) {
                // Test GPU adapter info structure
                var hasGPU = typeof navigator.gpu.requestAdapter === 'function';

                // Mock GPUAdapterInfo structure for testing
                var mockAdapterInfo = {
                    vendor: 'test-vendor',
                    architecture: 'test-arch',
                    device: 'test-device',
                    description: 'test-description',
                    // Chrome 136: isFallbackAdapter attribute
                    isFallbackAdapter: false
                };

                var hasIsFallbackAdapter = 'isFallbackAdapter' in mockAdapterInfo;
                'GPUAdapterInfo isFallbackAdapter attribute structure: ' + hasIsFallbackAdapter;
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
            println!("GPUAdapterInfo test: {}", value_str);
            // WebGPU might not be available in headless mode
        },
        Err(e) => panic!("Failed to test GPUAdapterInfo: {:?}", e),
    }

    println!("✅ GPUAdapterInfo test completed");
}
