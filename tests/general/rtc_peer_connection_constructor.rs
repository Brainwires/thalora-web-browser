use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_rtc_peer_connection_constructor() {
    let mut context = Context::default();

    // Setup console and APIs
    thalora::apis::polyfills::console::setup_console(&mut context).unwrap();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).unwrap();

    // Test RTCPeerConnection constructor
    let result = context.eval(Source::from_bytes("new RTCPeerConnection()"));
    assert!(result.is_ok(), "RTCPeerConnection constructor should work");

    let value = result.unwrap();
    let string_result = value.to_string(&mut context).unwrap();
    assert!(!string_result.is_empty(), "RTCPeerConnection should return a valid object");
}
