//! IDBFactory Implementation
//!
//! The IDBFactory interface provides access to IndexedDB functionality.
//! It's available as window.indexedDB.
//!
//! Spec: https://w3c.github.io/IndexedDB/#factory-interface

use super::backend::{StorageBackend, SledBackend};
use super::database::IDBDatabase;
use super::key::IDBKey;
use super::request::{IDBRequest, IDBOpenDBRequest};
use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, JsValue,
};
use boa_gc::{Finalize, Trace};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// IDBFactory - the main entry point for IndexedDB
#[derive(Clone, Finalize)]
pub struct IDBFactory {
    /// Storage backend (Sled for persistence)
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
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // Backend doesn't contain GC'd objects
    }

    unsafe fn trace_non_roots(&self) {}

    fn run_finalizer(&self) {}
}

impl JsData for IDBFactory {}

impl IDBFactory {
    /// Create a new IDBFactory with default storage path
    pub fn new() -> Result<Self, String> {
        let storage_path = Self::get_default_storage_path();
        Self::with_path(storage_path)
    }

    /// Create a new IDBFactory with custom storage path
    pub fn with_path(path: PathBuf) -> Result<Self, String> {
        let backend = SledBackend::new(path)?;
        Ok(Self {
            backend: Arc::new(Mutex::new(Box::new(backend))),
        })
    }

    /// Get default storage path
    fn get_default_storage_path() -> PathBuf {
        // In test mode, use a unique temporary directory to avoid lock conflicts
        #[cfg(test)]
        {
            use std::sync::atomic::{AtomicU64, Ordering};
            static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);
            let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
            let mut path = std::env::temp_dir();
            path.push(format!("thalora-test-{}-{}", std::process::id(), counter));
            path.push("indexeddb");
            return path;
        }

        #[cfg(not(test))]
        {
            let mut path = dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."));
            path.push("thalora");
            path.push("indexeddb");
            path
        }
    }

    /// indexedDB.open(name, version?)
    /// Opens a database connection
    pub(crate) fn open(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBFactory object"))?;

        let factory = obj.downcast_ref::<IDBFactory>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBFactory object"))?;

        // Parse arguments
        let name = args.get_or_undefined(0).to_string(context)?;
        let name_str = name.to_std_string_escaped();

        let version = args.get(1)
            .map(|v| v.to_u32(context))
            .transpose()?
            .unwrap_or(1);

        // Validate version
        if version == 0 {
            return Err(JsNativeError::typ()
                .with_message("Version cannot be 0")
                .into());
        }

        // Create IDBOpenDBRequest
        let request = IDBOpenDBRequest::new();

        // Create request object for JavaScript - use IDBRequest prototype
        // IDBOpenDBRequest inherits from IDBRequest
        let request_proto = IDBRequest::get(context.intrinsics())
            .get(js_string!("prototype"), context)?
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("IDBRequest prototype not found"))?
            .clone();

        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            request_proto,
            request.clone(),
        );

        // Perform async open operation
        let backend = factory.backend.clone();
        let request_clone = request.clone();

        // For now, execute synchronously (would be async in real implementation)
        let result = {
            let mut backend = backend.lock().unwrap();
            backend.open_database(&name_str, version)
        };

        match result {
            Ok(handle) => {
                if handle.needs_upgrade {
                    // Trigger upgradeneeded event
                    request_clone.trigger_upgradeneeded(
                        handle.old_version,
                        handle.version,
                        context
                    )?;
                }

                // Create IDBDatabase object
                let db = IDBDatabase::new(
                    handle.name.clone(),
                    handle.version,
                    handle.object_stores.clone(),
                    backend.clone(),
                );

                let db_obj = JsObject::from_proto_and_data_with_shared_shape(
                    context.root_shape(),
                    context.intrinsics().constructors().object().prototype(),
                    db,
                );

                request_clone.base().set_result(db_obj.into());
                request_clone.base().trigger_success(context)?;
            }
            Err(e) => {
                request_clone.base().set_error(e);
                request_clone.base().trigger_error(context)?;
            }
        }

        Ok(request_obj.into())
    }

    /// indexedDB.deleteDatabase(name)
    /// Deletes a database
    pub(crate) fn delete_database(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBFactory object"))?;

        let factory = obj.downcast_ref::<IDBFactory>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBFactory object"))?;

        let name = args.get_or_undefined(0).to_string(context)?;
        let name_str = name.to_std_string_escaped();

        // Create request
        let request = IDBRequest::new();
        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            request.clone(),
        );

        // Perform deletion
        let backend = factory.backend.clone();
        let result = {
            let mut backend = backend.lock().unwrap();
            backend.delete_database(&name_str)
        };

        match result {
            Ok(()) => {
                request.set_result(JsValue::undefined());
                request.trigger_success(context)?;
            }
            Err(e) => {
                request.set_error(e);
                request.trigger_error(context)?;
            }
        }

        Ok(request_obj.into())
    }

    /// indexedDB.databases()
    /// Returns a list of all available databases
    pub(crate) fn databases(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBFactory object"))?;

        let factory = obj.downcast_ref::<IDBFactory>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBFactory object"))?;

        let backend = factory.backend.clone();
        let databases = {
            let backend = backend.lock().unwrap();
            backend.databases()
                .map_err(|e| JsNativeError::error().with_message(e))?
        };

        // Convert to JavaScript array
        use boa_engine::builtins::array::Array;
        let array = Array::array_create(databases.len() as u64, None, context)?;

        for (i, (name, version)) in databases.iter().enumerate() {
            let db_info = JsObject::with_object_proto(context.intrinsics());
            db_info.set(js_string!("name"), JsValue::from(JsString::from(name.clone())), false, context)?;
            db_info.set(js_string!("version"), JsValue::from(*version), false, context)?;

            array.set(i, db_info, true, context)?;
        }

        Ok(array.into())
    }

    /// indexedDB.cmp(first, second)
    /// Compares two keys
    pub(crate) fn cmp(
        _this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let first = args.get_or_undefined(0);
        let second = args.get_or_undefined(1);

        let key1 = IDBKey::from_js_value(first, context)?;
        let key2 = IDBKey::from_js_value(second, context)?;

        let result = match key1.cmp(&key2) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        };

        Ok(JsValue::from(result))
    }
}

impl Default for IDBFactory {
    fn default() -> Self {
        Self::new().expect("Failed to create IDBFactory")
    }
}

impl IntrinsicObject for IDBFactory {
    fn init(realm: &Realm) {
        let _intrinsics = realm.intrinsics();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Methods
            .method(Self::open, js_string!("open"), 1)
            .method(Self::delete_database, js_string!("deleteDatabase"), 1)
            .method(Self::databases, js_string!("databases"), 0)
            .method(Self::cmp, js_string!("cmp"), 2)
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
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;  // Estimated prototype property count
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;  // Constructor properties

    const CONSTRUCTOR_ARGUMENTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&boa_engine::context::intrinsics::StandardConstructors) -> &StandardConstructor =
        |constructors| {
            constructors.idb_factory()  // Use IDBFactory's own intrinsic slot
        };

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // IDBFactory is not constructible
        Err(JsNativeError::typ()
            .with_message("Illegal constructor")
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_creation() {
        let factory = IDBFactory::new();
        assert!(factory.is_ok());
    }

    #[test]
    fn test_key_comparison() {
        let key1 = IDBKey::Number(1.0);
        let key2 = IDBKey::Number(2.0);
        let key3 = IDBKey::Number(1.0);

        assert!(key1 < key2);
        assert_eq!(key1, key3);
    }
}
