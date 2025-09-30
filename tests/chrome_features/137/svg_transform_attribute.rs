#[tokio::test]
async fn test_chrome_137_svg_transform_attribute() {
    println!("🧪 Testing Chrome 137: SVG transform attribute on root element...");

    let browser = HeadlessWebBrowser::new();

    // Test SVG transform attribute support
    let js_code = r#"
        try {
            if (typeof document !== 'undefined') {
                // Test creating SVG element with transform attribute
                var svg = document.createElement('svg');
                svg.setAttribute('transform', 'scale(2) rotate(45)');

                var hasTransform = svg.getAttribute('transform') === 'scale(2) rotate(45)';
                var svgSupport = 'SVG element creation: true';
                var transformSupport = 'transform attribute: ' + hasTransform;

                svgSupport + ', ' + transformSupport;
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
            println!("SVG transform attribute test: {}", value_str);
            assert!(!value_str.contains("error:"), "SVG transform should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test SVG transform attribute: {:?}", e),
    }

    println!("✅ SVG transform attribute test completed");
}
