use anyhow::Result;
use boa_engine::Context;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Simple WebStorage that delegates to JavaScript polyfills
pub struct WebStorage {
    local_storage: Arc<Mutex<HashMap<String, String>>>,
    session_storage: Arc<Mutex<HashMap<String, String>>>,
}

impl WebStorage {
    pub fn new() -> Self {
        Self {
            local_storage: Arc::new(Mutex::new(HashMap::new())),
            session_storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Setup storage - delegates to the storage_polyfills.rs JavaScript implementation
    pub fn setup_storage_globals(&self, context: &mut Context) -> Result<()> {
        // The actual storage implementation is done via JavaScript polyfills
        // This is just a holder for the Rust-side storage data
        crate::dom::storage_polyfills::setup_storage(context)
    }

    pub fn get_local_storage_data(&self) -> HashMap<String, String> {
        self.local_storage.lock().unwrap().clone()
    }

    pub fn get_session_storage_data(&self) -> HashMap<String, String> {
        self.session_storage.lock().unwrap().clone()
    }

    pub fn clear_storage(&self) {
        self.local_storage.lock().unwrap().clear();
        self.session_storage.lock().unwrap().clear();
    }
}