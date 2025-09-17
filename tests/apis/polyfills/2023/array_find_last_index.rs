use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_find_last_index() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.prototype.findLastIndex
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3, 4, 5];
        [
            arr.findLastIndex(x => x > 3),
            arr.findLastIndex(x => x > 10),
            arr.findLastIndex(x => x % 2 === 0),
            [].findLastIndex(x => true)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
