use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_math_object() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Math object functionality
    let result = engine.execute_enhanced(r#"
        [
            Math.PI > 3.1,
            Math.max(1, 2, 3) === 3,
            Math.min(1, 2, 3) === 1,
            Math.abs(-5) === 5,
            Math.floor(3.7) === 3,
            Math.ceil(3.1) === 4,
            Math.round(3.5) === 4
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
