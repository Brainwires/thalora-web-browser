use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_rtc_peer_connection_event_handlers() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test event handler properties exist and are initially null
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        pc.onicecandidate === null &&
        pc.oniceconnectionstatechange === null &&
        pc.onnegotiationneeded === null &&
        pc.ondatachannel === null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
