#[tokio::test]
async fn test_navigator_plugins_exists() {
    let mut context = Context::default();

    // Use the standard browser API initialization which creates the navigator instance
    thalora_browser_apis::initialize_browser_apis(&mut context)
        .expect("Failed to initialize browser APIs");

    // Test navigator exists
    let result = context.eval(Source::from_bytes("typeof navigator")).unwrap();
    let nav_type = result.to_string(&mut context).unwrap().to_std_string_escaped();
    eprintln!("✅ navigator type: {}", nav_type);
    assert_eq!(nav_type, "object");

    // Check navigator prototype properties
    let result = context.eval(Source::from_bytes("Object.getOwnPropertyNames(Object.getPrototypeOf(navigator)).join(', ')")).unwrap();
    let proto_props = result.to_string(&mut context).unwrap().to_std_string_escaped();
    eprintln!("✅ Navigator prototype properties: {}", proto_props);

    // Check if plugins is in navigator's prototype
    let result = context.eval(Source::from_bytes("'plugins' in navigator")).unwrap();
    let has_plugins = result.to_string(&mut context).unwrap().to_std_string_escaped();
    eprintln!("✅ 'plugins' in navigator: {}", has_plugins);

    // Test navigator.plugins exists and is an object
    let result = context.eval(Source::from_bytes("typeof navigator.plugins")).unwrap();
    let plugins_type = result.to_string(&mut context).unwrap().to_std_string_escaped();
    eprintln!("✅ navigator.plugins type: {}", plugins_type);
    assert_eq!(plugins_type, "object");

    // Test navigator.plugins.length exists and is a number
    let result = context.eval(Source::from_bytes("typeof navigator.plugins.length")).unwrap();
    let length_type = result.to_string(&mut context).unwrap().to_std_string_escaped();
    eprintln!("✅ navigator.plugins.length type: {}", length_type);
    assert_eq!(length_type, "number");

    // Test navigator.plugins.length value
    let result = context.eval(Source::from_bytes("navigator.plugins.length")).unwrap();
    let length_value = result.to_string(&mut context).unwrap().to_std_string_escaped();
    eprintln!("✅ navigator.plugins.length value: {}", length_value);
    assert_eq!(length_value, "0");  // Empty array for security
}
