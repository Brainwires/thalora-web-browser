//! Realm-scoped browsing-context state.
//!
//! Stores the active origin and worker flag on the JS realm via
//! `Realm::host_defined()`. OPFS uses this to scope `navigator.storage.getDirectory()`
//! per origin and to decide whether `createSyncAccessHandle()` is allowed
//! (workers only, per WHATWG File System spec).
//!
//! See `engines/boa/core/engine/src/host_defined.rs` for the storage primitive.

use boa_engine::{Context, JsData};
use boa_gc::{Finalize, Trace};

const DEFAULT_ORIGIN: &str = "thalora://local";

#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct ThaloraRealmContext {
    #[unsafe_ignore_trace]
    origin: String,
    is_worker: bool,
}

impl ThaloraRealmContext {
    pub fn new(origin: String, is_worker: bool) -> Self {
        Self { origin, is_worker }
    }

    pub fn origin(&self) -> &str {
        &self.origin
    }

    pub fn is_worker(&self) -> bool {
        self.is_worker
    }
}

/// Install the realm context. Replaces any existing entry.
pub fn install(context: &mut Context, origin: String, is_worker: bool) {
    let realm = context.realm().clone();
    realm
        .host_defined_mut()
        .insert(ThaloraRealmContext::new(origin, is_worker));
}

/// Update the active origin on the existing realm context.
/// Called when navigation completes and `window.location` changes.
pub fn set_active_origin(context: &mut Context, origin: String) {
    let realm = context.realm().clone();
    let is_worker = realm
        .host_defined()
        .get::<ThaloraRealmContext>()
        .map(ThaloraRealmContext::is_worker)
        .unwrap_or(false);
    realm
        .host_defined_mut()
        .insert(ThaloraRealmContext::new(origin, is_worker));
}

/// Read the current origin. Falls back to `"thalora://local"` if no context
/// has been installed (e.g. tests that don't call `initialize_browser_apis`).
pub fn current_origin(context: &Context) -> String {
    context
        .realm()
        .host_defined()
        .get::<ThaloraRealmContext>()
        .map(|ctx| ctx.origin().to_string())
        .unwrap_or_else(|| DEFAULT_ORIGIN.to_string())
}

/// True if the current realm is a worker. Used to gate
/// `FileSystemFileHandle.createSyncAccessHandle()` per spec.
pub fn is_worker(context: &Context) -> bool {
    context
        .realm()
        .host_defined()
        .get::<ThaloraRealmContext>()
        .map(ThaloraRealmContext::is_worker)
        .unwrap_or(false)
}
