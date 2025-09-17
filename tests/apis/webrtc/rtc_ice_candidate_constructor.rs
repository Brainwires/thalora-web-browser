#[tokio::test]
async fn test_rtc_ice_candidate_constructor() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test RTCIceCandidate exists
    let result = context.eval(Source::from_bytes("typeof RTCIceCandidate")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");
    // Test RTCIceCandidate can be instantiated with init object
    let result = context.eval(Source::from_bytes(r#"
        const candidate = new RTCIceCandidate({
            candidate: "candidate:1 1 UDP 2130706431 192.168.1.1 54400 typ host",
            sdpMid: "data",
            sdpMLineIndex: 0
        });
        candidate.candidate.includes("192.168.1.1") &&
        candidate.sdpMid === "data" &&
        candidate.sdpMLineIndex === 0;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
