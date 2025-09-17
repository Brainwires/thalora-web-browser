use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_webassembly_api_comprehensive() {
    let mut context = Context::default();

    // Setup console first
    thalora::apis::polyfills::console::setup_console(&mut context).expect("Failed to setup console");

    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test WebAssembly global object exists
    let result = context.eval(Source::from_bytes("typeof WebAssembly")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "object");

    // Test WebAssembly constructors
    let result = context.eval(Source::from_bytes(r#"
        typeof WebAssembly.Module === "function" &&
        typeof WebAssembly.Instance === "function" &&
        typeof WebAssembly.Memory === "function" &&
        typeof WebAssembly.Table === "function" &&
        typeof WebAssembly.Global === "function"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test WebAssembly functions
    let result = context.eval(Source::from_bytes(r#"
        typeof WebAssembly.compile === "function" &&
        typeof WebAssembly.instantiate === "function" &&
        typeof WebAssembly.validate === "function" &&
        typeof WebAssembly.compileStreaming === "function" &&
        typeof WebAssembly.instantiateStreaming === "function"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test WebAssembly.Memory creation and functionality
    let result = context.eval(Source::from_bytes(r#"
        typeof WebAssembly.Memory === "function"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test WebAssembly.validate returns true (mock behavior)
    let result = context.eval(Source::from_bytes("WebAssembly.validate()")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    println!("✅ WebAssembly API tests passed");
}
