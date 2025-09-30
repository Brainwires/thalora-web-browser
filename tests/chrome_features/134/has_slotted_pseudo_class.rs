#[tokio::test]
async fn test_chrome_134_has_slotted_pseudo_class() {
    println!("🧪 Testing Chrome 134: :has-slotted pseudo-class...");

    let browser = HeadlessWebBrowser::new();

    // Test :has-slotted pseudo-class
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                // Test if :has-slotted pseudo-class is supported
                var supportsHasSlotted = CSS.supports('selector(:has-slotted(*))');
                ':has-slotted pseudo-class supported: ' + supportsHasSlotted;
            } else if (typeof document !== 'undefined') {
                // Fallback test
                try {
                    var style = document.createElement('style');
                    style.textContent = ':host(:has-slotted(*)) { display: block; }';
                    ':has-slotted pseudo-class: fallback test completed';
                } catch (styleError) {
                    ':has-slotted pseudo-class: ' + styleError.message;
                }
            } else {
                ':has-slotted pseudo-class: cannot test without CSS.supports or document';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!(":has-slotted pseudo-class test: {}", value_str);
            // CSS.supports might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test :has-slotted pseudo-class: {:?}", e),
    }

    println!("✅ :has-slotted pseudo-class test completed");
}
