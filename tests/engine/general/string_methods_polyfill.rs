use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_string_methods_polyfill() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String methods exist
    let result = engine.execute_enhanced(r#"
        const str = "hello";
        typeof str.isWellFormed === 'function' &&
        typeof str.toWellFormed === 'function' &&
        typeof str.dedent === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
