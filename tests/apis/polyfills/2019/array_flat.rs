use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_flat() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.prototype.flat
    let result = engine.execute_enhanced(r#"
        const arr1 = [1, 2, [3, 4]];
        const arr2 = [1, 2, [3, 4, [5, 6]]];
        [
            JSON.stringify(arr1.flat()),
            JSON.stringify(arr2.flat()),
            JSON.stringify(arr2.flat(2))
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
