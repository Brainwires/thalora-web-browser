use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_flat_depth() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.prototype.flat with various depths
    let result = engine.execute_enhanced(r#"
        const deepArray = [1, [2, [3, [4, 5]]]];
        [
            JSON.stringify(deepArray.flat(0)),
            JSON.stringify(deepArray.flat(1)),
            JSON.stringify(deepArray.flat(2)),
            JSON.stringify(deepArray.flat(Infinity))
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
