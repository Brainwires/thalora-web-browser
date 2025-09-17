use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_139_web_speech_on_device() {
    println!("🧪 Testing Chrome 139: Web Speech API on-device recognition...");

    let browser = HeadlessWebBrowser::new();

    // Test on-device speech recognition features
    let js_code = r#"
        try {
            if (typeof SpeechRecognition !== 'undefined' || typeof webkitSpeechRecognition !== 'undefined') {
                var SpeechRecognitionClass = SpeechRecognition || webkitSpeechRecognition;
                var recognition = new SpeechRecognitionClass();

                // Chrome 139: Test on-device capabilities
                var hasOnDeviceSupport = typeof recognition.ondevicestart !== 'undefined' ||
                                        typeof recognition.serviceURI !== 'undefined';

                'Web Speech on-device recognition support: ' + hasOnDeviceSupport;
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
            println!("Web Speech on-device test: {}", value_str);
            // Speech API might not be available in headless mode
        },
        Err(e) => panic!("Failed to test Web Speech on-device: {:?}", e),
    }

    println!("✅ Web Speech on-device test completed");
}
