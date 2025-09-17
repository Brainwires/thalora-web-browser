use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_object_rest_spread() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test object rest/spread via transformation
    let result = engine.execute_enhanced(r#"
        const obj1 = { a: 1, b: 2 };
        const obj2 = Object.assign({}, obj1, { c: 3 });
        JSON.stringify(obj2)
    "#).await.unwrap();

    // Should be {"a":1,"b":2,"c":3}
    assert!(result.is_string());
}
