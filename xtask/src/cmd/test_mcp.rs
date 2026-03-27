use anyhow::Result;
use std::process::Command;

pub fn run(flag: &str) -> Result<()> {
    match flag {
        "--quick" | "-q" => run_quick(),
        "--perf" | "-p" => run_perf(),
        "--verbose" | "-v" => {
            unsafe { std::env::set_var("RUST_LOG", "debug") };
            run_all()
        }
        "" => run_all(),
        other => {
            eprintln!("Unknown flag: {other}");
            eprintln!("Usage: cargo xtask test-mcp [--quick | --perf | --verbose]");
            std::process::exit(1);
        }
    }
}

fn build_debug() -> Result<()> {
    println!("Building Thalora MCP server...");
    super::run_cmd(Command::new("cargo").args(["build", "--quiet"]))
}

fn build_release() -> Result<()> {
    println!("Building release version for performance tests...");
    super::run_cmd(Command::new("cargo").args(["build", "--release", "--quiet"]))
}

fn run_category(filter: &str, description: &str) -> Result<()> {
    println!("Running {description}...");
    super::run_cmd(
        Command::new("cargo")
            .args(["test", "--test", "mcp_tests", filter, "--", "--nocapture"]),
    )
}

fn run_all() -> Result<()> {
    println!("Thalora MCP Test Suite");
    println!("======================");

    build_debug()?;

    let mut failed = 0u32;

    let categories: &[(&str, &str)] = &[
        ("test_harness", "Test Harness Verification"),
        (
            "test_mcp_initialization|test_tools_list|test_expected_tools",
            "Protocol Compliance Tests",
        ),
        (
            "test_ai_memory|test_snapshot_url|test_cdp_runtime",
            "Core Tool Functionality Tests",
        ),
        (
            "test_research_workflow|test_browser_automation|test_data_persistence",
            "Integration Workflow Tests",
        ),
    ];

    for (filter, desc) in categories {
        if run_category(filter, desc).is_err() {
            failed += 1;
        }
    }

    // Performance tests need release build
    println!("Performance tests require release build...");
    if build_release().is_ok() {
        if run_category("test_.*_performance|test_stress_test", "Performance & Stress Tests").is_err() {
            failed += 1;
        }
    } else {
        eprintln!("Skipping performance tests due to release build failure");
        failed += 1;
    }

    let total = 5u32;
    let passed = total - failed;
    println!("\n=========================");
    println!("Test Suite Summary");
    println!("=========================");
    if failed == 0 {
        println!("All test categories passed! ({passed}/{total})");
        println!("The MCP server is ready for AI model integration!");
    } else {
        eprintln!("{failed}/{total} test categories failed");
        std::process::exit(1);
    }
    Ok(())
}

fn run_quick() -> Result<()> {
    println!("Running quick smoke tests...");
    build_debug()?;
    super::run_cmd(
        Command::new("cargo").args([
            "test",
            "--test",
            "mcp_tests",
            "test_harness_functionality|test_tool_categories_smoke",
            "--",
            "--nocapture",
        ]),
    )?;
    println!("Quick tests passed - MCP server basic functionality works");
    Ok(())
}

fn run_perf() -> Result<()> {
    println!("Running performance tests only...");
    build_release()?;
    super::run_cmd(
        Command::new("cargo").args([
            "test",
            "--release",
            "--test",
            "mcp_tests",
            "performance",
            "--",
            "--nocapture",
        ]),
    )?;
    println!("Performance tests completed");
    Ok(())
}
