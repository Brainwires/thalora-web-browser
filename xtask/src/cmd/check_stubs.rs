//! Scan for unfinished code markers
//!
//! Detects todo!(), unimplemented!(), FIXME, HACK, etc. in Rust source files.
//! Hard blockers (panic macros) always fail; soft markers (comments) only fail
//! with --strict.

use std::process::ExitCode;
use walkdir::WalkDir;

const SKIP_DIRS: &[&str] = &["target", ".git", "node_modules", "test-results", "pkg"];

/// Patterns that panic at runtime - always errors
const HARD_BLOCKERS: &[(&str, &str)] = &[
    ("todo!(", "todo!() macro - panics at runtime"),
    (
        "unimplemented!(",
        "unimplemented!() macro - panics at runtime",
    ),
];

/// Comment markers - warnings (errors with --strict)
const SOFT_MARKERS: &[(&str, &str)] = &[
    ("FIXME", "FIXME marker"),
    ("HACK", "HACK marker"),
    ("XXX", "XXX marker"),
    ("STOPSHIP", "STOPSHIP marker"),
];

/// Files to skip (self-references, intentional marker collections)
const SKIP_FILES: &[&str] = &["xtask/src/cmd/check_stubs.rs"];

struct Finding {
    path: String,
    line_num: usize,
    line: String,
    reason: String,
    is_hard: bool,
}

pub fn run(args: &[String]) -> ExitCode {
    let strict = args.iter().any(|a| a == "--strict" || a == "-s");
    let verbose = args.iter().any(|a| a == "--verbose" || a == "-v");

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return ExitCode::SUCCESS;
    }

    let root = crate::workspace_root();
    println!("Scanning for unfinished code in: {}", root.display());

    let mut findings: Vec<Finding> = Vec::new();
    let mut file_count = 0u32;

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_str().unwrap_or("");
            !SKIP_DIRS.contains(&name)
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("rs") {
            continue;
        }

        // Check skip list
        let rel = path.strip_prefix(&root).unwrap_or(path);
        let rel_str = rel.to_string_lossy();
        if SKIP_FILES.iter().any(|s| rel_str.contains(s)) {
            continue;
        }

        file_count += 1;
        if verbose {
            println!("  scanning: {}", rel_str);
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let mut in_test_block = false;
        let mut brace_depth = 0i32;

        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Track test module scope to skip
            if trimmed.contains("#[cfg(test)]") || trimmed.contains("mod tests") {
                in_test_block = true;
                brace_depth = 0;
            }
            if in_test_block {
                brace_depth += trimmed.matches('{').count() as i32;
                brace_depth -= trimmed.matches('}').count() as i32;
                if brace_depth < 0 {
                    in_test_block = false;
                    brace_depth = 0;
                }
                continue;
            }

            // Skip lines that are about the checker itself
            if trimmed.contains("check-stubs") || trimmed.contains("check_stubs") {
                continue;
            }

            // Check hard blockers
            for (pattern, reason) in HARD_BLOCKERS {
                if trimmed.contains(pattern) {
                    findings.push(Finding {
                        path: rel_str.to_string(),
                        line_num: line_idx + 1,
                        line: trimmed.to_string(),
                        reason: reason.to_string(),
                        is_hard: true,
                    });
                    break;
                }
            }

            // Check soft markers (only in comments/strings)
            for (pattern, reason) in SOFT_MARKERS {
                if is_marker_in_comment(trimmed, pattern) {
                    findings.push(Finding {
                        path: rel_str.to_string(),
                        line_num: line_idx + 1,
                        line: trimmed.to_string(),
                        reason: reason.to_string(),
                        is_hard: false,
                    });
                    break;
                }
            }
        }
    }

    println!("Scanned {} .rs files", file_count);

    let hard: Vec<&Finding> = findings.iter().filter(|f| f.is_hard).collect();
    let soft: Vec<&Finding> = findings.iter().filter(|f| !f.is_hard).collect();

    if !hard.is_empty() {
        println!("\nERRORS - hard blockers ({} found):", hard.len());
        for f in &hard {
            println!(
                "  {}:{}: {} [{}]",
                f.path,
                f.line_num,
                truncate(&f.line, 80),
                f.reason
            );
        }
    }

    if !soft.is_empty() {
        println!("\nWARNINGS - comment markers ({} found):", soft.len());
        for f in &soft {
            println!(
                "  {}:{}: {} [{}]",
                f.path,
                f.line_num,
                truncate(&f.line, 80),
                f.reason
            );
        }
    }

    if hard.is_empty() && soft.is_empty() {
        println!("Clean! No unfinished code markers found.");
        return ExitCode::SUCCESS;
    }

    let has_errors = !hard.is_empty() || (strict && !soft.is_empty());

    if has_errors {
        println!(
            "\nFAILED: {} error(s), {} warning(s)",
            if strict {
                hard.len() + soft.len()
            } else {
                hard.len()
            },
            if strict { 0 } else { soft.len() }
        );
        ExitCode::FAILURE
    } else {
        if !soft.is_empty() {
            println!(
                "\nPassed with {} warning(s). Use --strict to treat warnings as errors.",
                soft.len()
            );
        }
        ExitCode::SUCCESS
    }
}

/// Check if a marker appears in a comment context (// or /* */)
fn is_marker_in_comment(line: &str, marker: &str) -> bool {
    // Only match if the marker appears after // or inside a comment-like context
    if let Some(pos) = line.find(marker) {
        let before = &line[..pos];
        // Check for line comment
        if before.contains("//") {
            return true;
        }
        // Check for block comment start
        if before.contains("/*") {
            return true;
        }
    }
    false
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

pub fn print_help() {
    println!("Scan for unfinished code markers");
    println!();
    println!("USAGE:");
    println!("  cargo xtask check-stubs [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  -s, --strict   Treat FIXME/HACK/XXX as errors (not just warnings)");
    println!("  -v, --verbose  Show all scanned files");
    println!("  -h, --help     Show this help message");
    println!();
    println!("Hard blockers (always fail):");
    println!("  todo!(), unimplemented!()");
    println!();
    println!("Soft markers (warnings, errors with --strict):");
    println!("  FIXME, HACK, XXX, STOPSHIP");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marker_in_comment() {
        assert!(is_marker_in_comment("// FIXME: broken", "FIXME"));
        assert!(is_marker_in_comment("/* HACK */", "HACK"));
        assert!(!is_marker_in_comment("let fixme_flag = true;", "FIXME"));
    }

    #[test]
    fn test_marker_not_in_identifier() {
        // "FIXME" after // is a match, standalone in code is not
        assert!(!is_marker_in_comment("fn handle_FIXME_case() {}", "FIXME"));
        assert!(is_marker_in_comment("// XXX: temporary", "XXX"));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a longer string", 15), "this is a lo...");
    }
}
