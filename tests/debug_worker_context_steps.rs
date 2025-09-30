use boa_engine::{Context, Source};
use thalora::apis::{WebApis, polyfills, dom_native};

#[test]
fn debug_worker_availability_through_thalora_setup() {
    println!("🔍 Debugging Worker API availability through Thalora's setup steps...");

    // Start with a fresh context like Thalora does
    let mut context = Context::builder()
        .build()
        .expect("failed to build JS context");

    // Helper function to check Worker availability
    let check_worker = |context: &mut Context, step: &str| {
        let result = context.eval(Source::from_bytes("typeof Worker"));
        match result {
            Ok(value) => {
                let type_str = value.to_string(context).unwrap().to_std_string_escaped();
                println!("   {} - Worker type: '{}'", step, type_str);
                type_str == "function"
            },
            Err(e) => {
                println!("   {} - Error: {:?}", step, e);
                false
            }
        }
    };

    // Check initial state (should be function from Boa's set_default_global_bindings)
    let initial = check_worker(&mut context, "Initial Context");
    assert!(initial, "Worker should be available initially");

    // Step 1: setup_all_polyfills
    println!("\n📋 Step 1: Running setup_all_polyfills...");
    polyfills::setup_all_polyfills(&mut context).unwrap();
    let after_polyfills = check_worker(&mut context, "After polyfills");

    // Step 2: WebApis::setup_all_apis
    println!("\n📋 Step 2: Running WebApis::setup_all_apis...");
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).unwrap();
    let after_webapis = check_worker(&mut context, "After WebApis");

    // Step 3: setup_native_dom_globals (SUSPECT!)
    println!("\n📋 Step 3: Running setup_native_dom_globals...");
    dom_native::setup_native_dom_globals(&mut context).unwrap();
    let after_dom = check_worker(&mut context, "After DOM setup");

    // Summary
    println!("\n🎯 Summary:");
    println!("   Initial:       Worker = {}", if initial { "✅ function" } else { "❌ undefined" });
    println!("   After polyfills:  Worker = {}", if after_polyfills { "✅ function" } else { "❌ undefined" });
    println!("   After WebApis:    Worker = {}", if after_webapis { "✅ function" } else { "❌ undefined" });
    println!("   After DOM setup:  Worker = {}", if after_dom { "✅ function" } else { "❌ undefined" });

    if !after_dom {
        println!("\n🚨 FOUND THE CULPRIT: setup_native_dom_globals is breaking Worker availability!");
    }
}