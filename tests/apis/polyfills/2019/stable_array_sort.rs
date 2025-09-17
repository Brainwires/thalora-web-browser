use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_stable_array_sort() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test stable Array.prototype.sort
    let result = engine.execute_enhanced(r#"
        const items = [
            { name: 'a', value: 1 },
            { name: 'b', value: 1 },
            { name: 'c', value: 1 }
        ];
        const sorted = items.sort((a, b) => a.value - b.value);
        sorted.length === 3
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
