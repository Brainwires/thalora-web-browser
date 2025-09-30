#[tokio::test]
async fn test_audio_context_create_oscillator() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test createOscillator method
    let result = context.eval(Source::from_bytes(r#"
        const ctx = new AudioContext();
        const osc = ctx.createOscillator();
        osc.type === "sine" &&
        typeof osc.frequency === "object" &&
        typeof osc.start === "function" &&
        typeof osc.stop === "function" &&
        typeof osc.connect === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
