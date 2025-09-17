use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_to_reversed() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.prototype.toReversed
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3];
        const reversed = arr.toReversed();
        [
            JSON.stringify(arr),
            JSON.stringify(reversed),
            arr !== reversed
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
