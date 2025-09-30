use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_observable_basic() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test basic Observable functionality
    let result = engine.execute_enhanced(r#"
        const values = [];
        const obs = Observable.of(1, 2, 3);
        obs.subscribe({
            next: value => values.push(value),
            complete: () => {}
        });

        [
            values.length === 3,
            values[0] === 1,
            values[2] === 3,
            typeof Observable === 'function',
            typeof Observable.of === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
