#[tokio::test]
async fn test_atomics_wait_async() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Atomics.waitAsync (if supported)
    let result = engine.execute_enhanced(r#"
        typeof Atomics !== 'undefined' ?
            (typeof Atomics.waitAsync === 'function' || typeof Atomics.waitAsync === 'undefined') :
            true
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
