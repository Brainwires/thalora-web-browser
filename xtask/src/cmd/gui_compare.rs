//! `cargo xtask gui-compare` — capture a Thalora screenshot and open it side-by-side
//! with a reference image (e.g., a Chrome screenshot) using the system image viewer.
//!
//! Usage:
//!   cargo xtask gui-compare [URL] [--ref PATH] [--port PORT] [--delay MS] [--out PATH]
//!
//! If --ref is omitted the reference slot is left empty and only the Thalora screenshot
//! is opened. Use `--ref chrome.png` to compare against a pre-captured Chrome screenshot.

use anyhow::{Context, Result};

pub fn run(args: &[String]) -> Result<()> {
    let reference = parse_flag(args, "--ref");
    let out = parse_flag(args, "--out")
        .unwrap_or_else(|| "/tmp/thalora-compare.png".to_string());

    // Reuse gui_screenshot to capture
    let mut screenshot_args = args.to_vec();
    // Ensure --out is set to our compare path
    if parse_flag(args, "--out").is_none() {
        screenshot_args.push("--out".to_string());
        screenshot_args.push(out.clone());
    }
    super::gui_screenshot::run(&screenshot_args)?;

    // Open images
    let mut to_open: Vec<String> = Vec::new();
    if let Some(r) = &reference {
        if std::path::Path::new(r).exists() {
            to_open.push(r.clone());
        } else {
            eprintln!("Warning: reference image not found: {r}");
        }
    }
    to_open.push(out.clone());

    if to_open.is_empty() {
        println!("No images to open.");
        return Ok(());
    }

    println!("── Opening images for comparison...");
    let viewer = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    };

    for path in &to_open {
        std::process::Command::new(viewer)
            .arg(path)
            .spawn()
            .with_context(|| format!("opening {path}"))?;
    }

    Ok(())
}

fn parse_flag(args: &[String], flag: &str) -> Option<String> {
    let prefix = format!("{flag}=");
    for (i, a) in args.iter().enumerate() {
        if a == flag {
            return args.get(i + 1).cloned();
        }
        if a.starts_with(&prefix) {
            return Some(a[prefix.len()..].to_string());
        }
    }
    None
}
