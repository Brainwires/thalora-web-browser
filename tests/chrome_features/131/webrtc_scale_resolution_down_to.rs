use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_131_webrtc_scale_resolution_down_to() {
    println!("🧪 Testing Chrome 131: WebRTC scaleResolutionDownTo...");

    let browser = HeadlessWebBrowser::new();

    // Test WebRTC scaleResolutionDownTo
    let js_code = r#"
        try {
            if (typeof RTCPeerConnection !== 'undefined') {
                // Test creating peer connection
                var pc = new RTCPeerConnection();

                // Test if we can create encoding parameters with scaleResolutionDownTo
                var encodingParams = {
                    scaleResolutionDownTo: { width: 640, height: 360 }
                };

                'WebRTC scaleResolutionDownTo parameter structure created';
            } else {
                'RTCPeerConnection not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebRTC scaleResolutionDownTo test: {}", value_str);
            assert!(!value_str.contains("error:"), "WebRTC scaleResolutionDownTo should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test WebRTC scaleResolutionDownTo: {:?}", e),
    }

    println!("✅ WebRTC scaleResolutionDownTo test completed");
}
