#[tokio::test]
async fn test_rtc_peer_connection_states() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test initial connection states
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        pc.connectionState === "new" &&
        pc.iceConnectionState === "new" &&
        pc.iceGatheringState === "new" &&
        pc.signalingState === "stable";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
