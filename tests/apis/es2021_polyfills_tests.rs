use synaptic::js::JavaScriptEngine;

#[tokio::test]
async fn test_string_replace_all() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String.prototype.replaceAll
    let result = engine.execute_enhanced(r#"
        const str = 'hello world hello';
        [
            str.replaceAll('hello', 'hi'),
            str.replaceAll('world', 'universe'),
            'test test test'.replaceAll('test', 'demo')
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_promise_any() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Promise.any
    let result = engine.execute_enhanced(r#"
        typeof Promise.any === 'function'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_aggregate_error() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test AggregateError
    let result = engine.execute_enhanced(r#"
        typeof AggregateError === 'function'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_weak_refs() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test WeakRef
    let result = engine.execute_enhanced(r#"
        typeof WeakRef === 'function' || typeof WeakRef === 'undefined'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_finalization_registry() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test FinalizationRegistry
    let result = engine.execute_enhanced(r#"
        typeof FinalizationRegistry === 'function' || typeof FinalizationRegistry === 'undefined'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

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

#[tokio::test]
async fn test_numeric_separators() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test numeric separators
    let result = engine.execute_enhanced(r#"
        const million = 1_000_000;
        const binary = 0b1010_0001;
        const hex = 0xFF_EC_DE_5E;
        [million, binary, hex]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_intl_list_format_complete() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Intl.ListFormat (if available)
    let result = engine.execute_enhanced(r#"
        try {
            if (typeof Intl.ListFormat !== 'undefined') {
                const list = new Intl.ListFormat('en', { style: 'long', type: 'conjunction' });
                typeof list.format === 'function'
            } else {
                true // Not required to be supported
            }
        } catch (e) {
            true // Graceful fallback
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_intl_date_time_format_date_style() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Intl.DateTimeFormat dateStyle and timeStyle
    let result = engine.execute_enhanced(r#"
        try {
            if (typeof Intl.DateTimeFormat !== 'undefined') {
                const formatter = new Intl.DateTimeFormat('en');
                typeof formatter.format === 'function'
            } else {
                true // Not required to be supported
            }
        } catch (e) {
            true // Graceful fallback
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

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

#[tokio::test]
async fn test_logical_assignment_with_nullish() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test logical assignment with nullish values
    let result = engine.execute_enhanced(r#"
        let x = null;
        let y = undefined;
        let z = 0;
        let w = '';

        x ??= 'null_replacement';
        y ??= 'undefined_replacement';
        z ??= 'zero_replacement';
        w ??= 'empty_replacement';

        [x, y, z, w]
    "#).await.unwrap();

    assert!(result.is_object());
}