use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_engine_initialization() {
    let engine = JavaScriptEngine::new();
    match &engine {
        Ok(_) => println!("Engine initialized successfully"),
        Err(e) => println!("Engine initialization failed: {}", e),
    }
    assert!(engine.is_ok());
}

#[tokio::test]
async fn test_basic_javascript_execution() {
    let mut engine = JavaScriptEngine::new().unwrap();

    let result = engine.execute_enhanced("5 + 3").await.unwrap();
    assert!(result.as_number().is_some());
    assert_eq!(result.as_number().unwrap(), 8.0);
}

#[tokio::test]
async fn test_es2023_array_findlast() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that findLast polyfill is loaded
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3, 4, 5];
        typeof arr.findLast === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_es2024_promise_with_resolvers() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that Promise.withResolvers polyfill is loaded
    let result = engine.execute_enhanced(r#"
        typeof Promise.withResolvers === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_es2025_experimental_record() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that Record polyfill is loaded
    let result = engine.execute_enhanced(r#"
        typeof Record === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_syntax_transformation_nullish_coalescing() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test nullish coalescing transformation
    let result = engine.execute_enhanced(r#"
        const a = null;
        const b = "default";
        a ?? b
    "#).await.unwrap();

    // Check if result is a string
    assert!(result.is_string());
}

#[tokio::test]
async fn test_syntax_transformation_numeric_separators() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test numeric separators transformation
    let result = engine.execute_enhanced(r#"
        const big = 1_000_000;
        big
    "#).await.unwrap();
    assert_eq!(result.as_number().unwrap(), 1000000.0);
}

#[tokio::test]
async fn test_console_polyfill() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test console object exists
    let result = engine.execute_enhanced(r#"
        typeof console === 'object' && typeof console.log === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_version_info() {
    let engine = JavaScriptEngine::new().unwrap();
    let version = engine.version_info();
    assert_eq!(version, "Enhanced JavaScript Engine v3.0 (ES2025+ Compatible)");
}

#[tokio::test]
async fn test_array_methods_polyfill() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test multiple array methods exist
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3];
        typeof arr.findLast === 'function' &&
        typeof arr.findLastIndex === 'function' &&
        typeof arr.toReversed === 'function' &&
        typeof arr.toSorted === 'function' &&
        typeof arr.toSpliced === 'function' &&
        typeof arr.with === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_set_methods_polyfill() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Set methods exist
    let result = engine.execute_enhanced(r#"
        const set = new Set([1, 2, 3]);
        typeof set.intersection === 'function' &&
        typeof set.union === 'function' &&
        typeof set.difference === 'function' &&
        typeof set.isSubsetOf === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_object_groupby_polyfill() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Object.groupBy exists
    let result = engine.execute_enhanced(r#"
        typeof Object.groupBy === 'function' &&
        typeof Map.groupBy === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_string_methods_polyfill() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String methods exist
    let result = engine.execute_enhanced(r#"
        const str = "hello";
        typeof str.isWellFormed === 'function' &&
        typeof str.toWellFormed === 'function' &&
        typeof str.dedent === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_experimental_features_polyfill() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test experimental features exist
    let result = engine.execute_enhanced(r#"
        typeof Record === 'function' &&
        typeof Tuple === 'function' &&
        typeof match === 'function' &&
        typeof pipe === 'function' &&
        typeof Observable === 'function' &&
        typeof AsyncContext === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_math_extensions_polyfill() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Math extensions exist
    let result = engine.execute_enhanced(r#"
        typeof Math.sumPrecise === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_error_extensions_polyfill() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Error extensions exist
    let result = engine.execute_enhanced(r#"
        typeof Error.isError === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_function_partial_application() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Function.prototype.partial exists
    let result = engine.execute_enhanced(r#"
        function test() { return true; }
        typeof test.partial === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}