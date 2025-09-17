#[tokio::test]
async fn test_speech_recognition_api() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test SpeechRecognition exists (both standard and webkit prefixed)
    let result = context.eval(Source::from_bytes(r#"
        typeof SpeechRecognition === "function" &&
        typeof webkitSpeechRecognition === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
    // Test SpeechRecognition can be instantiated
    let result = context.eval(Source::from_bytes(r#"
        const recognition = new SpeechRecognition();
        recognition.continuous === false &&
        recognition.interimResults === false &&
        recognition.lang === "en-US" &&
        recognition.maxAlternatives === 1 &&
        typeof recognition.start === "function" &&
        typeof recognition.stop === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
    // Test webkit prefixed version works the same
    let result = context.eval(Source::from_bytes(r#"
        const recognition = new webkitSpeechRecognition();
        recognition.continuous === false &&
        recognition.lang === "en-US";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
