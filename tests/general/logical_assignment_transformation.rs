use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_logical_assignment_transformation() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test logical assignment operators transformation
    let result = engine.execute_enhanced(r#"
        let a = false;
        let b = true;
        let c = null;
        let d = 'existing';

        a ||= 'fallback';
        b &&= 'modified';
        c ??= 'null-fallback';
        d ??= 'wont-change';

        [a, b, c, d]
    "#).await.unwrap();

    assert!(result.is_object());
}
