use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_audio_context_constructor() {
    let mut context = Context::default();

    // Setup console and APIs
    thalora::apis::polyfills::console::setup_console(&mut context).unwrap();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).unwrap();

    // Test AudioContext constructor
    let result = context.eval(Source::from_bytes("new AudioContext()"));
    assert!(result.is_ok(), "AudioContext constructor should work");

    let value = result.unwrap();
    let string_result = value.to_string(&mut context).unwrap();
    assert!(!string_result.is_empty(), "AudioContext should return a valid object");
}
