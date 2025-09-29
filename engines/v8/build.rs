use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=v8-source");
    println!("cargo:rerun-if-env-changed=V8_FROM_SOURCE");
    println!("cargo:rerun-if-env-changed=THALORA_V8_BRANCH");

    // Force building V8 from source using our local submodule
    env::set_var("V8_FROM_SOURCE", "1");
    
    // Use the local submodule path instead of remote cloning
    let v8_source_path = Path::new("engines/v8/v8-source").canonicalize()
        .expect("V8 submodule not found. Run: git submodule update --init --recursive");
    
    // Set V8 source directory to our local submodule
    env::set_var("RUSTY_V8_REPO", v8_source_path.to_string_lossy().to_string());
    
    // Handle custom branch if specified
    let custom_branch = env::var("THALORA_V8_BRANCH")
        .unwrap_or_else(|_| "main".to_string());
    
    if custom_branch != "main" {
        println!("cargo:warning=Using V8 branch: {}", custom_branch);
        // Switch to the specified branch in the submodule
        let output = Command::new("git")
            .args(&["checkout", &custom_branch])
            .current_dir(&v8_source_path)
            .output();
            
        match output {
            Ok(result) if result.status.success() => {
                println!("cargo:warning=Successfully switched to branch: {}", custom_branch);
            }
            Ok(result) => {
                println!("cargo:warning=Failed to switch to branch {}: {}", 
                    custom_branch, String::from_utf8_lossy(&result.stderr));
            }
            Err(e) => {
                println!("cargo:warning=Git command failed: {}", e);
            }
        }
    }
    
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
    println!("cargo:warning=Thalora V8 Build Configuration:");
    println!("cargo:warning=  Source: Local submodule ({})", v8_source_path.display());
    println!("cargo:warning=  Branch: {}", custom_branch);
    println!("cargo:warning=  Building from source: enabled");
    println!("cargo:warning=  Pre-built binaries: disabled");
    println!("cargo:warning=");
    println!("cargo:warning=V8 build may take 15-30 minutes on first compile");
    println!("cargo:warning=Subsequent builds will be much faster (incremental)");
    
    // Check build requirements
    check_build_requirements();
}

fn check_build_requirements() {
    // Check for Python (required by V8 build system)
    if which::which("python3").is_err() && which::which("python").is_err() {
        println!("cargo:warning=WARNING: Python not found - V8 build requires Python 3.6+");
    }

    // Check for git (required for submodule management)
    if which::which("git").is_err() {
        println!("cargo:warning=WARNING: Git not found - required for submodule management");
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
    if which::which("pkg-config").is_err() {
        println!("cargo:warning=WARNING: pkg-config not found - may be needed for dependencies");
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
    println!("cargo:warning=NOTE: Ensure Windows SDK is installed");
}