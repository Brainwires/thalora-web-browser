//! `cargo xtask gui-screenshot` — launch the GUI browser, capture a screenshot, save it.
//!
//! Usage:
//!   cargo xtask gui-screenshot [URL] [--out PATH] [--port PORT] [--delay MS] [--no-kill]
//!
//! Steps:
//!   1. Build the GUI (dotnet build)
//!   2. Launch: dotnet run --project gui/ThaloraBrowser -- --url URL --control-port PORT
//!   3. Poll /health until ready (up to 30s)
//!   4. Hit /screenshot?delay=DELAY_MS
//!   5. Save PNG to --out (default: /tmp/thalora-screenshot.png)
//!   6. Kill the browser process (unless --no-kill)

use anyhow::{Context, Result};
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub fn run(args: &[String]) -> Result<()> {
    let url = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .map(String::as_str)
        .unwrap_or("https://www.google.com");

    let port = parse_flag(args, "--port").unwrap_or_else(|| "9222".to_string());
    let delay = parse_flag(args, "--delay").unwrap_or_else(|| "2000".to_string());
    let out =
        parse_flag(args, "--out").unwrap_or_else(|| "/tmp/thalora-screenshot.png".to_string());
    let no_kill = args.iter().any(|a| a == "--no-kill");
    let no_build = args.iter().any(|a| a == "--no-build");

    // Kill any existing browser on this port first
    kill_existing(&port);

    if !no_build {
        println!("── Building GUI...");
        let status = Command::new("dotnet")
            .args(["build", "gui/ThaloraBrowser", "--nologo", "-q"])
            .status()
            .context("dotnet build failed")?;
        if !status.success() {
            anyhow::bail!("GUI build failed");
        }
        println!("   Build OK");
    }

    println!("── Launching browser → {url}  (port {port})");
    let mut child = Command::new("dotnet")
        .args([
            "run",
            "--project",
            "gui/ThaloraBrowser",
            "--no-build",
            "--",
            "--url",
            url,
            "--control-port",
            &port,
        ])
        .spawn()
        .context("failed to launch ThaloraBrowser")?;

    // Wait for health endpoint via curl
    let health_url = format!("http://localhost:{port}/health");
    let deadline = Instant::now() + Duration::from_secs(35);
    let mut ready = false;
    while Instant::now() < deadline {
        sleep(Duration::from_millis(800));
        let ok = Command::new("curl")
            .args(["-sf", "--max-time", "1", &health_url])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if ok {
            ready = true;
            break;
        }
    }
    if !ready {
        let _ = child.kill();
        anyhow::bail!("Browser did not become ready within 35s");
    }
    println!("   Browser ready");

    // Capture screenshot via curl
    let screenshot_url = format!("http://localhost:{port}/screenshot?delay={delay}");
    println!("── Capturing screenshot (delay={delay}ms)...");
    let status = Command::new("curl")
        .args(["-sf", &screenshot_url, "-o", &out])
        .status()
        .context("curl screenshot failed")?;
    if !status.success() {
        anyhow::bail!("Screenshot request failed (curl exit {status})");
    }
    let size = std::fs::metadata(&out).map(|m| m.len()).unwrap_or(0);
    println!("   Saved → {out}  ({size} bytes)");

    if !no_kill {
        kill_browser(&mut child);
    } else {
        println!("   Browser left running on port {port} (--no-kill)");
        // Detach so xtask exits without waiting
        drop(child);
    }

    println!("── Done.");
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

fn kill_existing(port: &str) {
    // Best-effort: find and kill any process listening on the control port
    let _ = Command::new("sh")
        .args([
            "-c",
            &format!("lsof -ti tcp:{port} | xargs kill -9 2>/dev/null; true"),
        ])
        .status();
    sleep(Duration::from_millis(500));
}

fn kill_browser(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}
