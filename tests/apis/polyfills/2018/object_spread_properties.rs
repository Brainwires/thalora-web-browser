use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_object_spread_properties() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test object spread properties transformation
    let result = engine.execute_enhanced(r#"
        const obj1 = { a: 1, b: 2 };
        const obj2 = { c: 3, d: 4 };
        const combined = Object.assign({}, obj1, obj2, { e: 5 });
        combined.a === 1 && combined.c === 3 && combined.e === 5
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
