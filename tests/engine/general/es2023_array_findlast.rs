use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_es2023_array_findlast() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that findLast polyfill is loaded
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3, 4, 5];
        typeof arr.findLast === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
