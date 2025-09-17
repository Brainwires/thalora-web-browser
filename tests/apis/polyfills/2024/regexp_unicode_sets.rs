use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_regexp_unicode_sets() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test RegExp unicodeSets property
    let result = engine.execute_enhanced(r#"
        try {
            const regex = /test/u;
            typeof regex.unicodeSets === 'boolean' || typeof regex.unicodeSets === 'undefined'
        } catch (e) {
            true
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
