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
