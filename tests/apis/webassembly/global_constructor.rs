use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_webassembly_global_constructor() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test WebAssembly.Global exists and can be instantiated
    let result = context.eval(Source::from_bytes(r#"
        const global = new WebAssembly.Global();
        typeof global.value;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "number");

    // Test global has valueOf method
    let result = context.eval(Source::from_bytes(r#"
        const global = new WebAssembly.Global();
        typeof global.valueOf;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");
}
