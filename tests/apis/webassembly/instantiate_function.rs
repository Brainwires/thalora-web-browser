use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_webassembly_instantiate_function() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test WebAssembly.instantiate exists and returns Promise-like object
    let result = context.eval(Source::from_bytes(r#"
        const result = WebAssembly.instantiate();
        typeof result.then === "function" && typeof result.catch === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
