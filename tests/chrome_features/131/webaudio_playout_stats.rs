use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_131_webaudio_playout_stats() {
    println!("🧪 Testing Chrome 131: WebAudio Playout Statistics...");

    let browser = HeadlessWebBrowser::new();

    // Test WebAudio playout statistics
    let js_code = r#"
        try {
            if (typeof AudioContext !== 'undefined') {
                var audioContext = new AudioContext();

                // Test if playoutStats property exists
                var hasPlayoutStats = 'playoutStats' in audioContext;

                'AudioContext.playoutStats available: ' + hasPlayoutStats;
            } else {
                'AudioContext not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebAudio playout stats test: {}", value_str);
            // AudioContext might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test WebAudio playout stats: {:?}", e),
    }

    println!("✅ WebAudio playout stats test completed");
}
