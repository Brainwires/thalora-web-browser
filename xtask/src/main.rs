mod cmd;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let task = args.get(1).map(String::as_str).unwrap_or("help");

    match task {
        "build-wasm" => cmd::build_wasm::run(),
        "test-mcp" => cmd::test_mcp::run(args.get(2).map(String::as_str).unwrap_or("")),
        "test-mcp-clean" => cmd::test_mcp_clean::run(),
        "demo-form" => cmd::demo_form::run(),
        "test-events" => cmd::test_events::run(),
        "test-windows" => cmd::test_windows::run(),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        unknown => {
            eprintln!("Unknown task: {unknown}");
            eprintln!("Run `cargo xtask help` for a list of available tasks.");
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("Thalora xtask runner");
    println!();
    println!("USAGE:");
    println!("  cargo xtask <TASK> [OPTIONS]");
    println!();
    println!("TASKS:");
    println!("  build-wasm       Build WASM targets (bundler, nodejs, web) via wasm-pack");
    println!("  test-mcp         Run MCP test suite [--quick | --perf | --verbose]");
    println!("  test-mcp-clean   Run quick smoke tests with clean JSON output");
    println!("  demo-form        Run full form automation demo");
    println!("  test-events      Run DOM Event API isolation tests");
    println!("  test-windows     Run multi-window workflow tests");
    println!("  help             Show this help message");
}
