// Web APIs and standards implementation
pub mod crypto_api;
pub mod fetch_api;
pub mod url_api;
pub mod service_worker;
pub mod websocket;
pub mod storage;
pub mod events;

// JavaScript polyfills organized by modules
pub mod polyfills;

use anyhow::Result;
use boa_engine::Context;

/// Modern Web APIs implementation for headless browser
pub struct WebApis;

impl WebApis {
    pub fn new() -> Self {
        Self
    }

    /// Setup all Web API implementations in the JavaScript context
    pub fn setup_all_apis(&self, context: &mut Context) -> Result<()> {
        // Setup all Web API modules
        url_api::setup_url_api(context)?;
        crypto_api::setup_crypto(context)?;
        fetch_api::setup_fetch(context)?;
        let sw_manager = service_worker::ServiceWorkerManager::new();
        sw_manager.setup_service_worker_api(context).map_err(|e| anyhow::Error::msg(format!("Service worker setup failed: {:?}", e)))?;
        let web_storage = storage::WebStorage::new();
        web_storage.setup_storage_globals(context)?;
        // events::setup_event_system(context)?; // TODO: Check if this function exists

        Ok(())
    }
}