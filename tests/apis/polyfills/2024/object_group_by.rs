#[tokio::test]
async fn test_object_group_by() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Object.groupBy
    let result = engine.execute_enhanced(r#"
        const items = [
            { category: 'A', value: 1 },
            { category: 'B', value: 2 },
            { category: 'A', value: 3 },
            { category: 'C', value: 4 }
        ];
        const grouped = Object.groupBy(items, item => item.category);
        [
            grouped.A.length,
            grouped.B.length,
            grouped.C.length,
            typeof grouped.A[0] === 'object'
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
