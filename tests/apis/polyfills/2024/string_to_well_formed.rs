use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_string_to_well_formed() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String.prototype.toWellFormed
    let result = engine.execute_enhanced(r#"
        [
            'hello world'.toWellFormed(),
            'test string'.toWellFormed(),
            ''.toWellFormed(),
            typeof 'test'.toWellFormed === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
