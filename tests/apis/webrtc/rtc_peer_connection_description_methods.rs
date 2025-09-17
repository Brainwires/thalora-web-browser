use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_rtc_peer_connection_description_methods() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test setLocalDescription and setRemoteDescription exist
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        typeof pc.setLocalDescription === "function" &&
        typeof pc.setRemoteDescription === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test they return Promise-like objects
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        const local = pc.setLocalDescription();
        const remote = pc.setRemoteDescription();
        typeof local.then === "function" && typeof remote.then === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
