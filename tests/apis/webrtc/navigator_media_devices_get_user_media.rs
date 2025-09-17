use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_navigator_media_devices_get_user_media() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test navigator.mediaDevices.getUserMedia exists
    let result = context.eval(Source::from_bytes("typeof navigator.mediaDevices.getUserMedia")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");

    // Test getUserMedia returns Promise-like object
    let result = context.eval(Source::from_bytes(r#"
        const result = navigator.mediaDevices.getUserMedia();
        typeof result.then === "function" && typeof result.catch === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test getUserMedia callback receives MediaStream
    let result = context.eval(Source::from_bytes(r#"
        let streamReceived = false;
        let streamId = null;
        let isActive = false;

        navigator.mediaDevices.getUserMedia().then(function(stream) {
            streamReceived = true;
            streamId = stream.id;
            isActive = stream.active;
        });

        streamReceived && streamId === "mock-stream-id" && isActive === true;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
