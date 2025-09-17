use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_webassembly_memory_constructor() {
    let mut context = Context::default();

    // Setup console and APIs
    thalora::apis::polyfills::console::setup_console(&mut context).unwrap();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).unwrap();

    // Test WebAssembly.Memory constructor
    let result = context.eval(Source::from_bytes("new WebAssembly.Memory()"));
    assert!(result.is_ok(), "WebAssembly.Memory constructor should work");

    let value = result.unwrap();
    let string_result = value.to_string(&mut context).unwrap();
    assert!(!string_result.is_empty(), "WebAssembly.Memory should return a valid object");
}

#[tokio::test]
async fn test_audio_context_constructor() {
    let mut context = Context::default();

    // Setup console and APIs
    thalora::apis::polyfills::console::setup_console(&mut context).unwrap();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).unwrap();

    // Test AudioContext constructor
    let result = context.eval(Source::from_bytes("new AudioContext()"));
    assert!(result.is_ok(), "AudioContext constructor should work");

    let value = result.unwrap();
    let string_result = value.to_string(&mut context).unwrap();
    assert!(!string_result.is_empty(), "AudioContext should return a valid object");
}

#[tokio::test]
async fn test_rtc_peer_connection_constructor() {
    let mut context = Context::default();

    // Setup console and APIs
    thalora::apis::polyfills::console::setup_console(&mut context).unwrap();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).unwrap();

    // Test RTCPeerConnection constructor
    let result = context.eval(Source::from_bytes("new RTCPeerConnection()"));
    assert!(result.is_ok(), "RTCPeerConnection constructor should work");

    let value = result.unwrap();
    let string_result = value.to_string(&mut context).unwrap();
    assert!(!string_result.is_empty(), "RTCPeerConnection should return a valid object");
}

#[tokio::test]
async fn test_all_api_constructors_comprehensive() {
    let mut context = Context::default();

    // Setup console and APIs
    thalora::apis::polyfills::console::setup_console(&mut context).unwrap();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).unwrap();

    // Test multiple constructors in sequence
    let constructors = [
        "new WebAssembly.Memory()",
        "new AudioContext()",
        "new RTCPeerConnection()",
    ];

    for constructor in &constructors {
        let result = context.eval(Source::from_bytes(constructor));
        assert!(result.is_ok(), "Constructor {} should work", constructor);

        let value = result.unwrap();
        let string_result = value.to_string(&mut context).unwrap();
        assert!(!string_result.is_empty(), "Constructor {} should return a valid object", constructor);
    }
}