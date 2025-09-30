#[tokio::test]
async fn test_rtc_peer_connection_constructor() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test RTCPeerConnection exists
    let result = context.eval(Source::from_bytes("typeof RTCPeerConnection")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");
    // Test RTCPeerConnection can be instantiated
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        typeof pc === "object";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
