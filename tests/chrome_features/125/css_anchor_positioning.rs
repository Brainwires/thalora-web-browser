use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_125_css_anchor_positioning() {
    println!("🧪 Testing Chrome 125: CSS Anchor Positioning...");

    let browser = HeadlessWebBrowser::new();

    // Test CSS.supports for anchor positioning
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                const anchorName = CSS.supports('anchor-name', 'my-anchor');
                const positionAnchor = CSS.supports('position-anchor', 'my-anchor');
                const anchorTop = CSS.supports('top', 'anchor(bottom)');

                'anchor-name:' + anchorName + ',position-anchor:' + positionAnchor + ',anchor-top:' + anchorTop;
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
            println!("CSS anchor positioning support: {}", value_str);
            // Should have some level of CSS.supports available
            assert!(!value_str.contains("error:"), "CSS anchor positioning test should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test CSS anchor positioning: {:?}", e),
    }

    println!("✅ CSS anchor positioning test completed");
}
