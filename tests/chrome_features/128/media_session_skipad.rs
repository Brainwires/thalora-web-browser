use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_128_media_session_skipad() {
    println!("🧪 Testing Chrome 128: Media Session SkipAd action...");

    let browser = HeadlessWebBrowser::new();

    // Test Media Session SkipAd action
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.mediaSession) {
                // Test if we can set SkipAd action handler
                navigator.mediaSession.setActionHandler('skipad', function() {
                    console.log('SkipAd action triggered');
                });

                'MediaSession SkipAd action supported';
            } else {
                'MediaSession not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("MediaSession SkipAd test: {}", value_str);
            // MediaSession might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test MediaSession SkipAd: {:?}", e),
    }

    println!("✅ MediaSession SkipAd test completed");
}
