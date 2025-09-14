pub mod globals;
pub mod timers;
pub mod url_api;
pub mod storage;
pub mod crypto_api;
pub mod fetch_api;
pub mod service_worker;

use anyhow::Result;
use boa_engine::{Context, Source};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Web API polyfills for Boa JavaScript engine
pub struct WebPolyfills {
    storage: Arc<Mutex<HashMap<String, String>>>,
    session_storage: Arc<Mutex<HashMap<String, String>>>,
}

impl WebPolyfills {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            session_storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Setup all Web API polyfills in the JavaScript context
    pub fn setup_all_polyfills(&self, context: &mut Context) -> Result<()> {
        // Setup all polyfill modules
        globals::setup_globals(context)?;
        timers::setup_timers(context)?;
        url_api::setup_url_api(context)?;
        storage::setup_storage(context)?;
        crypto_api::setup_crypto(context)?;
        fetch_api::setup_fetch(context)?;
        service_worker::setup_service_worker(context)?;

        Ok(())
    }

    /// Get current localStorage data
    pub fn get_local_storage_data(&self) -> HashMap<String, String> {
        self.storage.lock().unwrap().clone()
    }

    /// Get current sessionStorage data
    pub fn get_session_storage_data(&self) -> HashMap<String, String> {
        self.session_storage.lock().unwrap().clone()
    }

    /// Clear all storage
    pub fn clear_all_storage(&self) {
        self.storage.lock().unwrap().clear();
        self.session_storage.lock().unwrap().clear();
    }
}