pub mod build_wasm;
pub mod demo_form;
pub mod test_events;
pub mod test_mcp;
pub mod test_mcp_clean;
pub mod test_windows;

use std::process::{Command, Stdio};

/// Run a command, inheriting stdout/stderr. Returns an error if the process exits non-zero.
pub fn run_cmd(cmd: &mut Command) -> anyhow::Result<()> {
    let status = cmd.status()?;
    if !status.success() {
        let code = status.code().unwrap_or(-1);
        anyhow::bail!("command failed with exit code {code}");
    }
    Ok(())
}

/// Pipe `input` to the thalora binary's stdin and inherit stdout/stderr.
/// Ensures the release binary exists (builds it if missing).
pub fn run_mcp_binary(input: &str, env: &[(&str, &str)]) -> anyhow::Result<()> {
    ensure_release_binary()?;

    let mut cmd = Command::new("./target/release/thalora");
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.stdin(Stdio::piped());

    let mut child = cmd.spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(input.as_bytes())?;
    }
    let status = child.wait()?;
    if !status.success() {
        let code = status.code().unwrap_or(-1);
        anyhow::bail!("thalora exited with code {code}");
    }
    Ok(())
}

/// Build the release binary if it doesn't exist yet.
fn ensure_release_binary() -> anyhow::Result<()> {
    if std::path::Path::new("./target/release/thalora").exists() {
        return Ok(());
    }
    eprintln!("Release binary not found — building (this may take a while)...");
    run_cmd(Command::new("cargo").args(["build", "--release", "--quiet"]))
}
