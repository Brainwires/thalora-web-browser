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
