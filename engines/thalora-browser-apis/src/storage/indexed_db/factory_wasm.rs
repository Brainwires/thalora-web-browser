//! IDBFactory stub for WASM builds
//!
//! In WASM builds, the browser's native IndexedDB is used directly.

use super::backend::{StorageBackend, MemoryBackend};
use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    realm::Realm,
    Context, JsData, JsNativeError, JsResult, JsString, JsValue,
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};

/// IDBFactory for WASM - uses MemoryBackend for basic compatibility
/// Real WASM apps should use the browser's native IndexedDB
#[derive(Clone, Finalize)]
pub struct IDBFactory {
    backend: Arc<Mutex<Box<dyn StorageBackend>>>,
}

impl std::fmt::Debug for IDBFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IDBFactory")
            .field("backend", &"<StorageBackend>")
            .finish()
    }
}

unsafe impl Trace for IDBFactory {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {}
    unsafe fn trace_non_roots(&self) {}
    fn run_finalizer(&self) {}
}

impl JsData for IDBFactory {}

impl IDBFactory {
    /// Create a new IDBFactory with in-memory storage
    pub fn new() -> Result<Self, String> {
        let backend = MemoryBackend::new();
        Ok(Self {
            backend: Arc::new(Mutex::new(Box::new(backend))),
        })
    }
}

impl IntrinsicObject for IDBFactory {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(open, js_string!("open"), 1)
            .method(delete_database, js_string!("deleteDatabase"), 1)
            .method(databases, js_string!("databases"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for IDBFactory {
    const NAME: JsString = js_string!("IDBFactory");
}

impl BuiltInConstructor for IDBFactory {
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::idb_factory;

    fn constructor(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("IDBFactory cannot be constructed. Use window.indexedDB or the browser's native IndexedDB in WASM.")
            .into())
    }
}

fn open(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Err(JsNativeError::error()
        .with_message("IndexedDB.open() is not available in WASM. Use the browser's native IndexedDB API.")
        .into())
}

fn delete_database(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Err(JsNativeError::error()
        .with_message("IndexedDB.deleteDatabase() is not available in WASM. Use the browser's native IndexedDB API.")
        .into())
}

fn databases(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Err(JsNativeError::error()
        .with_message("IndexedDB.databases() is not available in WASM. Use the browser's native IndexedDB API.")
        .into())
}
