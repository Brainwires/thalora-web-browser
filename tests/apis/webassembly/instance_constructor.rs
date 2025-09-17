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
