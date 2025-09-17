use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_pipeline_operator() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test pipeline operator (pipe function)
    let result = engine.execute_enhanced(r#"
        function double(x) { return x * 2; }
        function addTen(x) { return x + 10; }
        function toString(x) { return String(x); }

        const result = pipe(5, double, addTen, toString);
        [
            result === '20',
            typeof pipe === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
