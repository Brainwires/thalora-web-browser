use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_full_browser_compatibility() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
    thalora::apis::polyfills::console::setup_console(&mut context).expect("Failed to setup console");
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test that all major browser APIs are present (like Firefox/Brave)
    // Evaluate each check separately and print a JSON object so we can identify
    // which API is missing in case of failure.
    let result = context.eval(Source::from_bytes(r#"
        JSON.stringify({
            webassembly: typeof WebAssembly === "object",
            geolocation: typeof navigator.geolocation === "object",
            rtc: typeof RTCPeerConnection === "function",
            audio_context: typeof AudioContext === "function",
            media_recorder: typeof MediaRecorder === "function",
            speech_synthesis: typeof speechSynthesis === "object",
            speech_recognition: typeof SpeechRecognition === "function",
            crypto: typeof crypto === "object",
            fetch: typeof fetch === "function",
            service_worker: typeof navigator.serviceWorker === "object"
        })
    "#)).unwrap();

    let checks = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("Compatibility checks: {}", checks);

    // Ensure every property is true
    assert!(checks.contains("\"webassembly\":true") , "webassembly missing: {}", checks);
    assert!(checks.contains("\"geolocation\":true"), "geolocation missing: {}", checks);
    assert!(checks.contains("\"rtc\":true"), "rtc missing: {}", checks);
    assert!(checks.contains("\"audio_context\":true"), "audio_context missing: {}", checks);
    assert!(checks.contains("\"media_recorder\":true"), "media_recorder missing: {}", checks);
    assert!(checks.contains("\"speech_synthesis\":true"), "speech_synthesis missing: {}", checks);
    assert!(checks.contains("\"speech_recognition\":true"), "speech_recognition missing: {}", checks);
    assert!(checks.contains("\"crypto\":true"), "crypto missing: {}", checks);
    assert!(checks.contains("\"fetch\":true"), "fetch missing: {}", checks);
    assert!(checks.contains("\"service_worker\":true"), "service_worker missing: {}", checks);

    println!("✅ Full browser compatibility test passed - Thalora is now a FULL-FEATURED browser!");
}
