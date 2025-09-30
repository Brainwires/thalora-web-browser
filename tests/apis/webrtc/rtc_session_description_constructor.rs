#[tokio::test]
async fn test_rtc_session_description_constructor() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test RTCSessionDescription exists
    let result = context.eval(Source::from_bytes("typeof RTCSessionDescription")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");
    // Test RTCSessionDescription can be instantiated with init object
    let result = context.eval(Source::from_bytes(r#"
        const desc = new RTCSessionDescription({
            type: "offer",
            sdp: "v=0\r\no=- 123 456 IN IP4 127.0.0.1\r\n"
        });
        desc.type === "offer" && desc.sdp.includes("v=0");
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
