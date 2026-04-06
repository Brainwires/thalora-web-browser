//! CI pipeline runner
//!
//! Runs fmt, check, clippy, test, and doc steps sequentially,
//! reporting pass/fail for each and a summary at the end.
//! Each step runs for both the main workspace and the boa engine workspace.

use std::process::{Command, ExitCode};

struct Step {
    key: &'static str,
    name: &'static str,
    /// Commands to run for this step. Each entry is (description, args).
    commands: &'static [(&'static str, &'static [&'static str])],
}

const STEPS: &[Step] = &[
    Step {
        key: "fmt",
        name: "Format",
        commands: &[
            ("workspace", &["cargo", "fmt", "--all", "--check"]),
            (
                "boa engine",
                &[
                    "cargo",
                    "fmt",
                    "--all",
                    "--check",
                    "--manifest-path",
                    "engines/boa/Cargo.toml",
                ],
            ),
        ],
    },
    Step {
        key: "check",
        name: "Check",
        commands: &[
            ("workspace", &["cargo", "check", "--workspace"]),
            (
                "boa engine",
                &[
                    "cargo",
                    "check",
                    "--manifest-path",
                    "engines/boa/Cargo.toml",
                    "--workspace",
                ],
            ),
        ],
    },
    Step {
        key: "clippy",
        name: "Clippy",
        commands: &[
            ("workspace", &["cargo", "clippy", "--workspace"]),
            (
                "boa engine",
                &[
                    "cargo",
                    "clippy",
                    "--manifest-path",
                    "engines/boa/Cargo.toml",
                    "--workspace",
                ],
            ),
        ],
    },
    Step {
        key: "test",
        name: "Test",
        commands: &[
            (
                "workspace",
                &[
                    "cargo",
                    "test",
                    "--workspace",
                    "--",
                    "--test-threads=1",
                ],
            ),
            (
                "boa engine",
                &[
                    "cargo",
                    "test",
                    "--manifest-path",
                    "engines/boa/Cargo.toml",
                    "--workspace",
                    "--",
                    "--test-threads=1",
                ],
            ),
        ],
    },
    Step {
        key: "doc",
        name: "Doc",
        commands: &[
            (
                "workspace",
                &["cargo", "doc", "--workspace", "--no-deps"],
            ),
            (
                "boa engine",
                &[
                    "cargo",
                    "doc",
                    "--manifest-path",
                    "engines/boa/Cargo.toml",
                    "--workspace",
                    "--no-deps",
                ],
            ),
        ],
    },
];

pub fn run(args: &[String]) -> ExitCode {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return ExitCode::SUCCESS;
    }

    // Filter steps if specific ones are requested
    let filter: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let steps: Vec<&Step> = if filter.is_empty() {
        STEPS.iter().collect()
    } else {
        STEPS
            .iter()
            .filter(|s| filter.iter().any(|f| f.eq_ignore_ascii_case(s.key)))
            .collect()
    };

    if steps.is_empty() {
        eprintln!("No matching CI steps. Available: fmt, check, clippy, test, doc");
        return ExitCode::FAILURE;
    }

    let total = steps.len();
    let mut passed = Vec::new();
    let mut failed = Vec::new();

    let ws_root = crate::workspace_root();

    for (i, step) in steps.iter().enumerate() {
        println!("\n[{}/{}] {} ...", i + 1, total, step.name);

        let mut step_ok = true;
        for (desc, cmd) in step.commands {
            println!("  -> {desc}");

            let status = Command::new(cmd[0])
                .args(&cmd[1..])
                .current_dir(&ws_root)
                .env("CARGO_TERM_COLOR", "always")
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!("  -> {desc} ... ok");
                }
                _ => {
                    println!("  -> {desc} ... FAILED");
                    step_ok = false;
                }
            }
        }

        if step_ok {
            println!("[{}/{}] {} ... ok", i + 1, total, step.name);
            passed.push(step.name);
        } else {
            println!("[{}/{}] {} ... FAILED", i + 1, total, step.name);
            failed.push(step.name);
        }
    }

    // Summary
    println!("\n--- CI Summary ---");
    if failed.is_empty() {
        println!("All {} steps passed.", total);
        ExitCode::SUCCESS
    } else {
        println!(
            "{}/{} steps failed: {}",
            failed.len(),
            total,
            failed.join(", ")
        );
        ExitCode::FAILURE
    }
}

pub fn print_help() {
    println!("Run CI pipeline steps (default: all)");
    println!();
    println!("Each step runs for both the main workspace and the boa engine workspace.");
    println!();
    println!("USAGE:");
    println!("  cargo xtask ci [STEPS...]");
    println!();
    println!("STEPS:");
    for step in STEPS {
        println!("  {:10} {}", step.key, step.name);
    }
    println!();
    println!("EXAMPLES:");
    println!("  cargo xtask ci              # Run all steps");
    println!("  cargo xtask ci check clippy # Run only check and clippy");
}
