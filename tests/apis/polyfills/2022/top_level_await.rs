use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_top_level_await() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test top-level await transformation
    let result = engine.execute_enhanced(r#"
        // Top-level await would be wrapped in async IIFE
        (async function() {
            const result = await Promise.resolve(42);
            return result;
        })()
    "#).await.unwrap();

    assert!(result.is_object()); // Should be a Promise
}
