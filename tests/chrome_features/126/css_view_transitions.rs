use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_126_css_view_transitions() {
    println!("🧪 Testing Chrome 126: Cross-Document View Transitions...");

    let browser = HeadlessWebBrowser::new();

    // Test CSS.supports for view transition features
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                const viewTransitionName = CSS.supports('view-transition-name', 'my-transition');
                const atViewTransition = CSS.supports('@view-transition', 'navigation: auto');
                'view-transition-name:' + viewTransitionName + ',@view-transition:' + atViewTransition;
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
            println!("CSS view transitions support: {}", value_str);
            assert!(!value_str.contains("error:"), "CSS view transitions test should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test CSS view transitions: {:?}", e),
    }

    println!("✅ CSS view transitions test completed");
}
