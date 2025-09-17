use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_async_context_nesting() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test AsyncContext nesting
    let result = engine.execute_enhanced(r#"
        const context = new AsyncContext('nested-test');

        const result = context.run('outer', () => {
            const outer = context.get();
            const inner = context.run('inner', () => {
                return context.get();
            });
            const restored = context.get();

            return [outer, inner, restored];
        });

        [
            result[0] === 'outer',
            result[1] === 'inner',
            result[2] === 'outer'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
