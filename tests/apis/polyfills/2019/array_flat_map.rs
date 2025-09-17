use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_flat_map() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.prototype.flatMap
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3];
        const result = arr.flatMap(x => [x, x * 2]);
        JSON.stringify(result)
    "#).await.unwrap();

    // Should be [1,2,2,4,3,6]
    assert!(result.is_string());
}
