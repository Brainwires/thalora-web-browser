// JavaScript polyfills for browser APIs only
// NOTE: All ES6-ES2023 language features are now natively handled by Boa engine
// NOTE: Console is now handled by Boa's native console implementation
pub mod web_apis;
pub mod syntax_transformer;

// Modular polyfill components
pub mod performance;
pub mod security;
pub mod dom;
pub mod worker;
pub mod css;
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
    // Console is now handled by Boa's native runtime console

    // Setup browser timer implementation (setTimeout/setInterval)
    let timer_manager = TimerManager::new();
    timer_manager.setup_real_timers(context).map_err(|e| anyhow::Error::msg(format!("Timer setup failed: {:?}", e)))?;

    // Web APIs (fetch, websocket, etc.)
    web_apis::setup_web_apis(context).map_err(|e| anyhow::Error::msg(format!("Web API setup failed: {:?}", e)))?;

    // Only experimental/future proposal polyfills remain
    es2024_polyfills::setup_es2024_polyfills(context).map_err(|e| anyhow::Error::msg(format!("ES2024 setup failed: {:?}", e)))?;
    es2025_experimental::setup_es2025_experimental(context).map_err(|e| anyhow::Error::msg(format!("ES2025 setup failed: {:?}", e)))?;

    Ok(())
}