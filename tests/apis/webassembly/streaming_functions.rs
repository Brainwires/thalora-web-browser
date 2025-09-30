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
