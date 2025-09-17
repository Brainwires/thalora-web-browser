#[tokio::test]
async fn test_rtc_peer_connection_ice_candidates() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test addIceCandidate method exists
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        typeof pc.addIceCandidate === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
    // Test addIceCandidate returns Promise-like object
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        const result = pc.addIceCandidate();
        typeof result.then === "function" && typeof result.catch === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
