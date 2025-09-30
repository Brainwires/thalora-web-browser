use std::env;
use std::path::Path;

fn main() {
    eprintln!("cargo:rerun-if-changed=build.rs");
    eprintln!("cargo:rerun-if-changed=v8-source");
    eprintln!("cargo:rerun-if-env-changed=V8_FROM_SOURCE");

    // Force building V8 from source using our local v8-source directory
    env::set_var("V8_FROM_SOURCE", "1");
    
    // Verify the local V8 source directory exists
    let v8_source_path = Path::new("v8-source");
    if !v8_source_path.exists() {
        panic!("V8 source directory not found at v8-source/. Ensure the nightness/v8 fork is properly placed there.");
    }
    
    // Point rusty_v8 to use our local V8 source directory instead of downloading
    let current_dir = env::current_dir().expect("Could not get current directory");
    let full_v8_path = current_dir.join("v8-source");
    
    // Set environment variables to force rusty_v8 to use our local source
    env::set_var("RUSTY_V8_SRC", full_v8_path.to_string_lossy().to_string());
    
    // Configure V8 build arguments for better compatibility with Thalora
    let gn_args = [
        "v8_enable_sandbox=false",
        "v8_expose_symbols=true", 
        "v8_use_external_startup_data=false",
        "v8_enable_pointer_compression=false",
        "use_custom_libcxx=false",
        "treat_warnings_as_errors=false",
        "v8_enable_i18n_support=true",
        "v8_enable_webassembly=true"
    ].join(" ");
    env::set_var("GN_ARGS", gn_args);
    
    // Print build configuration
    eprintln!("cargo:warning=Thalora V8 Build Configuration:");
    eprintln!("cargo:warning=  Source: Local directory ({})", full_v8_path.display());
    eprintln!("cargo:warning=  Building from source: enabled");
    eprintln!("cargo:warning=  Pre-built binaries: disabled");
    eprintln!("cargo:warning=");
    eprintln!("cargo:warning=V8 build may take 15-30 minutes on first compile");
    eprintln!("cargo:warning=Subsequent builds will be much faster (incremental)");
    
    // Check build requirements
    check_build_requirements();
}

fn check_build_requirements() {
    // Check for Python (required by V8 build system)
    if which::which("python3").is_err() && which::which("python").is_err() {
        eprintln!("cargo:warning=WARNING: Python not found - V8 build requires Python 3.6+");
    }

    // Check for git (required for submodule management)
    if which::which("git").is_err() {
        eprintln!("cargo:warning=WARNING: Git not found - required for submodule management");
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
        eprintln!("cargo:warning=WARNING: No C++ compiler found - install clang or gcc");
    }
    if which::which("ninja").is_err() {
        eprintln!("cargo:warning=WARNING: ninja not found - install ninja-build package");
    }
    if which::which("pkg-config").is_err() {
        eprintln!("cargo:warning=WARNING: pkg-config not found - may be needed for dependencies");
    }
}

#[cfg(target_os = "macos")]  
fn check_macos_requirements() {
    if which::which("clang").is_err() {
        eprintln!("cargo:warning=WARNING: clang not found - install Xcode command line tools");
    }
    if which::which("ninja").is_err() {
        eprintln!("cargo:warning=WARNING: ninja not found - install with 'brew install ninja'");
    }
}

#[cfg(target_os = "windows")]
fn check_windows_requirements() {
    eprintln!("cargo:warning=NOTE: Windows V8 build requires Visual Studio 2019+ with C++ tools");
    eprintln!("cargo:warning=NOTE: Ensure Windows SDK is installed");
}