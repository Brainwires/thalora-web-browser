use thalora::apis::WebApis;
use boa_engine::{Context, Source};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    // Setup console first
    synaptic::apis::polyfills::console::setup_console(&mut context)?;

    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context)?;

    // Test each constructor individually
    println!("Testing WebAssembly.Memory...");
    match context.eval(Source::from_bytes("new WebAssembly.Memory()")) {
        Ok(result) => println!("✅ WebAssembly.Memory: {}", result.to_string(&mut context)?),
        Err(e) => println!("❌ WebAssembly.Memory: {}", e),
    }

    println!("Testing AudioContext...");
    match context.eval(Source::from_bytes("new AudioContext()")) {
        Ok(result) => println!("✅ AudioContext: {}", result.to_string(&mut context)?),
        Err(e) => println!("❌ AudioContext: {}", e),
    }

    println!("Testing RTCPeerConnection...");
    match context.eval(Source::from_bytes("new RTCPeerConnection()")) {
        Ok(result) => println!("✅ RTCPeerConnection: {}", result.to_string(&mut context)?),
        Err(e) => println!("❌ RTCPeerConnection: {}", e),
    }

    Ok(())
}