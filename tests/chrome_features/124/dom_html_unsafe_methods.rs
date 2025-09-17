use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_124_dom_html_unsafe_methods() {
    println!("🧪 Testing Chrome 124: DOM setHTMLUnsafe and parseHTMLUnsafe...");

    let browser = HeadlessWebBrowser::new();

    // Test Element.prototype.setHTMLUnsafe
    let result = browser.lock().unwrap().execute_javascript("typeof Element.prototype.setHTMLUnsafe").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("setHTMLUnsafe type: {}", value_str);
            assert!(value_str.contains("function"), "setHTMLUnsafe should be available, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check setHTMLUnsafe: {:?}", e),
    }

    // Test Document.parseHTMLUnsafe
    let result = browser.lock().unwrap().execute_javascript("typeof Document.parseHTMLUnsafe").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("parseHTMLUnsafe type: {}", value_str);
            assert!(value_str.contains("function"), "parseHTMLUnsafe should be available, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check parseHTMLUnsafe: {:?}", e),
    }

    println!("✅ DOM HTML unsafe methods test completed");
}
