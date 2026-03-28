use boa_engine::{Context, Source};

fn main() {
    eprintln!("🔍 Debugging Worker API availability in different contexts...");

    // Test 1: Default Context (what Boa unit tests use)
    eprintln!("\n📋 Test 1: Context::default()");
    let mut default_context = Context::default();
    let result = default_context.eval(Source::from_bytes("typeof Worker"));
    match result {
        Ok(value) => {
            let type_str = value
                .to_string(&mut default_context)
                .unwrap()
                .to_std_string_escaped();
            eprintln!("   Worker type in default context: '{}'", type_str);
        }
        Err(e) => eprintln!("   Error in default context: {:?}", e),
    }

    // Test 2: Builder Context (what Thalora uses)
    eprintln!("\n📋 Test 2: Context::builder().build()");
    let mut builder_context = Context::builder().build().unwrap();
    let result = builder_context.eval(Source::from_bytes("typeof Worker"));
    match result {
        Ok(value) => {
            let type_str = value
                .to_string(&mut builder_context)
                .unwrap()
                .to_std_string_escaped();
            eprintln!("   Worker type in builder context: '{}'", type_str);
        }
        Err(e) => eprintln!("   Error in builder context: {:?}", e),
    }

    // Test 3: Check if intrinsics have Worker
    eprintln!("\n📋 Test 3: Intrinsics check");
    let builder_context2 = Context::builder().build().unwrap();
    let worker_constructor = builder_context2.intrinsics().constructors().worker();
    eprintln!("   Worker constructor available in intrinsics: true");
    let _ = worker_constructor; // constructor object available

    // Test 4: Check WebSocket for comparison
    eprintln!("\n📋 Test 4: WebSocket comparison");
    let mut builder_context3 = Context::builder().build().unwrap();
    let result = builder_context3.eval(Source::from_bytes("typeof WebSocket"));
    match result {
        Ok(value) => {
            let type_str = value
                .to_string(&mut builder_context3)
                .unwrap()
                .to_std_string_escaped();
            eprintln!("   WebSocket type in builder context: '{}'", type_str);
        }
        Err(e) => eprintln!("   Error checking WebSocket: {:?}", e),
    }

    eprintln!("\n🎯 Analysis complete!");
}
