#[tokio::test]
async fn test_audio_constructor() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test Audio constructor exists
    let result = context.eval(Source::from_bytes("typeof Audio")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");
    // Test Audio can be instantiated with src
    let result = context.eval(Source::from_bytes(r#"
        const audio = new Audio("test.mp3");
        audio.src === "test.mp3" &&
        audio.currentTime === 0 &&
        audio.duration === 0 &&
        audio.paused === true &&
        audio.volume === 1.0 &&
        audio.muted === false;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
    // Test Audio without src
    let result = context.eval(Source::from_bytes(r#"
        const audio = new Audio();
        audio.src === "";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
