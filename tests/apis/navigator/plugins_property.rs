#[tokio::test]
async fn test_navigator_plugins_exists() {
    use boa_engine::builtins::{IntrinsicObject, BuiltInConstructor};
    use thalora_browser_apis::browser::navigator::Navigator;

    let mut context = Context::default();
    let realm = context.realm().clone();

    // Manually initialize just the Navigator (minimal test)
    Navigator::init(&realm);

    // Create a navigator instance using the constructor
    let navigator_constructor = Navigator::get(context.intrinsics());
    let navigator_instance = Navigator::constructor(
        &navigator_constructor.clone().into(),
        &[],
        &mut context,
    ).expect("Failed to create navigator instance");

    // Set it as a global
    context.global_object().set(
        boa_engine::js_string!("navigator"),
        navigator_instance,
        false,
        &mut context
    ).expect("Failed to set global navigator");

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

    // Test navigator.plugins.length value (Chrome reports 5 PDF plugins)
    let result = context.eval(Source::from_bytes("navigator.plugins.length")).unwrap();
    let length_value = result.to_string(&mut context).unwrap().to_std_string_escaped();
    eprintln!("✅ navigator.plugins.length value: {}", length_value);
    assert_eq!(length_value, "5");
}
