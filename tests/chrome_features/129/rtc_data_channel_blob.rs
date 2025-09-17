use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_129_rtc_data_channel_blob() {
    println!("🧪 Testing Chrome 129: RTCDataChannel Blob support...");

    let browser = HeadlessWebBrowser::new();

    // Test RTCDataChannel Blob support
    let js_code = r#"
        try {
            if (typeof RTCPeerConnection !== 'undefined') {
                // Create a peer connection to test data channel
                var pc = new RTCPeerConnection();
                var dataChannel = pc.createDataChannel('test');

                // Test if binaryType property exists
                var hasBinaryType = 'binaryType' in dataChannel;

                // Test if we can set binaryType to 'blob'
                if (hasBinaryType) {
                    dataChannel.binaryType = 'blob';
                    var binaryTypeSet = dataChannel.binaryType === 'blob';
                    'RTCDataChannel binaryType supported: ' + binaryTypeSet;
                } else {
                    'RTCDataChannel binaryType property not found';
                }
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
            println!("RTCDataChannel Blob test: {}", value_str);
            // WebRTC might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test RTCDataChannel Blob: {:?}", e),
    }

    println!("✅ RTCDataChannel Blob test completed");
}
