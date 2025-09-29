use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=V8_FROM_SOURCE");
    println!("cargo:rerun-if-env-changed=THALORA_V8_REPO");
    println!("cargo:rerun-if-env-changed=THALORA_V8_BRANCH");
    println!("cargo:rerun-if-env-changed=CUSTOM_V8_REPOSITORY");
    println!("cargo:rerun-if-env-changed=CUSTOM_V8_BRANCH");

    // Force building V8 from source instead of using prebuilt binaries
    env::set_var("V8_FROM_SOURCE", "1");
    
    // Configure custom V8 repository - prioritize THALORA_* env vars
    let custom_repo = env::var("THALORA_V8_REPO")
        .or_else(|_| env::var("CUSTOM_V8_REPOSITORY"))
        .unwrap_or_else(|_| "https://github.com/nightness/v8.git".to_string());
    
    let custom_branch = env::var("THALORA_V8_BRANCH")
        .or_else(|_| env::var("CUSTOM_V8_BRANCH"))
        .unwrap_or_else(|_| "main".to_string());
    
    // Set the V8 mirror to your custom fork
    env::set_var("RUSTY_V8_MIRROR", &custom_repo);
    
    // Configure V8 build arguments for better compatibility
    let gn_args = [
        "v8_enable_sandbox=false",
        "v8_expose_symbols=true", 
        "v8_use_external_startup_data=false",
        "v8_enable_pointer_compression=false",
        "use_custom_libcxx=false",
        "treat_warnings_as_errors=false"
    ].join(" ");
    env::set_var("GN_ARGS", gn_args);
    
    // Print build configuration
    println!("cargo:warning=Thalora V8 Build Configuration:");
    println!("cargo:warning=  Repository: {}", custom_repo);
    println!("cargo:warning=  Branch: {}", custom_branch);
    println!("cargo:warning=  Building from source: enabled");
    println!("cargo:warning=  Pre-built binaries: disabled");
    println!("cargo:warning=");
    println!("cargo:warning=V8 build may take 15-30 minutes on first compile");
    println!("cargo:warning=Ensure you have: Python 3.6+, ninja-build, git, and C++ compiler");
    
    // Check build requirements
    check_build_requirements();
}

fn check_build_requirements() {
    // Check for Python (required by V8 build system)
    if which::which("python3").is_err() && which::which("python").is_err() {
        println!("cargo:warning=WARNING: Python not found - V8 build requires Python 3.6+");
    }

    // Check for git (required to clone V8 sources)
    if which::which("git").is_err() {
        println!("cargo:warning=WARNING: Git not found - required for V8 source download");
    }

    // Platform-specific build tool checks
    #[cfg(target_os = "linux")]
    check_linux_requirements();
    
    #[cfg(target_os = "macos")]
    check_macos_requirements();
    
    #[cfg(target_os = "windows")]
    check_windows_requirements();
}

#[cfg(target_os = "linux")]
fn check_linux_requirements() {
    if which::which("clang").is_err() && which::which("gcc").is_err() {
        println!("cargo:warning=WARNING: No C++ compiler found - install clang or gcc");
    }
    if which::which("ninja").is_err() {
        println!("cargo:warning=WARNING: ninja not found - install ninja-build package");
    }
}

#[cfg(target_os = "macos")]  
fn check_macos_requirements() {
    if which::which("clang").is_err() {
        println!("cargo:warning=WARNING: clang not found - install Xcode command line tools");
    }
    if which::which("ninja").is_err() {
        println!("cargo:warning=WARNING: ninja not found - install with 'brew install ninja'");
    }
}

#[cfg(target_os = "windows")]
fn check_windows_requirements() {
    println!("cargo:warning=NOTE: Windows V8 build requires Visual Studio 2019+ with C++ tools");
}