use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_object_has_own_edge_cases() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Object.hasOwn edge cases
    let result = engine.execute_enhanced(r#"
        [
            Object.hasOwn({}, 'missing'),
            Object.hasOwn({ 0: 'zero' }, '0'),
            Object.hasOwn({ null: 'value' }, 'null'),
            typeof Object.hasOwn === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
