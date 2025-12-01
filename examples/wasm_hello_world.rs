// NOTE: This example is currently disabled as WebAssembly support
// has been moved to the Boa JavaScript engine natively.
// WebAssembly modules can now be loaded and executed through the
// standard Boa engine via HeadlessWebBrowser.execute_javascript()

use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("🧪 WebAssembly Hello World Example");
    eprintln!("===================================");
    eprintln!("⚠️  NOTE: WebAssembly is now natively supported in Boa engine");
    eprintln!("    Use HeadlessWebBrowser.execute_javascript() to load WASM modules");

    let _start_total = Instant::now();

    eprintln!("\n📚 Example Code:");
    eprintln!("================");
    eprintln!("use thalora::engine::browser::HeadlessWebBrowser;");
    eprintln!("");
    eprintln!("let mut browser = HeadlessWebBrowser::new().await?;");
    eprintln!("let result = browser.execute_javascript(r#\"");
    eprintln!("  // Load and instantiate a WebAssembly module");
    eprintln!("  const wasmBytes = new Uint8Array([...]);");
    eprintln!("  const module = await WebAssembly.instantiate(wasmBytes);");
    eprintln!("  module.instance.exports.add(5, 3);");
    eprintln!("\"#).await?;");
    eprintln!("");
    eprintln!("✅ WebAssembly support is built into the Boa engine");
    eprintln!("   and can be accessed through standard JavaScript WebAssembly API");

    Ok(())
}