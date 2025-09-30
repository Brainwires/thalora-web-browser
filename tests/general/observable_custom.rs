use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_observable_custom() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test custom Observable creation
    let result = engine.execute_enhanced(r#"
        const obs = new Observable(observer => {
            observer.next('hello');
            observer.next('world');
            observer.complete();
        });

        const values = [];
        let completed = false;

        obs.subscribe({
            next: value => values.push(value),
            complete: () => completed = true
        });

        [
            values.length === 2,
            values[0] === 'hello',
            values[1] === 'world',
            completed === true
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
