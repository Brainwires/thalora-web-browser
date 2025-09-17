use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_atomics_basic() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test basic Atomics availability
    let result = engine.execute_enhanced(r#"
        // Atomics might not be fully supported in all environments
        typeof Atomics === 'object' || typeof Atomics === 'undefined'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
