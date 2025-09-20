// Web APIs and standards implementation
pub mod crypto_api;
pub mod fetch_api;
pub mod url_api;
pub mod service_worker;
pub mod websocket;
pub mod storage;
pub mod events;
pub mod timers;
pub mod navigator;
pub mod dom_native;
// pub mod credentials; // Temporarily disabled due to threading issues

// Full-featured browser APIs
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
        // Console is now handled by Boa's native console implementation

        // Setup core browser APIs
        let navigator_manager = navigator::NavigatorManager::new();
        navigator_manager.setup_navigator_api(context).map_err(|e| anyhow::Error::msg(format!("Navigator setup failed: {:?}", e)))?;

        // Setup Credential Management API (temporarily disabled due to threading issues)
        // let credential_manager = credentials::CredentialManager::new();
        // credential_manager.setup_credentials_api(context).map_err(|e| anyhow::Error::msg(format!("Credentials API setup failed: {:?}", e)))?;

        // Setup all Web API modules
        url_api::setup_url_api(context)?;
        crypto_api::setup_crypto(context)?;
        fetch_api::setup_fetch(context)?;

        // Setup screen global (alias for window.screen)
        self.setup_screen_global(context)?;

        // Setup WebSocket API
        let websocket_manager = websocket::WebSocketManager::new();
        let websocket_api = websocket::WebSocketJsApi::new(websocket_manager);
        websocket_api.setup_websocket_globals(context).map_err(|e| anyhow::Error::msg(format!("WebSocket setup failed: {:?}", e)))?;

        // Setup real timer implementation
        let timer_manager = timers::TimerManager::new();
        timer_manager.setup_real_timers(context).map_err(|e| anyhow::Error::msg(format!("Timer setup failed: {:?}", e)))?;

        let sw_manager = service_worker::ServiceWorkerManager::new().map_err(|e| anyhow::Error::msg(format!("Service worker manager creation failed: {:?}", e)))?;
        sw_manager.setup_service_worker_api(context).map_err(|e| anyhow::Error::msg(format!("Service worker setup failed: {:?}", e)))?;

        let web_storage = storage::WebStorage::new();
        web_storage.setup_storage_globals(context)?;

        // Setup full-featured browser APIs
        let wasm_manager = webassembly::WebAssemblyManager::new();
        wasm_manager.setup_webassembly_api(context).map_err(|e| anyhow::Error::msg(format!("WebAssembly setup failed: {:?}", e)))?;

        let geo_manager = geolocation::GeolocationManager::new();
        geo_manager.setup_geolocation_api(context).map_err(|e| anyhow::Error::msg(format!("Geolocation setup failed: {:?}", e)))?;

        let webrtc_manager = webrtc::WebRTCManager::new().map_err(|e| anyhow::Error::msg(format!("WebRTC manager creation failed: {:?}", e)))?;
        webrtc_manager.setup_webrtc_api(context).map_err(|e| anyhow::Error::msg(format!("WebRTC setup failed: {:?}", e)))?;

        let media_manager = media::MediaManager::new().map_err(|e| anyhow::Error::msg(format!("Media manager creation failed: {:?}", e)))?;
        media_manager.setup_media_apis(context).map_err(|e| anyhow::Error::msg(format!("Media APIs setup failed: {:?}", e)))?;

        // Setup comprehensive DOM Events system
        let event_manager = events::EventManager::new();
        event_manager.setup_events_api(context).map_err(|e| anyhow::Error::msg(format!("Events API setup failed: {:?}", e)))?;

        Ok(())
    }

    /// Setup screen global (alias for window.screen)
    fn setup_screen_global(&self, context: &mut Context) -> Result<()> {
        use boa_engine::{js_string, JsValue};

        // Execute JavaScript to create screen global as alias for window.screen
        let script = r#"
            (function() {
                if (typeof window !== 'undefined' && window.screen) {
                    // Set screen as a global variable (alias for window.screen)
                    globalThis.screen = window.screen;
                    return true;
                } else {
                    console.error('window.screen not available for global screen alias');
                    return false;
                }
            })();
        "#;

        let result = context.eval(boa_engine::Source::from_bytes(script))
            .map_err(|e| anyhow::Error::msg(format!("Failed to execute screen global setup: {:?}", e)))?;

        eprintln!("Screen global setup result: {:?}", result.to_string(context));

        Ok(())
    }
}