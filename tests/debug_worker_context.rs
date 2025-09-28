use boa_engine::{Context, Source};

#[test]
fn debug_worker_availability_in_contexts() {
    println!("🔍 Debugging Worker API availability in different contexts...");

    // Test 1: Default Context (what Boa unit tests use)
    println!("\n📋 Test 1: Context::default()");
    let mut default_context = Context::default();
    let result = default_context.eval(Source::from_bytes("typeof Worker"));
    match result {
        Ok(value) => {
            let type_str = value.to_string(&mut default_context).unwrap().to_std_string_escaped();
            println!("   Worker type in default context: '{}'", type_str);
            assert_eq!(type_str, "function", "Worker should be available in default context");
        },
        Err(e) => {
            println!("   Error in default context: {:?}", e);
            panic!("Worker should be available in default context");
        },
    }

    // Test 2: Builder Context (what Thalora uses)
    println!("\n📋 Test 2: Context::builder().build()");
    let mut builder_context = Context::builder().build().unwrap();
    let result = builder_context.eval(Source::from_bytes("typeof Worker"));
    match result {
        Ok(value) => {
            let type_str = value.to_string(&mut builder_context).unwrap().to_std_string_escaped();
            println!("   Worker type in builder context: '{}'", type_str);
            // This should be "function" but might be "undefined" - that's the bug we're investigating
        },
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
            let type_str = value.to_string(&mut builder_context3).unwrap().to_std_string_escaped();
            println!("   WebSocket type in builder context: '{}'", type_str);
        },
        Err(e) => println!("   Error checking WebSocket: {:?}", e),
    }

    println!("\n🎯 Analysis complete!");
}