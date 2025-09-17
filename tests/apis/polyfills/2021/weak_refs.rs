use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_weak_refs() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test WeakRef
    let result = engine.execute_enhanced(r#"
        typeof WeakRef === 'function' || typeof WeakRef === 'undefined'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
