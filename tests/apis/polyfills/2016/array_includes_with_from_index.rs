use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_includes_with_from_index() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test includes with fromIndex parameter
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3, 4, 5];
        [
            arr.includes(3, 2),    // true - found at index 2
            arr.includes(3, 3),    // false - start search from index 3
            arr.includes(1, -4),   // true - negative index
            arr.includes(5, -1)    // true - found at last position
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
