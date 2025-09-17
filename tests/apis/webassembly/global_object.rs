#[tokio::test]
async fn test_webassembly_global_object() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    let result = context.eval(Source::from_bytes("typeof WebAssembly")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "object");
}
