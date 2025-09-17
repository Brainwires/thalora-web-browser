use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_audio_context_constructor() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test AudioContext exists
    let result = context.eval(Source::from_bytes("typeof AudioContext")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");

    // Test AudioContext can be instantiated
    let result = context.eval(Source::from_bytes(r#"
        const ctx = new AudioContext();
        ctx.state === "running" &&
        ctx.sampleRate === 44100 &&
        ctx.currentTime === 0 &&
        typeof ctx.destination === "object";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
