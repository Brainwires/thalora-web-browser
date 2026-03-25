/// Debug utilities for Thalora
///
/// Provides a consistent debug output mechanism that respects the THALORA_SILENT environment variable.
/// This is crucial for MCP (Model Context Protocol) integration since debug output can corrupt JSON responses.
use std::sync::OnceLock;

static DEBUG_ENABLED: OnceLock<bool> = OnceLock::new();

/// Check if debug output is enabled
pub fn is_debug_enabled() -> bool {
    *DEBUG_ENABLED.get_or_init(|| {
        // Disable debug if THALORA_SILENT is set (respects MCP silent mode)
        // OR enable if THALORA_DEBUG=true (explicit debug mode)
        std::env::var("THALORA_SILENT").is_err()
            && std::env::var("THALORA_DEBUG").unwrap_or_else(|_| "false".to_string()) == "true"
    })
}

/// Debug print macro that respects THALORA_SILENT environment variable
/// Only prints if THALORA_SILENT is not set AND THALORA_DEBUG=true
#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        if $crate::debug_utils::is_debug_enabled() {
            eprintln!($($arg)*);
        }
    };
}

/// Alternative debug function for when macro import issues occur
pub fn debug_println(message: &str) {
    if is_debug_enabled() {
        eprintln!("{}", message);
    }
}

/// Debug print with formatted arguments
pub fn debug_printf(message: &str, args: &[&dyn std::fmt::Display]) {
    if is_debug_enabled() {
        let mut formatted = message.to_string();
        for (i, arg) in args.iter().enumerate() {
            formatted = formatted.replace(&format!("{{{}}}", i), &arg.to_string());
        }
        eprintln!("{}", formatted);
    }
}
