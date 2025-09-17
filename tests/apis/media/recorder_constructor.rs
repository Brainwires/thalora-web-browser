#[tokio::test]
async fn test_media_recorder_constructor() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test MediaRecorder exists
    let result = context.eval(Source::from_bytes("typeof MediaRecorder")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");
    // Test MediaRecorder can be instantiated
    let result = context.eval(Source::from_bytes(r#"
        const recorder = new MediaRecorder();
        recorder.state === "inactive" &&
        recorder.mimeType === "video/webm" &&
        typeof recorder.start === "function" &&
        typeof recorder.stop === "function" &&
        typeof recorder.pause === "function" &&
        typeof recorder.resume === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
