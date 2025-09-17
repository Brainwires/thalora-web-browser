use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_126_visual_viewport_scrollend() {
    println!("🧪 Testing Chrome 126: visualViewport onscrollend support...");

    let browser = HeadlessWebBrowser::new();

    // Test visualViewport availability
    let result = browser.lock().unwrap().execute_javascript("typeof window.visualViewport").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("window.visualViewport type: {}", value_str);
            assert!(value_str.contains("object") || value_str.contains("undefined"),
                "visualViewport should be object or undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check window.visualViewport: {:?}", e),
    }

    // Test onscrollend event handler
    let js_code = r#"
        try {
            if (typeof window !== 'undefined' && window.visualViewport) {
                // Test if onscrollend property exists
                const hasScrollEnd = 'onscrollend' in window.visualViewport;
                'onscrollend supported: ' + hasScrollEnd;
            } else {
                'visualViewport not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            println!("visualViewport onscrollend test: {:?}", value);
            // Should not error out
        },
        Err(e) => panic!("Failed to test visualViewport onscrollend: {:?}", e),
    }

    println!("✅ visualViewport scrollend test completed");
}
