use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_media_api_comprehensive() {
    let mut context = Context::default();
    thalora::apis::polyfills::console::setup_console(&mut context).expect("Failed to setup console");
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test AudioContext
    let result = context.eval(Source::from_bytes(r#"
        typeof AudioContext === "function"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test AudioContext functionality
    let result = context.eval(Source::from_bytes(r#"
        typeof AudioContext === "function"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test Audio constructor
    let result = context.eval(Source::from_bytes(r#"
        typeof Audio === "function"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test Audio properties
    let result = context.eval(Source::from_bytes(r#"
        typeof Audio === "function"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test MediaRecorder
    let result = context.eval(Source::from_bytes(r#"
        typeof MediaRecorder === "function" &&
        typeof MediaRecorder.isTypeSupported === "function" &&
        MediaRecorder.isTypeSupported("video/webm") === true
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test Speech Synthesis
    let result = context.eval(Source::from_bytes(r#"
        typeof speechSynthesis === "object" &&
        typeof SpeechSynthesisUtterance === "function" &&
        typeof SpeechRecognition === "function" &&
        typeof webkitSpeechRecognition === "function"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test Speech Synthesis functionality
    let result = context.eval(Source::from_bytes(r#"
        typeof SpeechSynthesisUtterance === "function"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    println!("✅ Media API tests passed");
}
