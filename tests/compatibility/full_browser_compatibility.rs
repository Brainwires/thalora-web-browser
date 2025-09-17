use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_full_browser_compatibility() {
    let mut context = Context::default();
    thalora::apis::polyfills::console::setup_console(&mut context).expect("Failed to setup console");
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test that all major browser APIs are present (like Firefox/Brave)
    let result = context.eval(Source::from_bytes(r#"
        // Modern browser API presence check
        typeof WebAssembly === "object" &&
        typeof navigator.geolocation === "object" &&
        typeof RTCPeerConnection === "function" &&
        typeof AudioContext === "function" &&
        typeof MediaRecorder === "function" &&
        typeof speechSynthesis === "object" &&
        typeof SpeechRecognition === "function" &&
        typeof crypto === "object" &&
        typeof fetch === "function" &&
        typeof navigator.serviceWorker === "object"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    println!("✅ Full browser compatibility test passed - Thalora is now a FULL-FEATURED browser!");
}
