#[tokio::test]
async fn test_speech_synthesis_api() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test speechSynthesis global exists
    let result = context.eval(Source::from_bytes("typeof speechSynthesis")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "object");
    // Test speechSynthesis properties and methods
    let result = context.eval(Source::from_bytes(r#"
        speechSynthesis.speaking === false &&
        speechSynthesis.pending === false &&
        speechSynthesis.paused === false &&
        typeof speechSynthesis.speak === "function" &&
        typeof speechSynthesis.cancel === "function" &&
        typeof speechSynthesis.getVoices === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
    // Test getVoices returns array-like object
    let result = context.eval(Source::from_bytes(r#"
        const voices = speechSynthesis.getVoices();
        typeof voices === "object" && voices.length === 0;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
