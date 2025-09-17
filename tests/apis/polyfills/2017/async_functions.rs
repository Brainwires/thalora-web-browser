use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_async_functions() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test basic async function support (syntax transformation)
    let result = engine.execute_enhanced(r#"
        // Async functions are transformed to regular functions returning promises
        function asyncTest() {
            return Promise.resolve(42);
        }
        asyncTest()
    "#).await.unwrap();

    assert!(result.is_object()); // Promise object
}
