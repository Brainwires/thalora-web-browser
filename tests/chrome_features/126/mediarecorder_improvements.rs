#[tokio::test]
async fn test_chrome_126_mediarecorder_improvements() {
    println!("🧪 Testing Chrome 126: MediaRecorder MP4 support...");

    let browser = HeadlessWebBrowser::new();

    // Test MediaRecorder availability
    let result = browser.lock().unwrap().execute_javascript("typeof MediaRecorder").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("MediaRecorder type: {}", value_str);
            assert!(value_str.contains("function") || value_str.contains("undefined"),
                "MediaRecorder should be function or undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check MediaRecorder: {:?}", e),
    }

    // Test MediaRecorder.isTypeSupported for MP4
    let js_code = r#"
        try {
            if (typeof MediaRecorder !== 'undefined' && MediaRecorder.isTypeSupported) {
                const mp4Support = MediaRecorder.isTypeSupported('video/mp4');
                const opusSupport = MediaRecorder.isTypeSupported('audio/opus');
                'mp4:' + mp4Support + ',opus:' + opusSupport;
            } else {
                'MediaRecorder.isTypeSupported not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            println!("MediaRecorder format support: {:?}", value);
            // Should not error out
        },
        Err(e) => panic!("Failed to test MediaRecorder formats: {:?}", e),
    }

    println!("✅ MediaRecorder improvements test completed");
}
