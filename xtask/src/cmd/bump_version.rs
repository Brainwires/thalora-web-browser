//! Version bumping for the workspace
//!
//! Updates version strings across the workspace: root Cargo.toml,
//! member Cargo.toml files, Rust source, and CHANGELOG.md.

use std::process::ExitCode;
use walkdir::WalkDir;

const SKIP_DIRS: &[&str] = &["target", ".git", "node_modules", "pkg", "test-results"];

pub fn run(args: &[String]) -> ExitCode {
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return if args.is_empty() {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        };
    }

    let new_version = &args[0];

    // Validate version format
    let parts: Vec<&str> = new_version.split('.').collect();
    if parts.len() != 3 || parts.iter().any(|p| p.parse::<u32>().is_err()) {
        eprintln!("Invalid version format: '{}'. Expected X.Y.Z", new_version);
        return ExitCode::FAILURE;
    }

    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");
    let root = crate::workspace_root();

    // Read current version
    let root_toml_path = root.join("Cargo.toml");
    let root_content = match std::fs::read_to_string(&root_toml_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read root Cargo.toml: {}", e);
            return ExitCode::FAILURE;
        }
    };

    let old_version = match read_workspace_version(&root_content) {
        Some(v) => v,
        None => {
            eprintln!("Could not find [package].version in root Cargo.toml");
            return ExitCode::FAILURE;
        }
    };

    if old_version == *new_version {
        println!("Version is already {}", new_version);
        return ExitCode::SUCCESS;
    }

    println!(
        "Bumping version: {} -> {}{}",
        old_version,
        new_version,
        if dry_run { " (dry run)" } else { "" }
    );

    let mut updated_count = 0u32;

    // Step 1: Update root Cargo.toml
    println!("\n[1/4] Updating root Cargo.toml");
    if let Some(new_content) = update_cargo_toml_version(&root_content, &old_version, new_version) {
        if !dry_run {
            if let Err(e) = std::fs::write(&root_toml_path, &new_content) {
                eprintln!("  Failed to write root Cargo.toml: {}", e);
                return ExitCode::FAILURE;
            }
        }
        println!("  updated root Cargo.toml");
        updated_count += 1;
    }

    // Step 2: Update member Cargo.toml files
    println!("\n[2/4] Updating member Cargo.toml files");
    for entry in WalkDir::new(&root)
        .max_depth(4)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_str().unwrap_or("");
            !SKIP_DIRS.contains(&name)
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path == root_toml_path {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) != Some("Cargo.toml") {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if let Some(new_content) = update_cargo_toml_version(&content, &old_version, new_version) {
            let rel = path.strip_prefix(&root).unwrap_or(path);
            if !dry_run {
                if let Err(e) = std::fs::write(path, &new_content) {
                    eprintln!("  Failed to write {}: {}", rel.display(), e);
                    continue;
                }
            }
            println!("  updated {}", rel.display());
            updated_count += 1;
        }
    }

    // Step 3: Update version strings in Rust source files
    println!("\n[3/4] Updating version strings in source files");
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

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Look for "version": "X.Y.Z" patterns (JSON-style in Rust strings)
        let old_pattern = format!("\"version\": \"{}\"", old_version);
        let new_pattern = format!("\"version\": \"{}\"", new_version);

        if content.contains(&old_pattern) {
            let new_content = content.replace(&old_pattern, &new_pattern);
            let rel = path.strip_prefix(&root).unwrap_or(path);
            if !dry_run {
                if let Err(e) = std::fs::write(path, &new_content) {
                    eprintln!("  Failed to write {}: {}", rel.display(), e);
                    continue;
                }
            }
            println!("  updated {}", rel.display());
            updated_count += 1;
        }
    }

    // Step 4: Update CHANGELOG.md
    println!("\n[4/4] Updating CHANGELOG.md");
    let changelog_path = root.join("CHANGELOG.md");
    if changelog_path.exists() {
        let content = match std::fs::read_to_string(&changelog_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("  Failed to read CHANGELOG.md: {}", e);
                return ExitCode::FAILURE;
            }
        };

        if let Some(new_content) = update_changelog(&content, new_version) {
            if !dry_run {
                if let Err(e) = std::fs::write(&changelog_path, &new_content) {
                    eprintln!("  Failed to write CHANGELOG.md: {}", e);
                    return ExitCode::FAILURE;
                }
            }
            println!("  stamped CHANGELOG.md with [{}]", new_version);
            updated_count += 1;
        } else {
            println!("  no [Unreleased] section found in CHANGELOG.md");
        }
    }

    println!(
        "\nDone! {} file(s) updated.{}",
        updated_count,
        if dry_run { " (dry run - no files written)" } else { "" }
    );
    ExitCode::SUCCESS
}

fn read_workspace_version(content: &str) -> Option<String> {
    let doc = content.parse::<toml_edit::DocumentMut>().ok()?;
    doc.get("package")?
        .get("version")?
        .as_str()
        .map(|s| s.to_string())
}

fn update_cargo_toml_version(content: &str, old: &str, new: &str) -> Option<String> {
    if !content.contains(old) {
        return None;
    }
    Some(content.replace(old, new))
}

/// Convert `## [Unreleased]` into `## [X.Y.Z] - YYYY-MM-DD` and add fresh Unreleased section
fn update_changelog(content: &str, version: &str) -> Option<String> {
    let today = chrono_lite_today();
    let old_header = "## [Unreleased]";
    if !content.contains(old_header) {
        return None;
    }

    let stamped = format!("## [{}] - {}", version, today);
    let new_unreleased = format!("## [Unreleased]\n\n{}", stamped);
    Some(content.replace(old_header, &new_unreleased))
}

/// Minimal date formatter (avoids chrono dependency)
fn chrono_lite_today() -> String {
    // Use system date command as lightweight alternative to chrono crate
    let output = std::process::Command::new("date")
        .args(["+%Y-%m-%d"])
        .output();
    match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        }
        _ => "YYYY-MM-DD".to_string(),
    }
}

pub fn print_help() {
    println!("Bump workspace version");
    println!();
    println!("USAGE:");
    println!("  cargo xtask bump-version <VERSION> [OPTIONS]");
    println!();
    println!("ARGUMENTS:");
    println!("  <VERSION>      New version in X.Y.Z format");
    println!();
    println!("OPTIONS:");
    println!("  -n, --dry-run  Show what would be changed without writing");
    println!("  -h, --help     Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("  cargo xtask bump-version 0.3.0");
    println!("  cargo xtask bump-version 0.3.0 --dry-run");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_workspace_version() {
        let toml = r#"
[package]
name = "test"
version = "0.2.0"
"#;
        assert_eq!(read_workspace_version(toml), Some("0.2.0".to_string()));
    }

    #[test]
    fn test_read_workspace_version_missing() {
        let toml = r#"
[package]
name = "test"
"#;
        assert_eq!(read_workspace_version(toml), None);
    }

    #[test]
    fn test_update_cargo_toml_version() {
        let content = r#"version = "0.2.0""#;
        let result = update_cargo_toml_version(content, "0.2.0", "0.3.0");
        assert_eq!(result, Some(r#"version = "0.3.0""#.to_string()));
    }

    #[test]
    fn test_update_changelog() {
        let content = "# Changelog\n\n## [Unreleased]\n\n### Added\n- Feature\n";
        let result = update_changelog(content, "0.3.0").unwrap();
        assert!(result.contains("## [0.3.0] - "));
        assert!(result.contains("## [Unreleased]"));
    }

    #[test]
    fn test_no_unreleased_section() {
        let content = "# Changelog\n\n## [0.2.0] - 2026-01-01\n";
        assert_eq!(update_changelog(content, "0.3.0"), None);
    }
}
