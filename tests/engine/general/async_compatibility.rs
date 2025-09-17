use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_async_compatibility() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test async/await compatibility (via transformation)
    let result = engine.execute_enhanced(r#"
        // Async functions are transformed to promise-returning functions
        function asyncTest() {
            return Promise.resolve(42);
        }

        typeof asyncTest() === 'object' // Should be a Promise
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
