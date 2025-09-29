use anyhow::Result;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use thalora::engine::{EngineFactory, EngineType, ThaloraBrowserEngine};

#[tokio::test]
async fn test_comprehensive_engine_comparison() -> Result<()> {
    println!("🔄 Starting comprehensive engine comparison...");
    
    // Create both engines
    let mut boa_engine = EngineFactory::create_engine(EngineType::Boa)?;
    let mut v8_engine = EngineFactory::create_engine(EngineType::V8)?;
    
    println!("✅ Both engines initialized successfully");
    
    // 1. Compare Global Object Properties
    compare_global_properties(&mut boa_engine, &mut v8_engine).await?;
    
    // 2. Compare Built-in Constructors
    compare_builtin_constructors(&mut boa_engine, &mut v8_engine).await?;
    
    // 3. Compare Standard Web APIs
    compare_web_apis(&mut boa_engine, &mut v8_engine).await?;
    
    // 4. Compare JavaScript Language Features
    compare_language_features(&mut boa_engine, &mut v8_engine).await?;
    
    // 5. Compare Type Behavior
    compare_type_behavior(&mut boa_engine, &mut v8_engine).await?;
    
    // 6. Compare Error Handling
    compare_error_handling(&mut boa_engine, &mut v8_engine).await?;
    
    // 7. Performance comparison
    compare_performance(&mut boa_engine, &mut v8_engine).await?;
    
    println!("🎉 Engine comparison test completed successfully!");
    Ok(())
}

async fn compare_global_properties(
    boa: &mut Box<dyn ThaloraBrowserEngine>, 
    v8: &mut Box<dyn ThaloraBrowserEngine>
) -> Result<()> {
    println!("\n📊 Comparing Global Object Properties...");
    
    let global_inspection_code = r#"
        (function() {
            const globals = [];
            const seen = new Set();
            
            // Get all own properties of the global object
            function collectProperties(obj, prefix = '') {
                if (seen.has(obj) || prefix.split('.').length > 3) return; // Prevent infinite recursion
                seen.add(obj);
                
                try {
                    const props = Object.getOwnPropertyNames(obj);
                    for (const prop of props) {
                        const fullName = prefix ? `${prefix}.${prop}` : prop;
                        try {
                            const value = obj[prop];
                            const type = typeof value;
                            globals.push({
                                name: fullName,
                                type: type,
                                isFunction: type === 'function',
                                isConstructor: type === 'function' && value.prototype !== undefined,
                                isNative: type === 'function' ? value.toString().includes('[native code]') : false
                            });
                        } catch (e) {
                            globals.push({
                                name: fullName,
                                type: 'inaccessible',
                                error: e.message
                            });
                        }
                    }
                } catch (e) {
                    // Object doesn't support property enumeration
                }
            }
            
            collectProperties(globalThis);
            return globals;
        })();
    "#;
    
    let boa_globals = boa.execute(global_inspection_code)?;
    let v8_globals = v8.execute(global_inspection_code)?;
    
    let boa_map = extract_global_map(&boa_globals)?;
    let v8_map = extract_global_map(&v8_globals)?;
    
    // Find differences
    let boa_only: HashSet<_> = boa_map.keys().cloned().collect();
    let v8_only: HashSet<_> = v8_map.keys().cloned().collect();
    
    let boa_exclusive: Vec<_> = boa_only.difference(&v8_only).collect();
    let v8_exclusive: Vec<_> = v8_only.difference(&boa_only).collect();
    let common: Vec<_> = boa_only.intersection(&v8_only).collect();
    
    println!("📈 Global Properties Summary:");
    println!("  - BOA total: {}", boa_map.len());
    println!("  - V8 total: {}", v8_map.len());
    println!("  - Common: {}", common.len());
    println!("  - BOA exclusive: {}", boa_exclusive.len());
    println!("  - V8 exclusive: {}", v8_exclusive.len());
    
    if !boa_exclusive.is_empty() {
        println!("\n🔵 BOA-only globals:");
        for prop in boa_exclusive.iter().take(10) {
            if let Some(info) = boa_map.get(*prop) {
                println!("  - {}: {} ({})", prop, info.get("type").unwrap_or(&Value::Null), 
                    if info.get("isFunction").unwrap_or(&Value::Bool(false)).as_bool().unwrap_or(false) { "function" } else { "value" });
            }
        }
        if boa_exclusive.len() > 10 {
            println!("  ... and {} more", boa_exclusive.len() - 10);
        }
    }
    
    if !v8_exclusive.is_empty() {
        println!("\n🟡 V8-only globals:");
        for prop in v8_exclusive.iter().take(10) {
            if let Some(info) = v8_map.get(*prop) {
                println!("  - {}: {} ({})", prop, info.get("type").unwrap_or(&Value::Null),
                    if info.get("isFunction").unwrap_or(&Value::Bool(false)).as_bool().unwrap_or(false) { "function" } else { "value" });
            }
        }
        if v8_exclusive.len() > 10 {
            println!("  ... and {} more", v8_exclusive.len() - 10);
        }
    }
    
    Ok(())
}

async fn compare_builtin_constructors(
    boa: &mut Box<dyn ThaloraBrowserEngine>, 
    v8: &mut Box<dyn ThaloraBrowserEngine>
) -> Result<()> {
    println!("\n🏗️ Comparing Built-in Constructors...");
    
    let constructor_test_code = r#"
        (function() {
            const constructors = [];
            const standardConstructors = [
                'Object', 'Array', 'Function', 'String', 'Number', 'Boolean', 'Symbol', 'BigInt',
                'Date', 'RegExp', 'Error', 'TypeError', 'ReferenceError', 'SyntaxError', 'RangeError',
                'Map', 'Set', 'WeakMap', 'WeakSet', 'Promise', 'Proxy', 'Reflect',
                'ArrayBuffer', 'SharedArrayBuffer', 'DataView', 
                'Int8Array', 'Uint8Array', 'Uint8ClampedArray', 'Int16Array', 'Uint16Array',
                'Int32Array', 'Uint32Array', 'Float32Array', 'Float64Array', 'BigInt64Array', 'BigUint64Array',
                'JSON', 'Math', 'Intl'
            ];
            
            for (const name of standardConstructors) {
                try {
                    const constructor = globalThis[name];
                    const info = {
                        name: name,
                        exists: constructor !== undefined,
                        type: typeof constructor,
                        isConstructor: typeof constructor === 'function' && constructor.prototype !== undefined,
                        hasPrototype: constructor && constructor.prototype !== undefined,
                        prototypeProps: constructor && constructor.prototype ? Object.getOwnPropertyNames(constructor.prototype).length : 0,
                        staticProps: constructor ? Object.getOwnPropertyNames(constructor).length : 0
                    };
                    
                    if (typeof constructor === 'function') {
                        info.toString = constructor.toString().substring(0, 100);
                    }
                    
                    constructors.push(info);
                } catch (e) {
                    constructors.push({
                        name: name,
                        exists: false,
                        error: e.message
                    });
                }
            }
            
            return constructors;
        })();
    "#;
    
    let boa_constructors = boa.execute(constructor_test_code)?;
    let v8_constructors = v8.execute(constructor_test_code)?;
    
    let boa_list = extract_constructor_list(&boa_constructors)?;
    let v8_list = extract_constructor_list(&v8_constructors)?;
    
    println!("🏗️ Constructor Comparison:");
    for (boa_info, v8_info) in boa_list.iter().zip(v8_list.iter()) {
        let name = boa_info.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
        let boa_exists = boa_info.get("exists").and_then(|v| v.as_bool()).unwrap_or(false);
        let v8_exists = v8_info.get("exists").and_then(|v| v.as_bool()).unwrap_or(false);
        
        let status = match (boa_exists, v8_exists) {
            (true, true) => "✅",
            (true, false) => "🔵", // BOA only
            (false, true) => "🟡", // V8 only
            (false, false) => "❌", // Neither
        };
        
        let boa_props = boa_info.get("prototypeProps").and_then(|v| v.as_u64()).unwrap_or(0);
        let v8_props = v8_info.get("prototypeProps").and_then(|v| v.as_u64()).unwrap_or(0);
        
        println!("  {} {:<20} BOA: {} props, V8: {} props", status, name, boa_props, v8_props);
    }
    
    Ok(())
}

async fn compare_web_apis(
    boa: &mut Box<dyn ThaloraBrowserEngine>, 
    v8: &mut Box<dyn ThaloraBrowserEngine>
) -> Result<()> {
    println!("\n🌐 Comparing Web APIs...");
    
    let web_api_test_code = r#"
        (function() {
            const webAPIs = [];
            const apiNames = [
                'console', 'setTimeout', 'setInterval', 'clearTimeout', 'clearInterval',
                'fetch', 'Request', 'Response', 'Headers', 'URL', 'URLSearchParams',
                'localStorage', 'sessionStorage', 'indexedDB',
                'crypto', 'performance', 'navigator', 'location', 'history',
                'XMLHttpRequest', 'WebSocket', 'EventSource',
                'Event', 'CustomEvent', 'EventTarget', 'AbortController', 'AbortSignal',
                'Blob', 'File', 'FileReader', 'FormData',
                'Worker', 'SharedWorker', 'ServiceWorker', 'MessageChannel', 'MessagePort',
                'TextEncoder', 'TextDecoder', 'atob', 'btoa'
            ];
            
            for (const name of apiNames) {
                try {
                    const api = globalThis[name];
                    const info = {
                        name: name,
                        exists: api !== undefined,
                        type: typeof api,
                        isFunction: typeof api === 'function',
                        hasProperties: api && typeof api === 'object' ? Object.keys(api).length > 0 : false
                    };
                    
                    // Special handling for some APIs
                    if (name === 'console' && api) {
                        info.methods = Object.getOwnPropertyNames(api).filter(prop => typeof api[prop] === 'function').length;
                    }
                    
                    if (name === 'crypto' && api) {
                        info.hasMethods = !!(api.getRandomValues || api.randomUUID);
                    }
                    
                    webAPIs.push(info);
                } catch (e) {
                    webAPIs.push({
                        name: name,
                        exists: false,
                        error: e.message
                    });
                }
            }
            
            return webAPIs;
        })();
    "#;
    
    let boa_apis = boa.execute(web_api_test_code)?;
    let v8_apis = v8.execute(web_api_test_code)?;
    
    let boa_list = extract_web_api_list(&boa_apis)?;
    let v8_list = extract_web_api_list(&v8_apis)?;
    
    println!("🌐 Web API Comparison:");
    for (boa_info, v8_info) in boa_list.iter().zip(v8_list.iter()) {
        let name = boa_info.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
        let boa_exists = boa_info.get("exists").and_then(|v| v.as_bool()).unwrap_or(false);
        let v8_exists = v8_info.get("exists").and_then(|v| v.as_bool()).unwrap_or(false);
        
        let status = match (boa_exists, v8_exists) {
            (true, true) => "✅",
            (true, false) => "🔵", // BOA only
            (false, true) => "🟡", // V8 only  
            (false, false) => "❌", // Neither
        };
        
        let boa_type = boa_info.get("type").and_then(|v| v.as_str()).unwrap_or("undefined");
        let v8_type = v8_info.get("type").and_then(|v| v.as_str()).unwrap_or("undefined");
        
        println!("  {} {:<20} BOA: {}, V8: {}", status, name, boa_type, v8_type);
    }
    
    Ok(())
}

async fn compare_language_features(
    boa: &mut Box<dyn ThaloraBrowserEngine>, 
    v8: &mut Box<dyn ThaloraBrowserEngine>
) -> Result<()> {
    println!("\n🔧 Comparing JavaScript Language Features...");
    
    let feature_tests = vec![
        ("ES6 Arrow Functions", "(() => 42)()"),
        ("ES6 Classes", "class Test {}; new Test() instanceof Test"),
        ("ES6 Template Literals", "`Hello ${1 + 1}` === 'Hello 2'"),
        ("ES6 Destructuring", "const [a, b] = [1, 2]; a === 1 && b === 2"),
        ("ES6 Default Parameters", "(function(x = 5) { return x; })() === 5"),
        ("ES6 Rest Parameters", "(function(...args) { return args.length; })(1, 2, 3) === 3"),
        ("ES6 Spread Operator", "[...[1, 2, 3]].length === 3"),
        ("ES6 for...of", "{ let sum = 0; for (const x of [1, 2, 3]) sum += x; sum === 6; }"),
        ("ES6 Map", "new Map().set('key', 'value').get('key') === 'value'"),
        ("ES6 Set", "new Set([1, 2, 2]).size === 2"),
        ("ES6 Promise", "typeof Promise === 'function'"),
        ("ES6 Symbol", "typeof Symbol === 'function'"),
        ("ES8 async/await", "(async function() { return await Promise.resolve(42); })() instanceof Promise"),
        ("ES2020 BigInt", "typeof BigInt === 'function'"),
        ("ES2020 Optional Chaining", "({})?.prop?.value === undefined"),
        ("ES2020 Nullish Coalescing", "null ?? 'default' === 'default'"),
        ("Proxy", "typeof Proxy === 'function'"),
        ("Reflect", "typeof Reflect === 'object'"),
        ("WeakMap", "typeof WeakMap === 'function'"),
        ("WeakSet", "typeof WeakSet === 'function'"),
    ];
    
    println!("🔧 Language Feature Support:");
    for (feature_name, test_code) in feature_tests {
        let boa_result = test_feature_support(boa, test_code).await;
        let v8_result = test_feature_support(v8, test_code).await;
        
        let status = match (boa_result, v8_result) {
            (true, true) => "✅",
            (true, false) => "🔵", // BOA only
            (false, true) => "🟡", // V8 only
            (false, false) => "❌", // Neither
        };
        
        println!("  {} {}", status, feature_name);
    }
    
    Ok(())
}

async fn test_feature_support(engine: &mut Box<dyn ThaloraBrowserEngine>, test_code: &str) -> bool {
    let wrapped_code = format!("try {{ {} }} catch (e) {{ false }}", test_code);
    
    match engine.execute(&wrapped_code) {
        Ok(result) => {
            // Try to parse as boolean
            if let Ok(value) = serde_json::from_str::<Value>(&result) {
                value.as_bool().unwrap_or(false)
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

async fn compare_type_behavior(
    boa: &mut Box<dyn ThaloraBrowserEngine>, 
    v8: &mut Box<dyn ThaloraBrowserEngine>
) -> Result<()> {
    println!("\n🔍 Comparing Type Behavior...");
    
    let type_tests = vec![
        ("typeof null", "typeof null"),
        ("typeof undefined", "typeof undefined"),
        ("typeof []", "typeof []"),
        ("typeof {}", "typeof {}"),
        ("typeof function(){}", "typeof function(){}"),
        ("Array.isArray([])", "Array.isArray([])"),
        ("Number.isNaN(NaN)", "Number.isNaN(NaN)"),
        ("Number.isFinite(42)", "Number.isFinite(42)"),
        ("Object.is(NaN, NaN)", "Object.is(NaN, NaN)"),
        ("'5' == 5", "'5' == 5"),
        ("'5' === 5", "'5' === 5"),
        ("null == undefined", "null == undefined"),
        ("null === undefined", "null === undefined"),
    ];
    
    println!("🔍 Type Behavior Comparison:");
    for (test_name, test_code) in type_tests {
        let boa_result = execute_and_stringify(boa, test_code).await;
        let v8_result = execute_and_stringify(v8, test_code).await;
        
        let status = if boa_result == v8_result { "✅" } else { "⚠️" };
        
        println!("  {} {:<25} BOA: {}, V8: {}", status, test_name, boa_result, v8_result);
    }
    
    Ok(())
}

async fn compare_error_handling(
    boa: &mut Box<dyn ThaloraBrowserEngine>, 
    v8: &mut Box<dyn ThaloraBrowserEngine>
) -> Result<()> {
    println!("\n❌ Comparing Error Handling...");
    
    let error_tests = vec![
        ("ReferenceError", "undefinedVariable"),
        ("TypeError", "null.property"),
        ("SyntaxError", "eval('var x = ;')"),
        ("RangeError", "new Array(-1)"),
        ("TypeError Function Call", "(42)()"),
    ];
    
    println!("❌ Error Handling Comparison:");
    for (error_type, test_code) in error_tests {
        let boa_error = get_error_type(boa, test_code).await;
        let v8_error = get_error_type(v8, test_code).await;
        
        let status = if boa_error == v8_error { "✅" } else { "⚠️" };
        
        println!("  {} {:<25} BOA: {}, V8: {}", status, error_type, boa_error, v8_error);
    }
    
    Ok(())
}

async fn get_error_type(engine: &mut Box<dyn ThaloraBrowserEngine>, test_code: &str) -> String {
    let wrapped_code = format!(
        "try {{ {}; 'no-error' }} catch (e) {{ e.constructor.name || 'UnknownError' }}", 
        test_code
    );
    
    execute_and_stringify(engine, &wrapped_code).await
}

async fn compare_performance(
    boa: &mut Box<dyn ThaloraBrowserEngine>, 
    v8: &mut Box<dyn ThaloraBrowserEngine>
) -> Result<()> {
    println!("\n⚡ Performance Comparison...");
    
    let performance_tests = vec![
        ("Simple arithmetic", "let sum = 0; for (let i = 0; i < 10000; i++) sum += i; sum"),
        ("Array operations", "const arr = Array(1000).fill(0).map((_, i) => i); arr.reduce((a, b) => a + b, 0)"),
        ("Object creation", "const objs = []; for (let i = 0; i < 1000; i++) objs.push({id: i, value: i * 2}); objs.length"),
        ("String manipulation", "let str = ''; for (let i = 0; i < 1000; i++) str += i.toString(); str.length"),
        ("Function calls", "function test(x) { return x * 2; } let result = 0; for (let i = 0; i < 1000; i++) result += test(i); result"),
    ];
    
    println!("⚡ Performance Results:");
    for (test_name, test_code) in performance_tests {
        let boa_time = measure_execution_time(boa, test_code).await;
        let v8_time = measure_execution_time(v8, test_code).await;
        
        let faster = if boa_time < v8_time { "🔵 BOA" } else { "🟡 V8" };
        let ratio = if boa_time < v8_time { 
            v8_time.as_nanos() as f64 / boa_time.as_nanos() as f64 
        } else { 
            boa_time.as_nanos() as f64 / v8_time.as_nanos() as f64 
        };
        
        println!("  {:<25} BOA: {:>8.2}ms, V8: {:>8.2}ms {} ({:.1}x faster)", 
            test_name, 
            boa_time.as_secs_f64() * 1000.0,
            v8_time.as_secs_f64() * 1000.0,
            faster,
            ratio
        );
    }
    
    Ok(())
}

async fn measure_execution_time(engine: &mut Box<dyn ThaloraBrowserEngine>, code: &str) -> std::time::Duration {
    let start = std::time::Instant::now();
    let _ = engine.execute(code);
    start.elapsed()
}

async fn execute_and_stringify(engine: &mut Box<dyn ThaloraBrowserEngine>, code: &str) -> String {
    match engine.execute(code) {
        Ok(result) => result,
        Err(e) => format!("Error: {}", e),
    }
}

// Helper functions to extract data from JSON responses

fn extract_global_map(json_str: &str) -> Result<HashMap<String, Map<String, Value>>> {
    let globals: Vec<Map<String, Value>> = serde_json::from_str(json_str)?;
    let mut map = HashMap::new();
    
    for global in globals {
        if let Some(name) = global.get("name").and_then(|v| v.as_str()) {
            map.insert(name.to_string(), global);
        }
    }
    
    Ok(map)
}

fn extract_constructor_list(json_str: &str) -> Result<Vec<Map<String, Value>>> {
    let constructors: Vec<Map<String, Value>> = serde_json::from_str(json_str)?;
    Ok(constructors)
}

fn extract_web_api_list(json_str: &str) -> Result<Vec<Map<String, Value>>> {
    let apis: Vec<Map<String, Value>> = serde_json::from_str(json_str)?;
    Ok(apis)
}

#[tokio::test]
async fn test_engine_consistency_basic() -> Result<()> {
    println!("🔄 Testing basic engine consistency...");
    
    let mut boa_engine = EngineFactory::create_engine(EngineType::Boa)?;
    let mut v8_engine = EngineFactory::create_engine(EngineType::V8)?;
    
    let test_cases = vec![
        "2 + 2",
        "Math.PI",
        "JSON.stringify({test: 'value'})",
        "[1, 2, 3].map(x => x * 2)",
        "typeof undefined",
        "typeof null",
    ];
    
    println!("🧪 Basic Consistency Tests:");
    for test in test_cases {
        let boa_result = boa_engine.execute(test)?;
        let v8_result = v8_engine.execute(test)?;
        
        let status = if boa_result == v8_result { "✅" } else { "⚠️" };
        println!("  {} {:<35} BOA: {}, V8: {}", status, test, boa_result, v8_result);
    }
    
    Ok(())
}