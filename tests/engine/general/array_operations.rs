use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_array_operations() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test array operations
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3, 4, 5];
        [
            arr.length === 5,
            arr.push(6) === 6,
            arr.pop() === 6,
            arr.slice(1, 3).length === 2,
            arr.indexOf(3) === 2,
            Array.isArray(arr) === true
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
