use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_speech_synthesis_utterance() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test SpeechSynthesisUtterance constructor exists
    let result = context.eval(Source::from_bytes("typeof SpeechSynthesisUtterance")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");

    // Test SpeechSynthesisUtterance can be instantiated with text
    let result = context.eval(Source::from_bytes(r#"
        const utterance = new SpeechSynthesisUtterance("Hello, world!");
        utterance.text === "Hello, world!" &&
        utterance.lang === "en-US" &&
        utterance.volume === 1.0 &&
        utterance.rate === 1.0 &&
        utterance.pitch === 1.0;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test SpeechSynthesisUtterance without text
    let result = context.eval(Source::from_bytes(r#"
        const utterance = new SpeechSynthesisUtterance();
        utterance.text === "";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
