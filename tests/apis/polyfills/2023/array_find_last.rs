use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_find_last() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.prototype.findLast
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3, 4, 5];
        [
            arr.findLast(x => x > 3),
            arr.findLast(x => x > 10),
            arr.findLast(x => x % 2 === 0),
            [].findLast(x => true)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
