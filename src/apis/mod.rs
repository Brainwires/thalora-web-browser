// Web APIs and standards implementation
pub mod crypto_api;
pub mod fetch_api;
pub mod url_api;
pub mod service_worker;
pub mod websocket;
pub mod storage;
pub mod events;

// Real functional implementations (not mocks)
pub mod webassembly_real;
pub mod geolocation_real;
pub mod webrtc_real;
pub mod media_real;

// Legacy mock implementations (deprecated)
pub mod webassembly;
pub mod geolocation;
pub mod webrtc;
pub mod media;

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

        // Setup REAL full-featured browser APIs (not mocks)
        let wasm_manager = webassembly_real::WebAssemblyManager::new();
        wasm_manager.setup_webassembly_api(context).map_err(|e| anyhow::Error::msg(format!("Real WebAssembly setup failed: {:?}", e)))?;

        let geo_manager = geolocation_real::GeolocationManager::new();
        geo_manager.setup_geolocation_api(context).map_err(|e| anyhow::Error::msg(format!("Real Geolocation setup failed: {:?}", e)))?;

        let webrtc_manager = webrtc_real::WebRTCManager::new().map_err(|e| anyhow::Error::msg(format!("WebRTC manager creation failed: {:?}", e)))?;
        webrtc_manager.setup_webrtc_api(context).map_err(|e| anyhow::Error::msg(format!("Real WebRTC setup failed: {:?}", e)))?;

        let media_manager = media_real::MediaManager::new().map_err(|e| anyhow::Error::msg(format!("Media manager creation failed: {:?}", e)))?;
        media_manager.setup_media_apis(context).map_err(|e| anyhow::Error::msg(format!("Real Media APIs setup failed: {:?}", e)))?;

        // events::setup_event_system(context)?; // TODO: Check if this function exists

        Ok(())
    }
}