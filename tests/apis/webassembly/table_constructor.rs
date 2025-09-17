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
