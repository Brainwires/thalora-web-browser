use thalora::apis::WebApis;
use boa_engine::{Context, Source};

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
