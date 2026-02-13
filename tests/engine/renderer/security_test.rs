use thalora::{HeadlessWebBrowser, RustRenderer};
use thalora::engine::EngineType;

/// Test safe JavaScript passes validation
#[test]
fn test_safe_javascript_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let safe_code = "const x = 5; console.log(x);";
    assert!(renderer.is_safe_javascript(safe_code));
}

/// Test basic arithmetic is allowed
#[test]
fn test_arithmetic_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "2 + 2";
    assert!(renderer.is_safe_javascript(code));
}

/// Test DOM manipulation is allowed
#[test]
fn test_dom_manipulation_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "document.getElementById('test').textContent = 'Hello';";
    assert!(renderer.is_safe_javascript(code));
}

/// Test eval is allowed (V8 compliant)
#[test]
fn test_eval_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "eval('2 + 2')";
    assert!(renderer.is_safe_javascript(code));
}

/// Test Function constructor is allowed (V8 compliant)
#[test]
fn test_function_constructor_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "new Function('return 2 + 2')()";
    assert!(renderer.is_safe_javascript(code));
}

/// Test process.exit is blocked
#[test]
fn test_process_exit_blocked() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "process.exit(1);";
    assert!(!renderer.is_safe_javascript(code));
}

/// Test process.abort is blocked
#[test]
fn test_process_abort_blocked() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "process.abort();";
    assert!(!renderer.is_safe_javascript(code));
}

/// Test process.kill is blocked
#[test]
fn test_process_kill_blocked() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "process.kill(process.pid);";
    assert!(!renderer.is_safe_javascript(code));
}

/// Test fs module require is blocked
#[test]
fn test_fs_require_blocked() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "require('fs')";
    assert!(!renderer.is_safe_javascript(code));

    let code2 = r#"require("fs")"#;
    assert!(!renderer.is_safe_javascript(code2));
}

/// Test child_process module require is blocked
#[test]
fn test_child_process_blocked() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "require('child_process')";
    assert!(!renderer.is_safe_javascript(code));

    let code2 = r#"require("child_process")"#;
    assert!(!renderer.is_safe_javascript(code2));
}

/// Test os module require is blocked
#[test]
fn test_os_module_blocked() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "require('os')";
    assert!(!renderer.is_safe_javascript(code));

    let code2 = r#"require("os")"#;
    assert!(!renderer.is_safe_javascript(code2));
}

/// Test __dirname is blocked
#[test]
fn test_dirname_blocked() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "console.log(__dirname);";
    assert!(!renderer.is_safe_javascript(code));
}

/// Test __filename is blocked
#[test]
fn test_filename_blocked() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "console.log(__filename);";
    assert!(!renderer.is_safe_javascript(code));
}

/// Test global.process is blocked
#[test]
fn test_global_process_blocked() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "global.process.exit();";
    assert!(!renderer.is_safe_javascript(code));
}

/// Test Buffer.allocUnsafe is blocked
#[test]
fn test_buffer_alloc_unsafe_blocked() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "Buffer.allocUnsafe(1000);";
    assert!(!renderer.is_safe_javascript(code));
}

/// Test constructor.constructor is blocked
#[test]
fn test_constructor_constructor_blocked() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "({}).constructor.constructor('return process')();";
    assert!(!renderer.is_safe_javascript(code));
}

/// Test very large code is blocked (>10MB)
#[test]
fn test_large_code_blocked() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let large_code = "x".repeat(10_000_001);
    assert!(!renderer.is_safe_javascript(&large_code));
}

/// Test code at size limit is allowed (exactly 10MB)
#[test]
fn test_code_at_size_limit_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code_at_limit = "x".repeat(10_000_000);
    assert!(renderer.is_safe_javascript(&code_at_limit));
}

/// Test empty code is allowed
#[test]
fn test_empty_code_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "";
    assert!(renderer.is_safe_javascript(code));
}

/// Test whitespace-only code is allowed
#[test]
fn test_whitespace_code_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "   \n\t  ";
    assert!(renderer.is_safe_javascript(code));
}

/// Test comments-only code is allowed
#[test]
fn test_comments_only_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "// This is a comment\n/* Block comment */";
    assert!(renderer.is_safe_javascript(code));
}

/// Test async/await is allowed
#[test]
fn test_async_await_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "async function test() { await Promise.resolve(42); }";
    assert!(renderer.is_safe_javascript(code));
}

/// Test fetch API is allowed
#[test]
fn test_fetch_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "fetch('https://api.example.com/data')";
    assert!(renderer.is_safe_javascript(code));
}

/// Test localStorage is allowed
#[test]
fn test_localstorage_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "localStorage.setItem('key', 'value');";
    assert!(renderer.is_safe_javascript(code));
}

/// Test WebSocket is allowed
#[test]
fn test_websocket_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "new WebSocket('ws://localhost:8080');";
    assert!(renderer.is_safe_javascript(code));
}

/// Test setTimeout is allowed
#[test]
fn test_settimeout_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "setTimeout(() => console.log('test'), 1000);";
    assert!(renderer.is_safe_javascript(code));
}

/// Test setInterval is allowed
#[test]
fn test_setinterval_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "setInterval(() => console.log('test'), 1000);";
    assert!(renderer.is_safe_javascript(code));
}

/// Test XMLHttpRequest is allowed
#[test]
fn test_xmlhttprequest_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "const xhr = new XMLHttpRequest();";
    assert!(renderer.is_safe_javascript(code));
}

/// Test complex nested code is allowed
#[test]
fn test_complex_nested_code_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = r#"
        function complexFunction() {
            const data = {
                nested: {
                    deep: {
                        value: 42
                    }
                }
            };
            return data.nested.deep.value;
        }
    "#;
    assert!(renderer.is_safe_javascript(code));
}

/// Test ES6 features are allowed
#[test]
fn test_es6_features_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = r#"
        const arrow = () => {};
        const [a, b] = [1, 2];
        const {x, y} = {x: 3, y: 4};
        const template = `Hello ${x}`;
        class MyClass {}
    "#;
    assert!(renderer.is_safe_javascript(code));
}

/// Test dangerous code embedded in strings is blocked
#[test]
fn test_dangerous_in_string_blocked() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = r#"const cmd = "process.exit(1)";"#;
    assert!(!renderer.is_safe_javascript(code));
}

/// Test multiple dangerous patterns
#[test]
fn test_multiple_dangerous_patterns() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "require('fs'); process.exit(1);";
    assert!(!renderer.is_safe_javascript(code));
}

/// Test case sensitivity in dangerous patterns
#[test]
fn test_case_sensitivity() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    // Exact match required
    let code = "Process.Exit(1);"; // Capital P and E
    assert!(renderer.is_safe_javascript(code)); // Should be allowed (case sensitive)
}

/// Test dangerous pattern at start of code
#[test]
fn test_dangerous_pattern_at_start() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "process.exit(0);";
    assert!(!renderer.is_safe_javascript(code));
}

/// Test dangerous pattern at end of code
#[test]
fn test_dangerous_pattern_at_end() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "const x = 5; process.exit(x);";
    assert!(!renderer.is_safe_javascript(code));
}

/// Test dangerous pattern in middle of code
#[test]
fn test_dangerous_pattern_in_middle() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "const x = 5; process.exit(x); const y = 10;";
    assert!(!renderer.is_safe_javascript(code));
}

/// Test regex patterns are allowed
#[test]
fn test_regex_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "const pattern = /test/gi;";
    assert!(renderer.is_safe_javascript(code));
}

/// Test JSON operations are allowed
#[test]
fn test_json_operations_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = r#"
        const obj = {key: "value"};
        const json = JSON.stringify(obj);
        const parsed = JSON.parse(json);
    "#;
    assert!(renderer.is_safe_javascript(code));
}

/// Test array operations are allowed
#[test]
fn test_array_operations_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = r#"
        const arr = [1, 2, 3];
        const mapped = arr.map(x => x * 2);
        const filtered = arr.filter(x => x > 1);
        const reduced = arr.reduce((a, b) => a + b, 0);
    "#;
    assert!(renderer.is_safe_javascript(code));
}

/// Test Promise operations are allowed
#[test]
fn test_promise_operations_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = r#"
        const promise = new Promise((resolve, reject) => {
            resolve(42);
        });
        promise.then(value => console.log(value));
    "#;
    assert!(renderer.is_safe_javascript(code));
}

/// Test Symbol is allowed
#[test]
fn test_symbol_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "const sym = Symbol('test');";
    assert!(renderer.is_safe_javascript(code));
}

/// Test Proxy is allowed
#[test]
fn test_proxy_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = r#"
        const handler = {
            get: (target, prop) => target[prop]
        };
        const proxy = new Proxy({}, handler);
    "#;
    assert!(renderer.is_safe_javascript(code));
}

/// Test Reflect is allowed
#[test]
fn test_reflect_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = "Reflect.get({x: 1}, 'x');";
    assert!(renderer.is_safe_javascript(code));
}

/// Test Generator functions are allowed
#[test]
fn test_generator_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = r#"
        function* generator() {
            yield 1;
            yield 2;
            yield 3;
        }
    "#;
    assert!(renderer.is_safe_javascript(code));
}

/// Test try-catch is allowed
#[test]
fn test_try_catch_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = r#"
        try {
            throw new Error('test');
        } catch (e) {
            console.error(e);
        } finally {
            console.log('done');
        }
    "#;
    assert!(renderer.is_safe_javascript(code));
}

/// Test weak collections are allowed
#[test]
fn test_weak_collections_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = r#"
        const weakMap = new WeakMap();
        const weakSet = new WeakSet();
    "#;
    assert!(renderer.is_safe_javascript(code));
}

/// Test typed arrays are allowed
#[test]
fn test_typed_arrays_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = r#"
        const buffer = new ArrayBuffer(16);
        const int32View = new Int32Array(buffer);
        const uint8View = new Uint8Array(buffer);
    "#;
    assert!(renderer.is_safe_javascript(code));
}

/// Test Map and Set are allowed
#[test]
fn test_map_set_allowed() {
    let renderer = RustRenderer::new_with_engine(EngineType::Boa);

    let code = r#"
        const map = new Map();
        map.set('key', 'value');
        const set = new Set([1, 2, 3]);
    "#;
    assert!(renderer.is_safe_javascript(code));
}
