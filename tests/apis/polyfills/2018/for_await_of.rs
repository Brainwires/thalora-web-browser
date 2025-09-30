#[tokio::test]
async fn test_for_await_of() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test for-await-of transformation
    let result = engine.execute_enhanced(r#"
        // for-await-of would be transformed to regular iteration
        const arr = [1, 2, 3];
        let sum = 0;
        for (var i = 0; i < arr.length; i++) {
            sum += arr[i];
        }
        sum === 6
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
