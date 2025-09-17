use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_function_partial_application() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Function.prototype.partial exists
    let result = engine.execute_enhanced(r#"
        function test() { return true; }
        typeof test.partial === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
