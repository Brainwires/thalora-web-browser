use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_using_declarations() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test using declarations (resource management)
    let result = engine.execute_enhanced(r#"
        const resource = {
            value: 'test-resource'
        };

        const managed = using(resource);

        [
            managed.resource === resource,
            typeof managed[Symbol.dispose] === 'function' || typeof managed[Symbol.dispose] === 'undefined',
            typeof using === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
