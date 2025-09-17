use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_audio_context_create_gain() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test createGain method
    let result = context.eval(Source::from_bytes(r#"
        const ctx = new AudioContext();
        const gain = ctx.createGain();
        gain.gain.value === 1.0 &&
        typeof gain.connect === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
