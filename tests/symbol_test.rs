use thalora_browser_apis::boa_engine::{Context, Source};

#[test]
fn test_symbol_has_instance() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test 1: Check Symbol exists
    let result = context.eval(Source::from_bytes("typeof Symbol")).unwrap();
    let type_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    assert_eq!(type_str, "function", "Symbol should be a function");
    println!("✅ Symbol exists: {}", type_str);

    // Test 2: Check Symbol.hasInstance exists
    let result = context.eval(Source::from_bytes("typeof Symbol.hasInstance")).unwrap();
    let type_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    assert_eq!(type_str, "symbol", "Symbol.hasInstance should be a symbol");
    println!("✅ Symbol.hasInstance exists: {}", type_str);

    // Test 3: Check Symbol.iterator exists
    let result = context.eval(Source::from_bytes("typeof Symbol.iterator")).unwrap();
    let type_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    assert_eq!(type_str, "symbol", "Symbol.iterator should be a symbol");
    println!("✅ Symbol.iterator exists: {}", type_str);

    // Test 4: Test the actual Cloudflare pattern
    let test_code = r#"
    (function() {
        function P(e, t) {
            return t != null && typeof Symbol !== "undefined" && t[Symbol.hasInstance] ? !!t[Symbol.hasInstance](e) : e instanceof t;
        }
        // Test with basic types
        var result = P([], Array);
        return result;
    })()
    "#;
    let result = context.eval(Source::from_bytes(test_code)).unwrap();
    let bool_result = result.to_boolean();
    assert!(bool_result, "P([], Array) should return true");
    println!("✅ Cloudflare pattern works: {}", bool_result);
}

#[test]
fn test_iframe_content_document() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test iframe creation
    let result = context.eval(Source::from_bytes(r#"
        var iframe = document.createElement('iframe');
        typeof iframe.contentDocument
    "#)).unwrap();
    let type_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("iframe.contentDocument type: {}", type_str);
    assert!(type_str == "object" || type_str == "null", "contentDocument should be object or null");

    // Test contentDocument has createElement
    let result = context.eval(Source::from_bytes(r#"
        var iframe = document.createElement('iframe');
        var doc = iframe.contentDocument;
        doc ? typeof doc.createElement : 'no doc'
    "#)).unwrap();
    let type_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("iframe.contentDocument.createElement type: {}", type_str);
    if type_str != "no doc" {
        assert_eq!(type_str, "function", "createElement should be a function");
    }
}

#[test]
fn test_function_has_instance() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test Function.prototype[Symbol.hasInstance]
    let result = context.eval(Source::from_bytes("typeof Function.prototype[Symbol.hasInstance]")).unwrap();
    let type_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("Function.prototype[Symbol.hasInstance] type: {}", type_str);
    
    // Test Array[Symbol.hasInstance]
    let result = context.eval(Source::from_bytes("typeof Array[Symbol.hasInstance]")).unwrap();
    let type_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("Array[Symbol.hasInstance] type: {}", type_str);

    // Test calling it
    let result = context.eval(Source::from_bytes(r#"
        var arr = [];
        Array[Symbol.hasInstance](arr)
    "#)).unwrap();
    let bool_result = result.to_boolean();
    println!("Array[Symbol.hasInstance]([]) = {}", bool_result);
    assert!(bool_result, "Array[Symbol.hasInstance]([]) should be true");
}

#[test]
fn test_cloudflare_script_pattern() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // This is the actual pattern from Cloudflare's script
    let test_code = r#"
    "use strict";(function(){
        function Ht(e,t,r,o,c,u,v){
            try{var h=e[u](v),f=h.value}catch(d){r(d);return}
            h.done?t(f):Promise.resolve(f).then(o,c)
        }
        function Gt(e){
            return function(){
                var t=this,r=arguments;
                return new Promise(function(o,c){
                    var u=e.apply(t,r);
                    function v(f){Ht(u,o,c,v,h,"next",f)}
                    function h(f){Ht(u,o,c,v,h,"throw",f)}
                    v(void 0)
                })
            }
        }
        function P(e,t){
            return t!=null&&typeof Symbol!="undefined"&&t[Symbol.hasInstance]?!!t[Symbol.hasInstance](e):e instanceof t;
        }
        
        // Test the P function
        var result = P([], Array);
        return result;
    })()
    "#;
    
    let result = context.eval(Source::from_bytes(test_code)).unwrap();
    let bool_result = result.to_boolean();
    println!("Cloudflare pattern P([], Array) = {}", bool_result);
    assert!(bool_result, "P([], Array) should return true");
}

#[test]
fn test_cloudflare_script_beginning() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test the actual beginning of Cloudflare script
    let test_code = r#""use strict";(function(){function Ht(e,t,r,o,c,u,v){try{var h=e[u](v),f=h.value}catch(d){r(d);return}h.done?t(f):Promise.resolve(f).then(o,c)}function Gt(e){return function(){var t=this,r=arguments;return new Promise(function(o,c){var u=e.apply(t,r);function v(f){Ht(u,o,c,v,h,"next",f)}function h(f){Ht(u,o,c,v,h,"throw",f)}v(void 0)})}}return "success";})()"#;
    
    let result = context.eval(Source::from_bytes(test_code));
    match result {
        Ok(value) => {
            let str_val = value.to_string(&mut context).unwrap().to_std_string_escaped();
            println!("Script beginning executed successfully: {}", str_val);
            assert_eq!(str_val, "success");
        },
        Err(e) => {
            println!("Script beginning failed: {:?}", e);
            panic!("Should have succeeded");
        }
    }
}

#[test]
fn test_cloudflare_incremental() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Read the actual script
    let script = std::fs::read_to_string("/tmp/cloudflare_script.js")
        .expect("Could not read /tmp/cloudflare_script.js");
    
    // Try executing first N characters, wrapping with a try-catch and return
    let sizes = [1000, 2000, 5000, 10000, 20000, 50000];
    
    for size in sizes {
        if size > script.len() {
            break;
        }
        
        // Get the first N characters
        let partial = &script[..size];
        
        // Wrap it to make it a valid expression that returns something
        let test_code = format!("try {{ {} }} catch(e) {{ 'error: ' + e.message }}", partial);
        
        match context.eval(Source::from_bytes(&test_code)) {
            Ok(value) => {
                let str_val = value.to_string(&mut context).unwrap().to_std_string_escaped();
                println!("✅ {} chars: {}", size, if str_val.len() > 50 { &str_val[..50] } else { &str_val });
            },
            Err(e) => {
                println!("❌ {} chars failed: {:?}", size, e);
                break;
            }
        }
    }
}

#[test]
fn test_cloudflare_full_script() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Read the actual script
    let script = match std::fs::read_to_string("/tmp/cloudflare_script.js") {
        Ok(s) => s,
        Err(_) => {
            println!("Skipping test - /tmp/cloudflare_script.js not found");
            return;
        }
    };
    
    println!("Script length: {} chars", script.len());
    println!("Script start: {}", &script[..100.min(script.len())]);
    
    match context.eval(Source::from_bytes(&script)) {
        Ok(value) => {
            let str_val = value.to_string(&mut context).unwrap().to_std_string_escaped();
            println!("✅ Full script executed: {}", if str_val.len() > 100 { &str_val[..100] } else { &str_val });
        },
        Err(e) => {
            println!("❌ Full script failed: {:?}", e);
            // This is expected to fail - we're trying to find out why
        }
    }
}

#[test]
fn test_self_and_this() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test self
    let result = context.eval(Source::from_bytes("typeof self")).unwrap();
    let type_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("typeof self: {}", type_str);

    // Test globalThis
    let result = context.eval(Source::from_bytes("typeof globalThis")).unwrap();
    let type_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("typeof globalThis: {}", type_str);

    // Test the UMD pattern
    let test_code = r#"
    (function(t, e) {
        var result;
        try {
            result = "exports check: " + (typeof exports);
        } catch(err) {
            result = "exports error: " + err.message;
        }
        try {
            result += ", module check: " + (typeof module);
        } catch(err) {
            result += ", module error: " + err.message;
        }
        try {
            result += ", define check: " + (typeof define);
        } catch(err) {
            result += ", define error: " + err.message;
        }
        try {
            result += ", self check: " + (typeof self);
        } catch(err) {
            result += ", self error: " + err.message;
        }
        return result;
    })(this, function(e) { return "inner function called with: " + e; })
    "#;
    
    let result = context.eval(Source::from_bytes(test_code)).unwrap();
    let str_val = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("UMD pattern test: {}", str_val);
}

#[test]
fn test_umd_exact_pattern() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test the exact pattern from cloudflare script line 2
    let test_code = r#"
    !function (t, e) { 
        "object" == typeof exports && "undefined" != typeof module 
            ? e(exports) 
            : "function" == typeof define && define.amd 
            ? define(["exports"], e) 
            : e((t = t || self).window = t.window || {}) 
    }(this, function (e) { 
        "use strict"; 
        return "success: exports object is " + typeof e; 
    })
    "#;
    
    match context.eval(Source::from_bytes(test_code)) {
        Ok(value) => {
            let str_val = value.to_string(&mut context).unwrap().to_std_string_escaped();
            println!("UMD exact pattern result: {}", str_val);
        },
        Err(e) => {
            println!("UMD exact pattern error: {:?}", e);
        }
    }
}

#[test]
fn test_cloudflare_main_beginning() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Read actual script
    let script = match std::fs::read_to_string("/tmp/cloudflare_main_script.js") {
        Ok(s) => s,
        Err(_) => {
            println!("Skipping - script file not found");
            return;
        }
    };
    
    // Get just line 2 (where the error is)
    let lines: Vec<&str> = script.lines().collect();
    if lines.len() < 2 {
        println!("Script has less than 2 lines");
        return;
    }
    
    let line2 = lines[1];
    println!("Line 2 length: {} chars", line2.len());
    println!("Line 2 start: {}", &line2[..200.min(line2.len())]);
    
    // Try executing just line 2
    match context.eval(Source::from_bytes(line2)) {
        Ok(value) => {
            let str_val = value.to_string(&mut context).unwrap().to_std_string_escaped();
            println!("Line 2 executed OK: {}", &str_val[..100.min(str_val.len())]);
        },
        Err(e) => {
            println!("Line 2 error: {:?}", e);
        }
    }
}

#[test]
fn test_first_500_chars() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // This is approximately the first 500 chars of line 2
    let test_code = r#"        !function (t, e) { "object" == typeof exports && "undefined" != typeof module ? e(exports) : "function" == typeof define && define.amd ? define(["exports"], e) : e((t = t || self).window = t.window || {}) }(this, function (e) { "use strict"; function _inheritsLoose(t, e) { t.prototype = Object.create(e.prototype), (t.prototype.constructor = t).__proto__ = e } function _assertThisInitialized(t) { if (void 0 === t) throw new ReferenceError("this hasn't been initialised - super() hasn't been called"); return t } return "done"; })"#;
    
    match context.eval(Source::from_bytes(test_code)) {
        Ok(value) => {
            let str_val = value.to_string(&mut context).unwrap().to_std_string_escaped();
            println!("First 500 chars executed OK: {}", str_val);
        },
        Err(e) => {
            println!("First 500 chars error: {:?}", e);
        }
    }
}

#[test]
fn test_find_failure_point() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Read line 2 of the script
    let script = match std::fs::read_to_string("/tmp/cloudflare_main_script.js") {
        Ok(s) => s,
        Err(_) => {
            println!("Skipping - script file not found");
            return;
        }
    };
    
    let lines: Vec<&str> = script.lines().collect();
    let line2 = lines[1];
    
    // Binary search for failure point
    let mut low = 0;
    let mut high = line2.len();
    let mut last_success = 0;
    
    // First, test at certain intervals to narrow down
    let test_points = [1000, 2000, 5000, 10000, 20000, 30000, 40000, 50000, 60000, 70000, 80000];
    
    for &point in &test_points {
        if point > line2.len() {
            break;
        }
        
        // Find a valid JavaScript boundary (end of expression/statement)
        let partial = &line2[..point];
        
        // Wrap in try-catch to make it valid
        let wrapped = format!("try {{ eval({:?}) }} catch(e) {{ 'error' }}", partial);
        
        match context.eval(Source::from_bytes(&wrapped)) {
            Ok(_) => {
                println!("✅ {} chars: OK", point);
                last_success = point;
            },
            Err(e) => {
                let err_str = format!("{:?}", e);
                if err_str.contains("not a callable") {
                    println!("❌ {} chars: not a callable function!", point);
                } else {
                    println!("⚠️ {} chars: {}", point, &err_str[..200.min(err_str.len())]);
                }
            }
        }
    }
    
    println!("\nLast successful point: {} chars", last_success);
}

#[test]
fn test_iife_structure() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Extract just the IIFE structure and replace the body with something simple
    // Original: !function (t, e) { ... }(this, function (e) { ... })
    let test_code = r#"
        !function (t, e) { 
            console.log("t =", typeof t);
            console.log("e =", typeof e);
            "object" == typeof exports && "undefined" != typeof module 
                ? e(exports) 
                : "function" == typeof define && define.amd 
                ? define(["exports"], e) 
                : e((t = t || self).window = t.window || {});
            return "IIFE executed";
        }(this, function (e) { 
            console.log("Inner function called with e =", typeof e);
            return "inner done"; 
        })
    "#;
    
    match context.eval(Source::from_bytes(test_code)) {
        Ok(value) => {
            let str_val = value.to_string(&mut context).unwrap().to_std_string_escaped();
            println!("Result: {}", str_val);
        },
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

#[test]
fn test_observers() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test ResizeObserver
    let result = context.eval(Source::from_bytes("typeof ResizeObserver")).unwrap();
    let type_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("typeof ResizeObserver: {}", type_str);
    
    // Test MutationObserver
    let result = context.eval(Source::from_bytes("typeof MutationObserver")).unwrap();
    let type_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("typeof MutationObserver: {}", type_str);
    
    // Test IntersectionObserver
    let result = context.eval(Source::from_bytes("typeof IntersectionObserver")).unwrap();
    let type_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("typeof IntersectionObserver: {}", type_str);
    
    // Test requestAnimationFrame
    let result = context.eval(Source::from_bytes("typeof requestAnimationFrame")).unwrap();
    let type_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("typeof requestAnimationFrame: {}", type_str);
    
    // Test creating a ResizeObserver
    match context.eval(Source::from_bytes("new ResizeObserver(function() {})")) {
        Ok(value) => println!("new ResizeObserver(): OK - {:?}", value.get_type()),
        Err(e) => println!("new ResizeObserver() error: {:?}", e),
    }
}

#[test]
fn test_script_with_error_capture() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let script = match std::fs::read_to_string("/tmp/cloudflare_main_script.js") {
        Ok(s) => s,
        Err(_) => {
            println!("Skipping - script file not found");
            return;
        }
    };
    
    // Wrap script in error catching
    let wrapped = format!(r#"
        var __errorInfo = null;
        try {{
            {}
        }} catch(e) {{
            __errorInfo = {{
                message: e.message,
                name: e.name,
                stack: e.stack
            }};
        }}
        __errorInfo
    "#, script);
    
    match context.eval(Source::from_bytes(&wrapped)) {
        Ok(value) => {
            if value.is_null() {
                println!("✅ Script executed successfully (no error caught)");
            } else {
                println!("❌ Script threw an error:");
                // Try to get error details
                if let Some(obj) = value.as_object() {
                    if let Ok(msg) = obj.get(boa_engine::js_string!("message"), &mut context) {
                        println!("  message: {}", msg.to_string(&mut context).unwrap().to_std_string_escaped());
                    }
                    if let Ok(name) = obj.get(boa_engine::js_string!("name"), &mut context) {
                        println!("  name: {}", name.to_string(&mut context).unwrap().to_std_string_escaped());
                    }
                    if let Ok(stack) = obj.get(boa_engine::js_string!("stack"), &mut context) {
                        let stack_str = stack.to_string(&mut context).unwrap().to_std_string_escaped();
                        println!("  stack: {}", &stack_str[..500.min(stack_str.len())]);
                    }
                }
            }
        },
        Err(e) => {
            println!("❌ Native error during eval: {:?}", e);
        }
    }
}

#[test]
fn test_script_lines_separately() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let script = match std::fs::read_to_string("/tmp/cloudflare_main_script.js") {
        Ok(s) => s,
        Err(_) => {
            println!("Skipping - script file not found");
            return;
        }
    };
    
    let lines: Vec<&str> = script.lines().collect();
    println!("Total lines: {}", lines.len());
    
    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            println!("Line {}: empty", i+1);
            continue;
        }
        
        println!("Line {}: {} chars", i+1, line.len());
        
        match context.eval(Source::from_bytes(line)) {
            Ok(_) => {
                println!("  ✅ OK");
            },
            Err(e) => {
                let err_str = format!("{:?}", e);
                if err_str.contains("not a callable") {
                    println!("  ❌ not a callable function!");
                } else {
                    println!("  ⚠️ Error: {}", &err_str[..200.min(err_str.len())]);
                }
            }
        }
    }
}

#[test]
fn test_binary_search_line2() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let script = match std::fs::read_to_string("/tmp/cloudflare_main_script.js") {
        Ok(s) => s,
        Err(_) => {
            println!("Skipping - script file not found");
            return;
        }
    };
    
    let lines: Vec<&str> = script.lines().collect();
    let line2 = lines[1];
    
    println!("Line 2 length: {} chars", line2.len());
    
    // Binary search for the failure point
    let mut low = 1000;  // We know 500 chars works
    let mut high = line2.len();
    
    while high - low > 100 {
        let mid = (low + high) / 2;
        
        // Find the nearest function boundary (ending with })
        let mut test_end = mid;
        for i in (mid..mid+500.min(line2.len()-mid)).rev() {
            if i < line2.len() && line2.as_bytes()[i] == b'}' {
                test_end = i + 1;
                break;
            }
        }
        
        // Create a test by appending a return statement 
        let partial = &line2[..test_end];
        let test_code = format!("try {{ (function() {{ {} }})() }} catch(e) {{ e.message }}", partial);
        
        match context.eval(Source::from_bytes(&test_code)) {
            Ok(value) => {
                let val_str = value.to_string(&mut context).unwrap().to_std_string_escaped();
                if val_str.contains("not a callable") {
                    println!("{} chars: ❌ not callable", test_end);
                    high = test_end;
                } else {
                    println!("{} chars: ✅", test_end);
                    low = test_end;
                }
            },
            Err(e) => {
                let err = format!("{:?}", e);
                if err.contains("Syntax") {
                    // Syntax error - we cut at wrong boundary
                    println!("{} chars: ⚠️ syntax error, adjusting", test_end);
                    low = test_end;
                } else {
                    println!("{} chars: ❓ {:?}", test_end, &err[..100.min(err.len())]);
                    low = test_end;
                }
            }
        }
    }
    
    println!("\nFailure somewhere around {} chars", (low + high) / 2);
}

#[test]
fn test_dom_apis_used_by_lenis() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test getComputedStyle
    let result = context.eval(Source::from_bytes("typeof getComputedStyle")).unwrap();
    println!("typeof getComputedStyle: {}", result.to_string(&mut context).unwrap().to_std_string_escaped());
    
    // Test window.getComputedStyle
    let result = context.eval(Source::from_bytes("typeof window.getComputedStyle")).unwrap();
    println!("typeof window.getComputedStyle: {}", result.to_string(&mut context).unwrap().to_std_string_escaped());
    
    // Test Element.prototype.getBoundingClientRect
    let result = context.eval(Source::from_bytes(r#"
        var el = document.createElement('div');
        typeof el.getBoundingClientRect
    "#)).unwrap();
    println!("typeof element.getBoundingClientRect: {}", result.to_string(&mut context).unwrap().to_std_string_escaped());

    // Test Element.prototype.classList
    let result = context.eval(Source::from_bytes(r#"
        var el = document.createElement('div');
        typeof el.classList
    "#)).unwrap();
    println!("typeof element.classList: {}", result.to_string(&mut context).unwrap().to_std_string_escaped());

    // Test scrollTo
    let result = context.eval(Source::from_bytes("typeof window.scrollTo")).unwrap();
    println!("typeof window.scrollTo: {}", result.to_string(&mut context).unwrap().to_std_string_escaped());

    // Test offsetParent
    let result = context.eval(Source::from_bytes(r#"
        var el = document.createElement('div');
        typeof el.offsetParent
    "#)).unwrap();
    println!("typeof element.offsetParent: {}", result.to_string(&mut context).unwrap().to_std_string_escaped());
}

#[test]
fn test_scroll_apis() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let apis = [
        "typeof window.scrollTo",
        "typeof window.scrollBy", 
        "typeof window.scroll",
        "typeof window.scrollX",
        "typeof window.scrollY",
        "typeof document.scrollingElement",
        "typeof document.documentElement",
    ];
    
    for api in apis {
        let result = context.eval(Source::from_bytes(api)).unwrap();
        let type_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
        println!("{}: {}", api, type_str);
    }
}

#[test]
fn test_element_scroll_properties() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let test_code = r#"
    var el = document.createElement('div');
    var props = [
        'scrollTop', 'scrollLeft', 'scrollWidth', 'scrollHeight',
        'offsetTop', 'offsetLeft', 'offsetWidth', 'offsetHeight', 'offsetParent',
        'clientTop', 'clientLeft', 'clientWidth', 'clientHeight'
    ];
    var results = [];
    for (var i = 0; i < props.length; i++) {
        results.push(props[i] + ': ' + typeof el[props[i]]);
    }
    results.join('\n');
    "#;
    
    let result = context.eval(Source::from_bytes(test_code)).unwrap();
    let str_val = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("Element scroll/offset properties:\n{}", str_val);
}

#[test]
fn test_instrumented_script() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let script = match std::fs::read_to_string("/tmp/cloudflare_main_script.js") {
        Ok(s) => s,
        Err(_) => {
            println!("Skipping - script file not found");
            return;
        }
    };

    let lines: Vec<&str> = script.lines().collect();
    let line2 = lines[1].trim();  // Get just line 2 (the GSAP script)

    println!("=== Running GSAP script directly ===");
    println!("Script length: {} chars", line2.len());

    // Try running the script directly first
    match context.eval(Source::from_bytes(line2)) {
        Ok(v) => {
            println!("Direct execution success: {:?}", v.get_type());
        }
        Err(e) => {
            println!("Direct execution error: {}", e);
            // Print more details
            let err_str = format!("{:?}", e);
            if err_str.contains("line_number") {
                println!("Error details: {}", err_str);
            }
        }
    }

    // Also test with the try-catch wrapper
    println!("\n=== Running with try-catch wrapper ===");
    let lines: Vec<&str> = script.lines().collect();
    let line2 = lines[1];
    
    // Wrap entire line 2 in try-catch that prints error location
    let instrumented = format!(r#"
        var __errorLocation = null;
        try {{
            {}
        }} catch(e) {{
            __errorLocation = {{
                message: e.message || String(e),
                name: e.name || 'Error',
                toString: e.toString ? e.toString() : String(e)
            }};
            throw e;
        }}
    "#, line2);
    
    match context.eval(Source::from_bytes(&instrumented)) {
        Ok(_) => println!("✅ Script executed successfully"),
        Err(e) => {
            // Get the error location we captured
            if let Ok(loc) = context.global_object().get(boa_engine::js_string!("__errorLocation"), &mut context) {
                if let Some(obj) = loc.as_object() {
                    if let Ok(msg) = obj.get(boa_engine::js_string!("message"), &mut context) {
                        println!("Captured error: {}", msg.to_string(&mut context).unwrap().to_std_string_escaped());
                    }
                    if let Ok(ts) = obj.get(boa_engine::js_string!("toString"), &mut context) {
                        println!("Error toString: {}", ts.to_string(&mut context).unwrap().to_std_string_escaped());
                    }
                }
            }
            println!("Native error: {:?}", e);
        }
    }
}

#[test]
fn test_gsap_dependencies() {
    use thalora_browser_apis::boa_engine::{Context, Source, js_string};

    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test all the global functions GSAP needs
    let deps_test = r#"
        var results = {};

        // Animation frame APIs
        results.requestAnimationFrame = typeof requestAnimationFrame;
        results.cancelAnimationFrame = typeof cancelAnimationFrame;

        // Timing APIs
        results.setTimeout = typeof setTimeout;
        results.clearTimeout = typeof clearTimeout;
        results.setInterval = typeof setInterval;
        results.clearInterval = typeof clearInterval;
        results.performance = typeof performance;
        results.performanceNow = performance ? typeof performance.now : 'no performance';

        // Date
        results.DateNow = typeof Date.now;

        // Document APIs
        results.createElementNS = typeof document.createElementNS;
        results.querySelector = typeof document.querySelector;
        results.querySelectorAll = typeof document.querySelectorAll;
        results.documentElement = typeof document.documentElement;
        results.body = typeof document.body;

        // Element APIs
        var div = document.createElement('div');
        results.getBoundingClientRect = typeof div.getBoundingClientRect;
        results.getComputedStyle = typeof getComputedStyle;
        results.addEventListener = typeof div.addEventListener;
        results.removeEventListener = typeof div.removeEventListener;

        // Math
        results.MathRandom = typeof Math.random;
        results.MathMax = typeof Math.max;

        // Object
        results.ObjectKeys = typeof Object.keys;
        results.ObjectCreate = typeof Object.create;
        results.ObjectDefineProperty = typeof Object.defineProperty;

        // Array
        results.ArrayFrom = typeof Array.from;
        results.ArrayIsArray = typeof Array.isArray;

        // Function
        results.FunctionBind = typeof Function.prototype.bind;

        // Proxy (used by some GSAP features)
        results.Proxy = typeof Proxy;

        // MutationObserver
        results.MutationObserver = typeof MutationObserver;

        // ResizeObserver
        results.ResizeObserver = typeof ResizeObserver;

        // IntersectionObserver
        results.IntersectionObserver = typeof IntersectionObserver;

        // matchMedia
        results.matchMedia = typeof matchMedia;

        JSON.stringify(results);
    "#;

    let result = context.eval(Source::from_bytes(deps_test)).unwrap();
    let result_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("GSAP dependencies test:\n{}", result_str);

    // Check critical ones
    assert!(result_str.contains("\"requestAnimationFrame\":\"function\""), "requestAnimationFrame must be a function");
    assert!(result_str.contains("\"setTimeout\":\"function\""), "setTimeout must be a function");
    assert!(result_str.contains("\"getComputedStyle\":\"function\""), "getComputedStyle must be a function");
}

#[test]
fn test_turnstile_api_script() {
    use thalora_browser_apis::boa_engine::{Context, Source, js_string};

    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // The error from the scrape test showed the Turnstile API script failing
    // Let's check what APIs it might need that we don't have

    let deps_test = r#"
        var results = {};

        // APIs that Cloudflare Turnstile commonly uses
        results.fetch = typeof fetch;
        results.Promise = typeof Promise;
        results.WeakMap = typeof WeakMap;
        results.WeakSet = typeof WeakSet;
        results.Map = typeof Map;
        results.Set = typeof Set;
        results.Uint8Array = typeof Uint8Array;
        results.ArrayBuffer = typeof ArrayBuffer;
        results.DataView = typeof DataView;
        results.TextEncoder = typeof TextEncoder;
        results.TextDecoder = typeof TextDecoder;
        results.atob = typeof atob;
        results.btoa = typeof btoa;
        results.crypto = typeof crypto;
        results.cryptoSubtle = crypto ? typeof crypto.subtle : 'no crypto';
        results.cryptoGetRandomValues = crypto ? typeof crypto.getRandomValues : 'no crypto';

        // DOM APIs
        results.MutationObserver = typeof MutationObserver;
        results.ResizeObserver = typeof ResizeObserver;
        results.IntersectionObserver = typeof IntersectionObserver;
        results.customElements = typeof customElements;
        results.ShadowRoot = typeof ShadowRoot;

        // Window/Document APIs
        results.postMessage = typeof postMessage;
        results.addEventListener = typeof addEventListener;
        results.MessageChannel = typeof MessageChannel;
        results.MessagePort = typeof MessagePort;
        results.BroadcastChannel = typeof BroadcastChannel;

        // Other APIs
        results.Blob = typeof Blob;
        results.File = typeof File;
        results.FileReader = typeof FileReader;
        results.URL = typeof URL;
        results.URLSearchParams = typeof URLSearchParams;
        results.FormData = typeof FormData;
        results.Headers = typeof Headers;
        results.Request = typeof Request;
        results.Response = typeof Response;

        // Check for specific patterns Turnstile uses
        results.defineProperty = typeof Object.defineProperty;
        results.getOwnPropertyDescriptor = typeof Object.getOwnPropertyDescriptor;
        results.Reflect = typeof Reflect;

        JSON.stringify(results);
    "#;

    let result = context.eval(Source::from_bytes(deps_test)).unwrap();
    let result_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("Turnstile API dependencies:\n{}", result_str);

    // Parse and check for undefined ones
    let parsed: serde_json::Value = serde_json::from_str(&result_str).unwrap();
    println!("\n=== Missing or undefined APIs ===");
    if let serde_json::Value::Object(map) = parsed {
        for (key, value) in map {
            if value == "undefined" {
                println!("❌ {}: undefined", key);
            }
        }
    }
}

#[test]
fn test_string_substr() {
    use thalora_browser_apis::boa_engine::{Context, Source, js_string};

    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test String.prototype.substr - GSAP uses this
    let test = r#"
        var s = "hello";
        var results = {
            substr: typeof s.substr,
            substring: typeof s.substring,
            slice: typeof s.slice,
            charAt: typeof s.charAt,
            charCodeAt: typeof s.charCodeAt,
            toUpperCase: typeof s.toUpperCase,
            toLowerCase: typeof s.toLowerCase,
            substrResult: s.substr(1),
            substringResult: s.substring(1),
            sliceResult: s.slice(1),
            charAtResult: s.charAt(0),
            toUpperCaseResult: s.toUpperCase()
        };
        JSON.stringify(results);
    "#;

    let result = context.eval(Source::from_bytes(test)).unwrap();
    let result_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("String methods test: {}", result_str);

    // Check specific issue
    assert!(result_str.contains("\"substr\":\"function\""), "substr must be a function");
}

#[test]
fn test_gsap_factory_call() {
    use thalora_browser_apis::boa_engine::{Context, Source, js_string};

    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test the exact UMD wrapper pattern with a simpler factory
    println!("=== Testing GSAP-style UMD factory call ===");

    // This is the exact UMD wrapper structure from GSAP
    let test1 = r#"
        !function (t, e) {
            "object" == typeof exports && "undefined" != typeof module
                ? e(exports)
                : "function" == typeof define && define.amd
                    ? define(["exports"], e)
                    : e((t = t || self).window = t.window || {})
        }(this, function (e) {
            "use strict";
            e.testValue = 123;
        });
        window.testValue
    "#;

    match context.eval(Source::from_bytes(test1)) {
        Ok(v) => println!("Basic UMD success: {}", v.to_string(&mut context).unwrap().to_std_string_escaped()),
        Err(e) => println!("Basic UMD error: {:?}", e),
    }

    // Test with the GSAP _inheritsLoose pattern
    let test2 = r#"
        !function (t, e) {
            "object" == typeof exports && "undefined" != typeof module
                ? e(exports)
                : "function" == typeof define && define.amd
                    ? define(["exports"], e)
                    : e((t = t || self).window = t.window || {})
        }(this, function (e) {
            "use strict";
            function _inheritsLoose(t, e) {
                t.prototype = Object.create(e.prototype);
                (t.prototype.constructor = t).__proto__ = e;
            }
            e._inheritsLoose = _inheritsLoose;
        });
        typeof window._inheritsLoose
    "#;

    match context.eval(Source::from_bytes(test2)) {
        Ok(v) => println!("_inheritsLoose UMD success: {}", v.to_string(&mut context).unwrap().to_std_string_escaped()),
        Err(e) => println!("_inheritsLoose UMD error: {:?}", e),
    }

    // Test the __proto__ assignment specifically
    let test3 = r#"
        (function() {
            function Parent() {}
            function Child() {}
            Child.prototype = Object.create(Parent.prototype);
            (Child.prototype.constructor = Child).__proto__ = Parent;
            return typeof Child.__proto__;
        })()
    "#;

    match context.eval(Source::from_bytes(test3)) {
        Ok(v) => println!("__proto__ assignment: {}", v.to_string(&mut context).unwrap().to_std_string_escaped()),
        Err(e) => println!("__proto__ error: {:?}", e),
    }
}

#[test]
fn test_umd_wrapper_debug() {
    use thalora_browser_apis::boa_engine::{Context, Source, js_string};

    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test the UMD pattern piece by piece
    println!("=== UMD Debugging ===");

    // Check `this`
    let result = context.eval(Source::from_bytes("typeof this")).unwrap();
    println!("typeof this: {}", result.to_string(&mut context).unwrap().to_std_string_escaped());

    // Check `self`
    let result = context.eval(Source::from_bytes("typeof self")).unwrap();
    println!("typeof self: {}", result.to_string(&mut context).unwrap().to_std_string_escaped());

    // Check `exports`
    let result = context.eval(Source::from_bytes("typeof exports")).unwrap();
    println!("typeof exports: {}", result.to_string(&mut context).unwrap().to_std_string_escaped());

    // Check `module`
    let result = context.eval(Source::from_bytes("typeof module")).unwrap();
    println!("typeof module: {}", result.to_string(&mut context).unwrap().to_std_string_escaped());

    // Check `define`
    let result = context.eval(Source::from_bytes("typeof define")).unwrap();
    println!("typeof define: {}", result.to_string(&mut context).unwrap().to_std_string_escaped());

    // Test the actual UMD pattern
    let umd_test = r#"
        (function() {
            var result = {};

            // The actual UMD check
            var t = this;
            var hasCommonJS = "object" == typeof exports && "undefined" != typeof module;
            var hasAMD = "function" == typeof define && define && define.amd;

            result.hasCommonJS = hasCommonJS;
            result.hasAMD = hasAMD;
            result.thisType = typeof t;
            result.selfType = typeof self;

            // Test the fallback path
            try {
                var globalObj = t || self;
                result.globalObjType = typeof globalObj;
                result.globalObjWindowType = typeof globalObj.window;

                // This is what the UMD does
                globalObj.window = globalObj.window || {};
                result.afterAssignWindowType = typeof globalObj.window;
            } catch(e) {
                result.error = e.message;
            }

            return JSON.stringify(result);
        })()
    "#;

    let result = context.eval(Source::from_bytes(umd_test)).unwrap();
    let result_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("UMD pattern test: {}", result_str);

    // Now test the actual UMD wrapper call
    let full_umd = r#"
        (function() {
            try {
                !function (t, e) {
                    "object" == typeof exports && "undefined" != typeof module
                        ? e(exports)
                        : "function" == typeof define && define.amd
                            ? define(["exports"], e)
                            : e((t = t || self).window = t.window || {})
                }(this, function (e) {
                    e.testValue = 42;
                });
                return "success: " + (window.testValue || "no testValue");
            } catch(err) {
                return "error: " + err.message + " | " + err.toString();
            }
        })()
    "#;

    let result = context.eval(Source::from_bytes(full_umd)).unwrap();
    let result_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("Full UMD test: {}", result_str);
}

#[test]
fn test_element_layout_properties() {
    use thalora_browser_apis::boa_engine::{Context, Source, js_string};

    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test that layout properties exist and return numbers
    let test_code = r#"
        var div = document.createElement('div');
        var results = {
            // offset properties
            offsetWidth: typeof div.offsetWidth,
            offsetHeight: typeof div.offsetHeight,
            offsetTop: typeof div.offsetTop,
            offsetLeft: typeof div.offsetLeft,
            offsetParent: div.offsetParent,

            // client properties
            clientWidth: typeof div.clientWidth,
            clientHeight: typeof div.clientHeight,
            clientTop: typeof div.clientTop,
            clientLeft: typeof div.clientLeft,

            // scroll properties
            scrollWidth: typeof div.scrollWidth,
            scrollHeight: typeof div.scrollHeight,
            scrollTop: typeof div.scrollTop,
            scrollLeft: typeof div.scrollLeft,

            // Test setting scroll properties
            canSetScrollTop: (function() {
                div.scrollTop = 100;
                return div.scrollTop >= 0;
            })(),
            canSetScrollLeft: (function() {
                div.scrollLeft = 50;
                return div.scrollLeft >= 0;
            })()
        };
        JSON.stringify(results);
    "#;

    let result = context.eval(Source::from_bytes(test_code)).unwrap();
    let result_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("Layout properties test results: {}", result_str);

    // Parse the JSON and verify
    assert!(result_str.contains("\"offsetWidth\":\"number\""), "offsetWidth should be number");
    assert!(result_str.contains("\"offsetHeight\":\"number\""), "offsetHeight should be number");
    assert!(result_str.contains("\"clientWidth\":\"number\""), "clientWidth should be number");
    assert!(result_str.contains("\"clientHeight\":\"number\""), "clientHeight should be number");
    assert!(result_str.contains("\"scrollTop\":\"number\""), "scrollTop should be number");
    assert!(result_str.contains("\"scrollLeft\":\"number\""), "scrollLeft should be number");
    assert!(result_str.contains("\"canSetScrollTop\":true"), "Should be able to set scrollTop");
    assert!(result_str.contains("\"canSetScrollLeft\":true"), "Should be able to set scrollLeft");

    println!("✅ All layout properties work correctly!");
}

#[test]
fn test_parentnode_methods() {
    use thalora_browser_apis::boa_engine::{Context, Source};

    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test ParentNode methods that Cloudflare might need
    let test_code = r#"
        var div = document.createElement('div');
        var results = {
            append: typeof div.append,
            prepend: typeof div.prepend,
            replaceChildren: typeof div.replaceChildren,
            after: typeof div.after,
            before: typeof div.before,
            remove: typeof div.remove,
            replaceWith: typeof div.replaceWith,
        };
        JSON.stringify(results);
    "#;

    let result = context.eval(Source::from_bytes(test_code)).unwrap();
    let result_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("ParentNode methods: {}", result_str);
}

#[test]
fn test_element_animate() {
    use thalora_browser_apis::boa_engine::{Context, Source};

    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test Web Animations API
    let test_code = r#"
        var div = document.createElement('div');
        var results = {
            animate: typeof div.animate,
            Animation: typeof Animation,
            KeyframeEffect: typeof KeyframeEffect,
            DocumentTimeline: typeof DocumentTimeline,
            getAnimations: typeof div.getAnimations,
        };
        JSON.stringify(results);
    "#;

    let result = context.eval(Source::from_bytes(test_code)).unwrap();
    let result_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("Web Animations API: {}", result_str);
}

#[test]
fn test_document_special_apis() {
    use thalora_browser_apis::boa_engine::{Context, Source};

    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test special document APIs that Cloudflare might use
    let test_code = r#"
        var results = {
            fonts: typeof document.fonts,
            adoptedStyleSheets: typeof document.adoptedStyleSheets,
            styleSheets: typeof document.styleSheets,
            elementFromPoint: typeof document.elementFromPoint,
            elementsFromPoint: typeof document.elementsFromPoint,
            caretRangeFromPoint: typeof document.caretRangeFromPoint,
            getSelection: typeof document.getSelection,
            caretPositionFromPoint: typeof document.caretPositionFromPoint,
            hidden: typeof document.hidden,
            visibilityState: typeof document.visibilityState,
        };
        JSON.stringify(results);
    "#;

    let result = context.eval(Source::from_bytes(test_code)).unwrap();
    let result_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("Document special APIs: {}", result_str);
}

#[test]
fn test_htmlcollection_array_methods() {
    use thalora_browser_apis::boa_engine::{Context, Source};

    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test HTMLCollection array-like access patterns
    let test_code = r#"
        var scripts = document.scripts;
        var results = {
            scriptsType: typeof scripts,
            scriptsLength: typeof scripts.length,
            scriptsItem: typeof scripts.item,
            scriptsNamedItem: typeof scripts.namedItem,
            scriptsForEach: typeof scripts.forEach,
            scriptsIterator: typeof scripts[Symbol.iterator],
        };
        JSON.stringify(results);
    "#;

    let result = context.eval(Source::from_bytes(test_code)).unwrap();
    let result_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("HTMLCollection methods: {}", result_str);
}

#[test]
fn test_cssom_apis() {
    use thalora_browser_apis::boa_engine::{Context, Source};

    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test CSSOM APIs that Cloudflare might use
    let test_code = r#"
        var div = document.createElement('div');
        document.body.appendChild(div);
        var results = {};

        // getComputedStyle
        try {
            var style = getComputedStyle(div);
            results.computedStyle = typeof style;
            results.computedStyleGetPropertyValue = typeof style.getPropertyValue;
        } catch(e) {
            results.computedStyleError = e.message;
        }

        // matchMedia
        try {
            var mq = matchMedia('(prefers-color-scheme: dark)');
            results.matchMedia = typeof mq;
            results.matchMediaMatches = typeof mq.matches;
            results.matchMediaAddListener = typeof mq.addListener;
            results.matchMediaAddEventListener = typeof mq.addEventListener;
        } catch(e) {
            results.matchMediaError = e.message;
        }

        JSON.stringify(results);
    "#;

    let result = context.eval(Source::from_bytes(test_code)).unwrap();
    let result_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("CSSOM APIs: {}", result_str);
}

#[test]
fn test_raf_callback_with_args() {
    use thalora_browser_apis::boa_engine::{Context, Source};

    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test that requestAnimationFrame passes a timestamp to callback
    context.eval(Source::from_bytes(r#"
        window._rafCallbackArgs = null;
        requestAnimationFrame(function(timestamp) {
            window._rafCallbackArgs = {
                argCount: arguments.length,
                timestampType: typeof timestamp,
                timestampValue: timestamp
            };
        });
    "#)).unwrap();

    // Process timers
    use thalora_browser_apis::timers::timers::Timers;
    Timers::process_timers(&mut context);

    let result = context.eval(Source::from_bytes(r#"
        window._rafCallbackArgs ? JSON.stringify(window._rafCallbackArgs) : 'callback not called'
    "#)).unwrap();
    let result_str = result.to_string(&mut context).unwrap().to_std_string_escaped();
    println!("requestAnimationFrame callback args: {}", result_str);
}
