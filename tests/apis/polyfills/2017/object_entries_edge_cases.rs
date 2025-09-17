use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_object_entries_edge_cases() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Object.entries edge cases
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3];
        const entries = Object.entries(arr);
        // Array entries should be [['0', 1], ['1', 2], ['2', 3]]
        entries.length === 3 && entries[0][0] === '0'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
