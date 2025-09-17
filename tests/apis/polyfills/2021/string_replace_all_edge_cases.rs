use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_string_replace_all_edge_cases() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String.prototype.replaceAll edge cases
    let result = engine.execute_enhanced(r#"
        [
            ''.replaceAll('', 'x'),
            'aaa'.replaceAll('a', ''),
            'hello'.replaceAll('xyz', 'abc'),
            'test'.replaceAll('t', 'T')
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
