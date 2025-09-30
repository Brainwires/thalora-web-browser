use boa_engine::{Context, Source};

#[tokio::test]
async fn test_boa_missing_features() {
    let mut context = Context::default();

    let tests = vec![
        // Basic ES features that might be missing
        ("Array.from", "typeof Array.from === 'function'"),
        ("Array.includes", "typeof [].includes === 'function'"),
        ("Array.flat", "typeof [].flat === 'function'"),
        ("Array.flatMap", "typeof [].flatMap === 'function'"),
        ("String.includes", "typeof ''.includes === 'function'"),
        ("String.startsWith", "typeof ''.startsWith === 'function'"),
        ("String.endsWith", "typeof ''.endsWith === 'function'"),
        ("String.repeat", "typeof ''.repeat === 'function'"),
        ("String.padStart", "typeof ''.padStart === 'function'"),
        ("String.padEnd", "typeof ''.padEnd === 'function'"),

        // ES6+ Features
        ("Object.assign", "typeof Object.assign === 'function'"),
        ("Object.keys", "typeof Object.keys === 'function'"),
        ("Object.values", "typeof Object.values === 'function'"),
        ("Object.entries", "typeof Object.entries === 'function'"),
        ("Object.fromEntries", "typeof Object.fromEntries === 'function'"),

        // Promise
        ("Promise", "typeof Promise === 'function'"),
        ("Promise.resolve", "typeof Promise.resolve === 'function'"),
        ("Promise.reject", "typeof Promise.reject === 'function'"),
        ("Promise.all", "typeof Promise.all === 'function'"),
        ("Promise.race", "typeof Promise.race === 'function'"),
        ("Promise.allSettled", "typeof Promise.allSettled === 'function'"),

        // Map/Set
        ("Map", "typeof Map === 'function'"),
        ("Set", "typeof Set === 'function'"),
        ("WeakMap", "typeof WeakMap === 'function'"),
        ("WeakSet", "typeof WeakSet === 'function'"),

        // Symbol
        ("Symbol", "typeof Symbol === 'function'"),
        ("Symbol.iterator", "typeof Symbol.iterator === 'symbol'"),

        // Web APIs that are definitely missing
        ("fetch", "typeof fetch === 'function'"),
        ("URL", "typeof URL === 'function'"),
        ("URLSearchParams", "typeof URLSearchParams === 'function'"),
        ("setTimeout", "typeof setTimeout === 'function'"),
        ("setInterval", "typeof setInterval === 'function'"),
        ("clearTimeout", "typeof clearTimeout === 'function'"),
        ("clearInterval", "typeof clearInterval === 'function'"),

        // Web APIs
        ("localStorage", "typeof localStorage === 'object'"),
        ("sessionStorage", "typeof sessionStorage === 'object'"),
        ("atob", "typeof atob === 'function'"),
        ("btoa", "typeof btoa === 'function'"),
        ("crypto", "typeof crypto === 'object'"),
        ("TextEncoder", "typeof TextEncoder === 'function'"),
        ("TextDecoder", "typeof TextDecoder === 'function'"),
        ("AbortController", "typeof AbortController === 'function'"),
        ("Blob", "typeof Blob === 'function'"),
        ("File", "typeof File === 'function'"),
        ("FormData", "typeof FormData === 'function'"),
        ("Headers", "typeof Headers === 'function'"),
        ("Request", "typeof Request === 'function'"),
        ("Response", "typeof Response === 'function'"),
    ];

    println!("\nTesting Boa JavaScript feature support:");
    println!("=======================================");

    let mut missing_features = Vec::new();
    let mut supported_features = Vec::new();

    for (feature_name, test_code) in tests {
        match context.eval(Source::from_bytes(test_code)) {
            Ok(result) => {
                let supported = result.as_boolean().unwrap_or(false);
                if supported {
                    supported_features.push(feature_name);
                    println!("{:<20} ✅ SUPPORTED", feature_name);
                } else {
                    missing_features.push(feature_name);
                    println!("{:<20} ❌ MISSING", feature_name);
                }
            }
            Err(e) => {
                missing_features.push(feature_name);
                println!("{:<20} ❌ ERROR: {}", feature_name, e);
            }
        }
    }

    println!("\nTesting syntax features:");
    println!("========================");

    let syntax_tests = vec![
        ("Arrow functions", "(() => 42)()"),
        ("Destructuring", "const [a, b] = [1, 2]; a + b"),
        ("Template literals", "`hello ${1 + 1}`"),
        ("Spread operator", "[...[1, 2, 3]].length"),
        ("Rest parameters", "((...args) => args.length)(1, 2, 3)"),
        ("Default parameters", "((a = 5) => a)()"),
        ("Let/const", "{ let x = 1; const y = 2; x + y }"),
        ("For...of loop", "{ let sum = 0; for (const x of [1, 2, 3]) sum += x; sum }"),
        ("Classes", "class Test { constructor() { this.x = 1; } } new Test().x"),
        ("Async/await", "async function test() { return await Promise.resolve(42); }; typeof test === 'function'"),
    ];

    for (feature_name, test_code) in syntax_tests {
        match context.eval(Source::from_bytes(test_code)) {
            Ok(result) => {
                println!("{:<20} ✅ SUPPORTED (result: {})", feature_name, result.display());
            }
            Err(e) => {
                println!("{:<20} ❌ ERROR: {}", feature_name, e);
            }
        }
    }

    println!("\n=== SUMMARY ===");
    println!("Supported features: {}", supported_features.len());
    println!("Missing features: {}", missing_features.len());

    if !missing_features.is_empty() {
        println!("\nMissing features that need polyfills:");
        for feature in missing_features {
            println!("- {}", feature);
        }
    }
}
