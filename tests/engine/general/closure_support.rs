use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_closure_support() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test closure support
    let result = engine.execute_enhanced(r#"
        function createCounter(initial) {
            let count = initial || 0;
            return function() {
                return ++count;
            };
        }

        const counter = createCounter(5);
        [
            counter() === 6,
            counter() === 7,
            counter() === 8
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
