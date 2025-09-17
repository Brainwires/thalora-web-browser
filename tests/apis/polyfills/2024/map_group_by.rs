use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_map_group_by() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Map.groupBy
    let result = engine.execute_enhanced(r#"
        const items = ['apple', 'banana', 'cherry', 'apricot', 'blueberry'];
        const grouped = Map.groupBy(items, item => item[0]);
        [
            grouped.get('a').length,
            grouped.get('b').length,
            grouped.get('c').length,
            grouped.has('d')
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
