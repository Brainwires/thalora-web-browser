// Web APIs and standards implementation
// crypto API is now natively implemented in Boa engine
// fetch API is now natively implemented in Boa engine
pub mod url_api;

// Service workers require core networking (tokio, reqwest)
#[cfg(feature = "core")]
pub mod service_worker;

// websocket API is now natively implemented in Boa engine
// Storage APIs are now natively implemented in Boa engine
// events API is now natively implemented in Boa engine
// timers API is now natively implemented in Boa engine
// navigator API is now natively implemented in Boa engine
pub mod dom_native;
pub mod credentials;

// Full-featured browser APIs
// webassembly API is now natively implemented in Boa engine

// Native-only: Geolocation uses sysinfo and ipgeolocate
#[cfg(feature = "native")]
pub mod geolocation;

// webrtc API is now natively implemented in Boa engine

// Native-only: Media uses cpal, rodio, ffmpeg
#[cfg(feature = "native")]
pub mod media;

// JavaScript polyfills organized by modules
pub mod polyfills;

use anyhow::Result;
use thalora_browser_apis::boa_engine::Context;

/// Modern Web APIs implementation for headless browser
pub struct WebApis;

impl WebApis {
    pub fn new() -> Self {
        Self
    }

    /// Setup all Web API implementations in the JavaScript context
    pub fn setup_all_apis(&self, context: &mut Context) -> Result<()> {
        // Console is now handled by Boa's native console implementation

        // navigator API is now natively handled by Boa engine

        // Setup Credential Management API
        let credential_manager = credentials::CredentialManager::new();
        credential_manager.setup_credentials_api(context).map_err(|e| anyhow::Error::msg(format!("Credentials API setup failed: {:?}", e)))?;

        // Setup all Web API modules
        url_api::setup_url_api(context)?;
        // crypto API is now natively handled by Boa engine
        // fetch API is now natively handled by Boa engine
        // websocket API is now natively handled by Boa engine

        // timers (setTimeout/setInterval) are now natively handled by Boa engine

        // Service Worker setup (requires core networking)
        #[cfg(feature = "core")]
        {
            let sw_manager = service_worker::ServiceWorkerManager::new().map_err(|e| anyhow::Error::msg(format!("Service worker manager creation failed: {:?}", e)))?;
            sw_manager.setup_service_worker_api(context).map_err(|e| anyhow::Error::msg(format!("Service worker setup failed: {:?}", e)))?;
        }

        // Storage APIs (localStorage/sessionStorage) are now natively implemented in Boa engine

        // Setup full-featured browser APIs
        // WebAssembly API is now natively implemented in Boa engine

        // Native-only: Geolocation setup
        #[cfg(feature = "native")]
        {
            let geo_manager = geolocation::GeolocationManager::new();
            geo_manager.setup_geolocation_api(context).map_err(|e| anyhow::Error::msg(format!("Geolocation setup failed: {:?}", e)))?;
        }

        // webrtc API is now natively implemented in Boa engine

        // Native-only: Media APIs setup
        #[cfg(feature = "native")]
        {
            let media_manager = media::MediaManager::new().map_err(|e| anyhow::Error::msg(format!("Media manager creation failed: {:?}", e)))?;
            media_manager.setup_media_apis(context).map_err(|e| anyhow::Error::msg(format!("Media APIs setup failed: {:?}", e)))?;
        }

        // events API (Event, EventTarget, CustomEvent) is now natively handled by Boa engine

        // DISABLED: Polyfill system is disabled to focus on real DOM implementation
        // Only using native DOM from Boa engine
        // polyfills::setup_all_polyfills(context).map_err(|e| anyhow::Error::msg(format!("Polyfill setup failed: {:?}", e)))?;

        Ok(())
    }
}