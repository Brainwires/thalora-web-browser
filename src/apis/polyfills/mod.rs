// JavaScript polyfills for browser APIs only
// NOTE: All ES6-ES2023 language features are now natively handled by Boa engine
// NOTE: Console is now handled by Boa's native console implementation
pub mod web_apis;
pub mod syntax_transformer;
pub mod console;

// Modular polyfill components
pub mod performance;
pub mod security;
// DOM and CSS are now natively implemented in Boa engine
pub mod worker;
pub mod storage;
pub mod chrome_features;

// Only experimental/proposal polyfills remain
pub mod es2024_polyfills;
pub mod es2025_experimental;

use anyhow::Result;
use boa_engine::Context;
use crate::apis::timers::TimerManager;

/// Setup JavaScript polyfills for browser APIs
/// NOTE: ES6-ES2023 language features are natively handled by Boa engine
/// NOTE: Console is now handled by Boa's native console implementation
pub fn setup_all_polyfills(context: &mut Context) -> Result<()> {
    if std::env::var("THALORA_SILENT").is_err() {
        eprintln!("🔧 setup_all_polyfills() - Starting polyfills setup");
    }

    // Console is now handled by Boa's native runtime console

    // Setup browser timer implementation (setTimeout/setInterval)
    if std::env::var("THALORA_SILENT").is_err() {
        eprintln!("🔧 setup_all_polyfills() - Creating TimerManager");
    }
    let timer_manager = TimerManager::new();
    if std::env::var("THALORA_SILENT").is_err() {
        eprintln!("🔧 setup_all_polyfills() - Setting up real timers");
    }
    timer_manager.setup_real_timers(context).map_err(|e| anyhow::Error::msg(format!("Timer setup failed: {:?}", e)))?;
    if std::env::var("THALORA_SILENT").is_err() {
        eprintln!("🔧 setup_all_polyfills() - Real timers setup complete");
    }

    // Web APIs (fetch, websocket, etc.)
    if std::env::var("THALORA_SILENT").is_err() {
        eprintln!("🔧 setup_all_polyfills() - Setting up web APIs");
    }
    web_apis::setup_web_apis(context).map_err(|e| anyhow::Error::msg(format!("Web API setup failed: {:?}", e)))?;
    if std::env::var("THALORA_SILENT").is_err() {
        eprintln!("🔧 setup_all_polyfills() - Web APIs setup complete");
    }

    // Only experimental/future proposal polyfills remain
    if std::env::var("THALORA_SILENT").is_err() {
        eprintln!("🔧 setup_all_polyfills() - Setting up ES2024 polyfills");
    }
    es2024_polyfills::setup_es2024_polyfills(context).map_err(|e| anyhow::Error::msg(format!("ES2024 setup failed: {:?}", e)))?;
    if std::env::var("THALORA_SILENT").is_err() {
        eprintln!("🔧 setup_all_polyfills() - ES2024 polyfills setup complete");
    }

    if std::env::var("THALORA_SILENT").is_err() {
        eprintln!("🔧 setup_all_polyfills() - Setting up ES2025 experimental polyfills");
    }
    es2025_experimental::setup_es2025_experimental(context).map_err(|e| anyhow::Error::msg(format!("ES2025 setup failed: {:?}", e)))?;
    if std::env::var("THALORA_SILENT").is_err() {
        eprintln!("🔧 setup_all_polyfills() - ES2025 experimental polyfills setup complete");
    }

    if std::env::var("THALORA_SILENT").is_err() {
        eprintln!("🔧 setup_all_polyfills() - All polyfills setup complete");
    }
    Ok(())
}