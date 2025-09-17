use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_media_recorder_event_handlers() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test MediaRecorder event handlers
    let result = context.eval(Source::from_bytes(r#"
        const recorder = new MediaRecorder();
        recorder.ondataavailable === null &&
        recorder.onstop === null &&
        recorder.onstart === null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
