#[tokio::test]
async fn test_audio_play_pause_methods() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test audio play and pause methods
    let result = context.eval(Source::from_bytes(r#"
        const audio = new Audio();
        typeof audio.play === "function" &&
        typeof audio.pause === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
    // Test play method returns Promise-like object
    let result = context.eval(Source::from_bytes(r#"
        const audio = new Audio();
        const playResult = audio.play();
        typeof playResult.then === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
