mod cmd;

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let task = args.first().map(String::as_str).unwrap_or("help");

    match task {
        // CI tasks
        "ci" => cmd::ci::run(&args[1..]),
        "check-stubs" => cmd::check_stubs::run(&args[1..]),
        "bump-version" => cmd::bump_version::run(&args[1..]),

        // GUI visual testing tasks
        "gui-screenshot" => exit_from_result(cmd::gui_screenshot::run(&args[1..])),
        "gui-compare" => exit_from_result(cmd::gui_compare::run(&args[1..])),

        // Existing build/test tasks
        "build-wasm" => exit_from_result(cmd::build_wasm::run()),
        "test-mcp" => exit_from_result(cmd::test_mcp::run(
            args.get(1).map(String::as_str).unwrap_or(""),
        )),
        "test-mcp-clean" => exit_from_result(cmd::test_mcp_clean::run()),
        "demo-form" => exit_from_result(cmd::demo_form::run()),
        "test-events" => exit_from_result(cmd::test_events::run()),
        "test-windows" => exit_from_result(cmd::test_windows::run()),

        "help" | "--help" | "-h" => {
            print_help();
            ExitCode::SUCCESS
        }
        unknown => {
            eprintln!("Unknown task: {unknown}");
            eprintln!("Run `cargo xtask help` for a list of available tasks.");
            ExitCode::FAILURE
        }
    }
}

fn print_help() {
    println!("Thalora xtask runner");
    println!();
    println!("USAGE:");
    println!("  cargo xtask <TASK> [OPTIONS]");
    println!();
    println!("CI TASKS:");
    println!("  ci [STEPS...]                Run CI pipeline: fmt, check, clippy, test, doc");
    println!("  check-stubs [--strict]       Scan for unfinished code (todo!(), FIXME, etc.)");
    println!("  bump-version <VER> [--dry-run]  Bump version across workspace");
    println!();
    println!("GUI VISUAL TESTING:");
    println!("  gui-screenshot [URL]         Build + launch browser, capture PNG screenshot");
    println!("    --out PATH                 Output file (default: /tmp/thalora-screenshot.png)");
    println!("    --port PORT                Control server port (default: 9222)");
    println!("    --delay MS                 Wait before screenshot in ms (default: 2000)");
    println!("    --no-build                 Skip dotnet build step");
    println!("    --no-kill                  Leave browser running after screenshot");
    println!("  gui-compare [URL]            Screenshot + open side-by-side with reference");
    println!("    --ref PATH                 Reference image to compare against (e.g. chrome.png)");
    println!(
        "    --out PATH                 Thalora output path (default: /tmp/thalora-compare.png)"
    );
    println!();
    println!("BUILD & TEST TASKS:");
    println!("  build-wasm       Build WASM targets (bundler, nodejs, web) via wasm-pack");
    println!("  test-mcp         Run MCP test suite [--quick | --perf | --verbose]");
    println!("  test-mcp-clean   Run quick smoke tests with clean JSON output");
    println!("  demo-form        Run full form automation demo");
    println!("  test-events      Run DOM Event API isolation tests");
    println!("  test-windows     Run multi-window workflow tests");
    println!();
    println!("  help             Show this help message");
}

/// Convert anyhow::Result to ExitCode for legacy commands
fn exit_from_result(result: anyhow::Result<()>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

/// Resolve the workspace root from the xtask crate location
pub fn workspace_root() -> PathBuf {
    let xtask_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    xtask_dir
        .parent()
        .expect("xtask should be inside workspace root")
        .to_path_buf()
}
