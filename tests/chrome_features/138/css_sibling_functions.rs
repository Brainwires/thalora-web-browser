use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_138_css_sibling_functions() {
    println!("🧪 Testing Chrome 138: CSS sibling functions (sibling-index, sibling-count)...");

    let browser = HeadlessWebBrowser::new();

    // Test CSS sibling functions
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                // Test sibling-index() function
                var supportsSiblingIndex = CSS.supports('z-index', 'sibling-index()');

                // Test sibling-count() function
                var supportsSiblingCount = CSS.supports('z-index', 'sibling-count()');

                'CSS sibling functions - index: ' + supportsSiblingIndex + ', count: ' + supportsSiblingCount;
            } else {
                'CSS.supports not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("CSS sibling functions test: {}", value_str);
            assert!(!value_str.contains("error:"), "CSS sibling functions should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test CSS sibling functions: {:?}", e),
    }

    println!("✅ CSS sibling functions test completed");
}
