use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_object_from_entries_edge_cases() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Object.fromEntries edge cases
    let result = engine.execute_enhanced(r#"
        [
            Object.fromEntries([]),
            Object.fromEntries([['key', undefined]]),
            Object.fromEntries([['0', 'zero'], ['1', 'one']]),
            typeof Object.fromEntries === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
