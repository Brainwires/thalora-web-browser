#[tokio::test]
async fn test_rtc_peer_connection_data_channel() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test createDataChannel method exists
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        typeof pc.createDataChannel === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
    // Test createDataChannel returns data channel object
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        const dc = pc.createDataChannel("test");
        dc.label === "test" &&
        dc.readyState === "connecting" &&
        dc.bufferedAmount === 0 &&
        typeof dc.send === "function" &&
        typeof dc.close === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
    // Test data channel event handlers
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        const dc = pc.createDataChannel("test");
        dc.onopen === null &&
        dc.onclose === null &&
        dc.onmessage === null &&
        dc.onerror === null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
