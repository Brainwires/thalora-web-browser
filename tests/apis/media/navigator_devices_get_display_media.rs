#[tokio::test]
async fn test_navigator_media_devices_get_display_media() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test navigator.mediaDevices.getDisplayMedia exists
    let result = context.eval(Source::from_bytes("typeof navigator.mediaDevices.getDisplayMedia")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");
    // Test getDisplayMedia returns Promise-like object and works
    let result = context.eval(Source::from_bytes(r#"
        let streamReceived = false;
        let streamId = null;
        let isActive = false;
        const result = navigator.mediaDevices.getDisplayMedia();
        result.then(function(stream) {
            streamReceived = true;
            streamId = stream.id;
            isActive = stream.active;
        });
        typeof result.then === "function" &&
        streamReceived &&
        streamId === "screen-capture-stream" &&
        isActive === true;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
