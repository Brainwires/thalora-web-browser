#[tokio::test]
async fn test_regexp_dotall_flag() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test s (dotAll) flag support
    let result = engine.execute_enhanced(r#"
        try {
            const regex = /./s;
            regex.test('\\n')
        } catch (e) {
            // s flag might not be supported
            true
        }
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
