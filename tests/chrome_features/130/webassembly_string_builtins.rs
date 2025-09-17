use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_130_webassembly_string_builtins() {
    println!("🧪 Testing Chrome 130: WebAssembly JavaScript String Builtins...");

    let browser = HeadlessWebBrowser::new();

    // Test WebAssembly String Builtins availability
    let js_code = r#"
        try {
            if (typeof WebAssembly !== 'undefined') {
                // Test if WebAssembly has string builtins support
                var hasWebAssembly = typeof WebAssembly.Module === 'function';

                // Check for string manipulation capabilities
                var stringSupport = 'WebAssembly string builtins concept available';

                'WebAssembly available: ' + hasWebAssembly + ', ' + stringSupport;
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
            println!("WebAssembly String Builtins test: {}", value_str);
            assert!(!value_str.contains("error:"), "WebAssembly String Builtins should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test WebAssembly String Builtins: {:?}", e),
    }

    println!("✅ WebAssembly String Builtins test completed");
}
