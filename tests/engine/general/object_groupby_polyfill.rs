use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_object_groupby_polyfill() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Object.groupBy exists
    let result = engine.execute_enhanced(r#"
        typeof Object.groupBy === 'function' &&
        typeof Map.groupBy === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
