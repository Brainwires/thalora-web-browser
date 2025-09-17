use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_async_context() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test AsyncContext functionality
    let result = engine.execute_enhanced(r#"
        const context = new AsyncContext('test-context');

        const result1 = context.run('test-value', () => {
            return context.get();
        });

        const result2 = context.get(); // Should be undefined outside run

        [
            result1 === 'test-value',
            result2 === undefined,
            typeof context.run === 'function',
            typeof context.get === 'function',
            typeof AsyncContext === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
