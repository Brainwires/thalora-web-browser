use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_media_recorder_is_type_supported() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test MediaRecorder.isTypeSupported static method
    let result = context.eval(Source::from_bytes(r#"
        typeof MediaRecorder.isTypeSupported === "function" &&
        MediaRecorder.isTypeSupported("video/webm") === true &&
        MediaRecorder.isTypeSupported("video/mp4") === true;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
