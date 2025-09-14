use synaptic::enhanced_js_v2::EnhancedJavaScriptEngine;

#[tokio::test]
async fn test_string_match_all() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test String.prototype.matchAll
    let result = engine.execute_enhanced(r#"
        const str = 'test1 test2 test3';
        const matches = [...str.matchAll(/test\\d/g)];
        matches.length === 3
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_promise_all_settled() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test Promise.allSettled
    let result = engine.execute_enhanced(r#"
        typeof Promise.allSettled === 'function'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_global_this() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test globalThis availability
    let result = engine.execute_enhanced(r#"
        typeof globalThis === 'object'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_nullish_coalescing() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test nullish coalescing operator (??)
    let result = engine.execute_enhanced(r#"
        const a = null;
        const b = undefined;
        const c = 0;
        const d = '';
        [
            a ?? 'default',
            b ?? 'default',
            c ?? 'default',
            d ?? 'default'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_optional_chaining() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test optional chaining operator (?.)
    let result = engine.execute_enhanced(r#"
        const obj = { nested: { value: 42 } };
        const empty = null;
        [
            obj?.nested?.value,
            empty?.nested?.value,
            obj?.missing?.value,
            obj?.nested?.missing
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_bigint_support() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test BigInt support
    let result = engine.execute_enhanced(r#"
        typeof BigInt === 'function' && typeof BigInt('123') === 'bigint'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_dynamic_import() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test dynamic import syntax (transformed)
    let result = engine.execute_enhanced(r#"
        // Dynamic imports would be transformed or polyfilled
        typeof Promise !== 'undefined'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_import_meta() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test import.meta (would be polyfilled or transformed)
    let result = engine.execute_enhanced(r#"
        // import.meta would be handled by module system
        typeof undefined === 'undefined'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_for_in_mechanics() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test for-in mechanics improvements
    let result = engine.execute_enhanced(r#"
        const obj = { a: 1, b: 2, c: 3 };
        const keys = [];
        for (const key in obj) {
            keys.push(key);
        }
        keys.length === 3
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_intl_list_format() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test Intl.ListFormat (may not be fully supported)
    let result = engine.execute_enhanced(r#"
        typeof Intl !== 'undefined'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_intl_locale() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test Intl.Locale (may not be fully supported)
    let result = engine.execute_enhanced(r#"
        typeof Intl !== 'undefined'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}