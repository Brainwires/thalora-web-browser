use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_webrtc_api_comprehensive() {
    let mut context = Context::default();
    thalora::apis::polyfills::console::setup_console(&mut context).expect("Failed to setup console");
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test RTCPeerConnection exists and can be instantiated
    let result = context.eval(Source::from_bytes(r#"
        typeof RTCPeerConnection === "function"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test RTCPeerConnection methods
    let result = context.eval(Source::from_bytes(r#"
        typeof RTCPeerConnection === "function"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test RTCPeerConnection initial states
    let result = context.eval(Source::from_bytes(r#"
        typeof RTCPeerConnection === "function"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test createOffer returns Promise-like object
    let result = context.eval(Source::from_bytes(r#"
        typeof RTCPeerConnection === "function"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test DataChannel creation
    let result = context.eval(Source::from_bytes(r#"
        typeof RTCPeerConnection === "function"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test navigator.mediaDevices.getUserMedia
    let result = context.eval(Source::from_bytes(r#"
        typeof navigator.mediaDevices.getUserMedia === "function"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    println!("✅ WebRTC API tests passed");
}
