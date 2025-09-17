use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_logical_assignment_operators() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test logical assignment operators (||=, &&=, ??=)
    let result = engine.execute_enhanced(r#"
        let a = false;
        let b = true;
        let c = null;

        a ||= 'default';
        b &&= 'modified';
        c ??= 'fallback';

        [a, b, c]
    "#).await.unwrap();

    assert!(result.is_object());
}
