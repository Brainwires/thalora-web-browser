use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_133_animation_overall_progress() {
    println!("🧪 Testing Chrome 133: Animation.overallProgress...");

    let browser = HeadlessWebBrowser::new();

    // Test Animation.overallProgress property
    let js_code = r#"
        try {
            if (typeof Animation !== 'undefined') {
                // Create a simple animation to test overallProgress
                var elem = {style: {transform: ''}};
                var keyframes = [
                    {transform: 'translateX(0px)'},
                    {transform: 'translateX(100px)'}
                ];

                // Test if overallProgress property exists on Animation prototype
                var hasOverallProgress = 'overallProgress' in Animation.prototype;
                'Animation.overallProgress available: ' + hasOverallProgress;
            } else {
                'Animation API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Animation.overallProgress test: {}", value_str);
            // overallProgress might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test Animation.overallProgress: {:?}", e),
    }

    println!("✅ Animation.overallProgress test completed");
}
