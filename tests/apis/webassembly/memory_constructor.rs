use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_webassembly_memory_constructor() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test WebAssembly.Memory exists and can be instantiated
    let result = context.eval(Source::from_bytes(r#"
        const memory = new WebAssembly.Memory();
        typeof memory.buffer;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "object");

    // Test memory has grow method
    let result = context.eval(Source::from_bytes(r#"
        const memory = new WebAssembly.Memory();
        typeof memory.grow;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");
}
