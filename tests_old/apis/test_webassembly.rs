use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_webassembly_global_object() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    let result = context.eval(Source::from_bytes("typeof WebAssembly")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "object");
}

#[tokio::test]
async fn test_webassembly_module_constructor() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test WebAssembly.Module exists
    let result = context.eval(Source::from_bytes("typeof WebAssembly.Module")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");

    // Test WebAssembly.Module constructor requires bytes
    let result = context.eval(Source::from_bytes(r#"
        try {
            new WebAssembly.Module();
            "no_error";
        } catch (e) {
            e.message;
        }
    "#)).unwrap();
    let error_msg = result.to_string(&mut context).unwrap().to_std_string_escaped();
    assert!(error_msg.contains("requires bytes"));
}

#[tokio::test]
async fn test_webassembly_instance_constructor() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test WebAssembly.Instance exists
    let result = context.eval(Source::from_bytes("typeof WebAssembly.Instance")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");

    // Test WebAssembly.Instance constructor requires module
    let result = context.eval(Source::from_bytes(r#"
        try {
            new WebAssembly.Instance();
            "no_error";
        } catch (e) {
            e.message;
        }
    "#)).unwrap();
    let error_msg = result.to_string(&mut context).unwrap().to_std_string_escaped();
    assert!(error_msg.contains("requires a module"));
}

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

#[tokio::test]
async fn test_webassembly_table_constructor() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test WebAssembly.Table exists and can be instantiated
    let result = context.eval(Source::from_bytes(r#"
        const table = new WebAssembly.Table();
        table.length;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "0");

    // Test table methods exist
    let result = context.eval(Source::from_bytes(r#"
        const table = new WebAssembly.Table();
        typeof table.get === "function" && typeof table.set === "function" && typeof table.grow === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

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

#[tokio::test]
async fn test_webassembly_compile_function() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test WebAssembly.compile exists and returns Promise-like object
    let result = context.eval(Source::from_bytes(r#"
        const result = WebAssembly.compile();
        typeof result.then === "function" && typeof result.catch === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

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

#[tokio::test]
async fn test_webassembly_validate_function() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test WebAssembly.validate exists and returns true (mock behavior)
    let result = context.eval(Source::from_bytes("WebAssembly.validate()")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_webassembly_streaming_functions() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test WebAssembly.instantiateStreaming exists
    let result = context.eval(Source::from_bytes("typeof WebAssembly.instantiateStreaming")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");

    // Test WebAssembly.compileStreaming exists
    let result = context.eval(Source::from_bytes("typeof WebAssembly.compileStreaming")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");

    // Test they return Promise-like objects
    let result = context.eval(Source::from_bytes(r#"
        const compile = WebAssembly.compileStreaming();
        const instantiate = WebAssembly.instantiateStreaming();
        typeof compile.then === "function" && typeof instantiate.then === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}