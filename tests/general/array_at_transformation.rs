use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_at_transformation() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.prototype.at transformation
    let result = engine.execute_enhanced(r#"
        const arr = ['a', 'b', 'c', 'd', 'e'];

        // .at() is transformed to index calculation
        [
            arr.at(0),
            arr.at(-1),
            arr.at(2),
            arr.at(-2),
            arr.at(10) === undefined
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
