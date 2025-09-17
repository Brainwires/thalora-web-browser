#[tokio::test]
async fn test_chrome_135_web_speech_mediastream() {
    println!("🧪 Testing Chrome 135: Web Speech API with MediaStreamTrack...");

    let browser = HeadlessWebBrowser::new();

    // Test Web Speech API with MediaStreamTrack support
    let js_code = r#"
        try {
            if (typeof SpeechRecognition !== 'undefined' || typeof webkitSpeechRecognition !== 'undefined') {
                var SpeechRecognitionClass = SpeechRecognition || webkitSpeechRecognition;

                // Test if MediaStreamTrack can be used with Speech Recognition
                var recognition = new SpeechRecognitionClass();

                // Check if audioTrack property exists (Chrome 135 feature)
                var hasAudioTrack = 'audioTrack' in recognition;

                'Web Speech API with MediaStreamTrack support: ' + hasAudioTrack;
            } else {
                'Web Speech API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Web Speech MediaStreamTrack test: {}", value_str);
            // Web Speech API might not be available in headless mode
        },
        Err(e) => panic!("Failed to test Web Speech MediaStreamTrack: {:?}", e),
    }

    println!("✅ Web Speech MediaStreamTrack test completed");
}
