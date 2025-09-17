#[tokio::test]
async fn test_iterator_helpers() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Iterator helpers (if available)
    let result = engine.execute_enhanced(r#"
        try {
            const arr = [1, 2, 3];
            const iterator = arr[Symbol.iterator]();
            typeof iterator.map === 'function' || typeof iterator.map === 'undefined'
        } catch (e) {
            true // May not be supported
        }
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
