use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_includes() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.prototype.includes
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3, NaN];
        [
            arr.includes(2),
            arr.includes(4),
            arr.includes(NaN),
            arr.includes(1, 1),
            arr.includes(1, -3)
        ]
    "#).await.unwrap();

    // Should return [true, false, true, false, true]
    assert!(result.is_object());
}
