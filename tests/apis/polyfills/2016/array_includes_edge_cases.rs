use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_includes_edge_cases() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test edge cases for includes
    let result = engine.execute_enhanced(r#"
        const arr = ['a', 'b', 'c'];
        [
            arr.includes('a'),
            arr.includes('d'),
            [].includes(undefined),
            [undefined].includes(undefined),
            [null].includes(null)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
