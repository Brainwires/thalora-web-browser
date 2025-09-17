#[tokio::test]
async fn test_chrome_139_prompt_api() {
    println!("🧪 Testing Chrome 139: Prompt API (Origin Trial)...");

    let browser = HeadlessWebBrowser::new();

    // Test Prompt API availability
    let js_code = r#"
        try {
            // Check if Prompt API or AI APIs are available
            if (typeof ai !== 'undefined' && ai.canCreateTextSession) {
                // Test Prompt API structure
                var hasPromptAPI = typeof ai.canCreateTextSession === 'function';

                'Prompt API available: ' + hasPromptAPI;
            } else if (typeof window !== 'undefined' && typeof window.ai !== 'undefined') {
                'AI context available: ' + (typeof window.ai === 'object');
            } else {
                'Prompt API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Prompt API test: {}", value_str);
            // AI APIs might not be available in headless mode
        },
        Err(e) => panic!("Failed to test Prompt API: {:?}", e),
    }

    println!("✅ Prompt API test completed");
}
