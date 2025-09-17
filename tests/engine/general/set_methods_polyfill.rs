use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_set_methods_polyfill() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Set methods exist
    let result = engine.execute_enhanced(r#"
        const set = new Set([1, 2, 3]);
        typeof set.intersection === 'function' &&
        typeof set.union === 'function' &&
        typeof set.difference === 'function' &&
        typeof set.isSubsetOf === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
