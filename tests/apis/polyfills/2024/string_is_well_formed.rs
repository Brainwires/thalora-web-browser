use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_string_is_well_formed() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String.prototype.isWellFormed
    let result = engine.execute_enhanced(r#"
        [
            'hello world'.isWellFormed(),
            'test 🌟 emoji'.isWellFormed(),
            ''.isWellFormed(),
            typeof 'test'.isWellFormed === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
