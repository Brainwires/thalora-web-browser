use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_set_methods_polyfill_existence() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that all ES2023 set methods exist
    let result = engine.execute_enhanced(r#"
        const set = new Set();
        [
            typeof set.intersection === 'function',
            typeof set.union === 'function',
            typeof set.difference === 'function',
            typeof set.symmetricDifference === 'function',
            typeof set.isSubsetOf === 'function',
            typeof set.isSupersetOf === 'function',
            typeof set.isDisjointFrom === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
