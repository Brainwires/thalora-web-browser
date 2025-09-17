use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_tuple_immutability() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Tuple immutability
    let result = engine.execute_enhanced(r#"
        const tup = Tuple('a', 'b', 'c');
        try {
            tup[1] = 'changed'; // Should not change the tuple
            tup[1] === 'b' // Original value preserved
        } catch (e) {
            true // Might throw in strict mode
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
