use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_object_get_own_property_descriptors() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Object.getOwnPropertyDescriptors
    let result = engine.execute_enhanced(r#"
        const obj = { a: 1 };
        const descriptors = Object.getOwnPropertyDescriptors(obj);
        typeof descriptors === 'object' && descriptors.a !== undefined
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
