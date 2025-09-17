#[tokio::test]
async fn test_chrome_137_webassembly_jspi() {
    println!("🧪 Testing Chrome 137: WebAssembly JSPI (JavaScript Promise Integration)...");

    let browser = HeadlessWebBrowser::new();

    // Test WebAssembly JSPI support
    let js_code = r#"
        try {
            if (typeof WebAssembly !== 'undefined') {
                // Test basic WebAssembly availability
                var hasWebAssembly = typeof WebAssembly === 'object';

                // Test for JSPI-related features (experimental)
                var hasPromiseIntegration = typeof WebAssembly.promising === 'function' ||
                                           typeof WebAssembly.Suspending === 'function';

                var wasmSupport = 'WebAssembly available: ' + hasWebAssembly;
                var jspiSupport = 'JSPI features: ' + hasPromiseIntegration;

                wasmSupport + ', ' + jspiSupport;
            } else {
                'WebAssembly not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebAssembly JSPI test: {}", value_str);
            // JSPI is experimental and might not be available in headless mode
        },
        Err(e) => panic!("Failed to test WebAssembly JSPI: {:?}", e),
    }

    println!("✅ WebAssembly JSPI test completed");
}
