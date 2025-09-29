use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=v8-source");

    // Always force building V8 from source (never use precompiled binaries)
    env::set_var("V8_FROM_SOURCE", "1");
    
    // Print build information
    println!("cargo:warning=Thalora V8: Building from custom nightness/v8 fork");
    println!("cargo:warning=Source location: engines/v8/v8-source (git submodule)");
    println!("cargo:warning=Build mode: Always from source (V8_FROM_SOURCE=1)");
    println!("cargo:warning=First compile may take 15-30 minutes");
    println!("cargo:warning=Subsequent builds will be much faster");
}