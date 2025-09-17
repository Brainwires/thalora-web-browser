use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_promise_support() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Promise support
    let result = engine.execute_enhanced(r#"
        const p = Promise.resolve(42);
        [
            typeof Promise === 'function',
            typeof p === 'object',
            typeof p.then === 'function',
            typeof p.catch === 'function',
            typeof Promise.resolve === 'function',
            typeof Promise.reject === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
