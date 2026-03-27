use boa_engine::{Context, Source};

#[test]
fn debug_worker_availability_in_contexts() {
    println!("🔍 Debugging Worker API availability in different contexts...");

    // Test 1: Context with browser APIs initialized
    println!("\n📋 Test 1: Context with initialize_browser_apis()");
    let mut default_context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut default_context)
        .expect("Failed to initialize browser APIs");
    let result = default_context.eval(Source::from_bytes("typeof Worker"));
    match result {
        Ok(value) => {
            let type_str = value
                .to_string(&mut default_context)
                .unwrap()
                .to_std_string_escaped();
            println!("   Worker type in initialized context: '{}'", type_str);
            // Worker is only available with the 'native' feature
            #[cfg(feature = "native")]
            assert_eq!(
                type_str, "function",
                "Worker should be available with native feature"
            );
            #[cfg(not(feature = "native"))]
            println!("   Worker not available without 'native' feature (expected)");
        }
        Err(e) => {
            println!("   Error in default context: {:?}", e);
            panic!("Eval should not fail");
        }
    }

    // Test 2: Builder Context (what Thalora uses)
    println!("\n📋 Test 2: Context::builder().build()");
    let mut builder_context = Context::builder().build().unwrap();
    let result = builder_context.eval(Source::from_bytes("typeof Worker"));
    match result {
        Ok(value) => {
            let type_str = value
                .to_string(&mut builder_context)
                .unwrap()
                .to_std_string_escaped();
            println!("   Worker type in builder context: '{}'", type_str);
            // This should be "function" but might be "undefined" - that's the bug we're investigating
        }
        Err(e) => println!("   Error in builder context: {:?}", e),
    }

    // Test 3: Check if intrinsics have Worker
    println!("\n📋 Test 3: Intrinsics check");
    let builder_context2 = Context::builder().build().unwrap();
    let worker_constructor = builder_context2.intrinsics().constructors().worker();
    println!("   Worker constructor available in intrinsics: true");

    // Test 4: Check WebSocket for comparison
    println!("\n📋 Test 4: WebSocket comparison");
    let mut builder_context3 = Context::builder().build().unwrap();
    let result = builder_context3.eval(Source::from_bytes("typeof WebSocket"));
    match result {
        Ok(value) => {
            let type_str = value
                .to_string(&mut builder_context3)
                .unwrap()
                .to_std_string_escaped();
            println!("   WebSocket type in builder context: '{}'", type_str);
        }
        Err(e) => println!("   Error checking WebSocket: {:?}", e),
    }

    println!("\n🎯 Analysis complete!");
}
