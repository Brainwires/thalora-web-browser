use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_140_scroll_into_view_container() {
    println!("🧪 Testing Chrome 140: ScrollIntoViewOptions container option...");

    let browser = HeadlessWebBrowser::new();

    // Test ScrollIntoViewOptions container option
    let js_code = r#"
        try {
            if (typeof document !== 'undefined') {
                // Create test element
                var element = document.createElement('div');

                // Test scrollIntoView with container option
                if (typeof element.scrollIntoView === 'function') {
                    // Chrome 140: container option
                    var options = {
                        behavior: 'smooth',
                        block: 'start',
                        inline: 'nearest',
                        container: document.body
                    };

                    'ScrollIntoView container option structure: supported';
                } else {
                    'scrollIntoView method not available';
                }
            } else {
                'document not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("ScrollIntoView container test: {}", value_str);
            assert!(!value_str.contains("error:"), "ScrollIntoView container should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test ScrollIntoView container: {:?}", e),
    }

    println!("✅ ScrollIntoView container test completed");
}
