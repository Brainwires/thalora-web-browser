//! Debug utilities for file-based logging during development.
//!
//! This module provides simple utilities for logging debug information to files,
//! useful during development when tracing execution flow through DOM operations.

use std::fs::OpenOptions;
use std::io::Write;

/// Log a message to a file. Creates the file if it doesn't exist, appends if it does.
///
/// # Arguments
/// * `path` - The file path to write to (e.g., "/tmp/debug.log")
/// * `message_fn` - A closure that returns the message to log
///
/// # Example
/// ```ignore
/// use thalora_browser_apis::debug_utils::log_to_file;
///
/// log_to_file("/tmp/debug.log", || "Simple message");
/// log_to_file("/tmp/debug.log", || format!("Value: {}", some_value));
/// ```
pub fn log_to_file<F, S>(path: &str, message_fn: F)
where
    F: FnOnce() -> S,
    S: AsRef<str>,
{
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(f, "{}", message_fn().as_ref());
    }
}

/// Log a message to a file only if a condition is true.
///
/// # Arguments
/// * `condition` - Only log if this is true
/// * `path` - The file path to write to
/// * `message_fn` - A closure that returns the message to log
pub fn log_to_file_if<F, S>(condition: bool, path: &str, message_fn: F)
where
    F: FnOnce() -> S,
    S: AsRef<str>,
{
    if condition {
        log_to_file(path, message_fn);
    }
}

/// Clear a log file (truncate to empty).
///
/// # Arguments
/// * `path` - The file path to clear
pub fn clear_log_file(path: &str) {
    if let Ok(f) = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
    {
        drop(f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_log_to_file() {
        let test_path = "/tmp/test_debug_utils.log";

        // Clear any existing file
        clear_log_file(test_path);

        // Log some messages
        log_to_file(test_path, || "First message");
        log_to_file(test_path, || format!("Value: {}", 42));

        // Read and verify
        let content = fs::read_to_string(test_path).unwrap();
        assert!(content.contains("First message"));
        assert!(content.contains("Value: 42"));

        // Cleanup
        let _ = fs::remove_file(test_path);
    }
}
