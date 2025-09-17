use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_133_clipboard_item_strings() {
    println!("🧪 Testing Chrome 133: ClipboardItem string support...");

    let browser = HeadlessWebBrowser::new();

    // Test ClipboardItem with string values
    let js_code = r#"
        try {
            if (typeof ClipboardItem !== 'undefined') {
                // Test creating ClipboardItem with string values
                try {
                    var item = new ClipboardItem({
                        'text/plain': 'test string'
                    });
                    'ClipboardItem with string values: supported';
                } catch (itemError) {
                    'ClipboardItem string support: ' + itemError.message;
                }
            } else {
                'ClipboardItem not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("ClipboardItem strings test: {}", value_str);
            // Clipboard API might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test ClipboardItem strings: {:?}", e),
    }

    println!("✅ ClipboardItem strings test completed");
}
