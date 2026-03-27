use anyhow::Result;
use std::process::Command;

pub fn run() -> Result<()> {
    // Check if wasm-pack is installed
    if Command::new("wasm-pack").arg("--version").output().is_err() {
        eprintln!("wasm-pack not found. Installing...");
        super::run_cmd(Command::new("cargo").args(["install", "wasm-pack"]))?;
    }

    let targets = [
        ("bundler", "pkg/bundler"),
        ("nodejs", "pkg/nodejs"),
        ("web", "pkg/web"),
    ];

    println!("Building thalora-web-browser for WebAssembly...");
    for (target, out_dir) in &targets {
        println!("\nBuilding for {target}...");
        super::run_cmd(
            Command::new("wasm-pack")
                .args(["build", "--target", target, "--out-dir", out_dir])
                .args(["--features", "wasm", "--no-default-features"]),
        )?;
    }

    println!("\nWASM build complete!");
    println!("\nOutput directories:");
    println!("  - pkg/bundler  (for webpack/rollup/vite)");
    println!("  - pkg/nodejs   (for Node.js)");
    println!("  - pkg/web      (for ES modules)");
    Ok(())
}
