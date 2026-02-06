//! IDBTransaction Implementation
//!
//! Provides a static, asynchronous transaction on a database.
//!
//! Spec: https://w3c.github.io/IndexedDB/#transaction

use super::backend::{StorageBackend, TransactionMode};
use super::object_store::IDBObjectStore;
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
use std::sync::{Arc, Mutex};

/// Transaction state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    Active,
    Inactive,
    Committing,
    Finished,
}

/// IDBTransaction provides transaction management
#[derive(Clone, Finalize)]
pub struct IDBTransaction {
    /// Transaction mode
    mode: TransactionMode,

    /// Object store names in this transaction
    object_store_names: Vec<String>,

    /// Storage backend
    backend: Arc<Mutex<Box<dyn StorageBackend>>>,

    /// Database name
    db_name: String,

    /// Transaction state
    state: Arc<Mutex<TransactionState>>,

    /// Transaction ID (if started)
    transaction_id: Arc<Mutex<Option<u64>>>,

    /// Error state
    error: Arc<Mutex<Option<String>>>,

    /// Event handlers
    onabort: Arc<Mutex<Option<JsObject>>>,
    oncomplete: Arc<Mutex<Option<JsObject>>>,
    onerror: Arc<Mutex<Option<JsObject>>>,
}

impl std::fmt::Debug for IDBTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IDBTransaction")
            .field("mode", &self.mode)
            .field("object_store_names", &self.object_store_names)
            .field("state", &*self.state.lock().unwrap())
            .finish()
    }
}

unsafe impl Trace for IDBTransaction {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // Backend doesn't contain GC'd objects
    }

    unsafe fn trace_non_roots(&self) {
        // No non-root tracing needed
    }

    fn run_finalizer(&self) {
        // No finalizer needed
    }
}

impl JsData for IDBTransaction {}

impl IDBTransaction {
    /// Create a new IDBTransaction
    pub fn new(
        backend: Arc<Mutex<Box<dyn StorageBackend>>>,
        db_name: String,
        stores: Vec<String>,
        mode: TransactionMode,
    ) -> Self {
        Self {
            mode,
            object_store_names: stores,
            backend,
            db_name,
            state: Arc::new(Mutex::new(TransactionState::Active)),
            transaction_id: Arc::new(Mutex::new(None)),
            error: Arc::new(Mutex::new(None)),
            onabort: Arc::new(Mutex::new(None)),
            oncomplete: Arc::new(Mutex::new(None)),
            onerror: Arc::new(Mutex::new(None)),
        }
    }

    /// Get the transaction mode
    pub fn mode(&self) -> TransactionMode {
        self.mode
    }

    /// Get the transaction state
    pub fn state(&self) -> TransactionState {
        *self.state.lock().unwrap()
    }

    /// Check if the transaction is active
    pub fn is_active(&self) -> bool {
        self.state() == TransactionState::Active
    }

    /// Ensure the transaction is active
    fn ensure_active(&self) -> Result<(), String> {
        if !self.is_active() {
            return Err("Transaction is not active".to_string());
        }
        Ok(())
    }

    /// Begin the backend transaction if not already started
    fn ensure_transaction_started(&self) -> Result<(), String> {
        let mut txn_id = self.transaction_id.lock().unwrap();
        if txn_id.is_none() {
            let mut backend = self.backend.lock().unwrap();
            let handle = backend.begin_transaction(&self.object_store_names, self.mode)?;
            *txn_id = Some(handle.id);
        }
        Ok(())
    }

    /// Commit the transaction
    pub fn commit(&self) -> Result<(), String> {
        self.ensure_active()?;
        self.ensure_transaction_started()?;

        // Set state to committing
        *self.state.lock().unwrap() = TransactionState::Committing;

        // Commit in backend
        let txn_id = self.transaction_id.lock().unwrap();
        if let Some(id) = *txn_id {
            let mut backend = self.backend.lock().unwrap();
            backend.commit_transaction(id)?;
        }

        // Set state to finished
        *self.state.lock().unwrap() = TransactionState::Finished;
        Ok(())
    }

    /// Abort the transaction
    pub fn abort(&self) -> Result<(), String> {
        let current_state = self.state();
        if current_state == TransactionState::Committing || current_state == TransactionState::Finished {
            return Err("Cannot abort transaction in current state".to_string());
        }

        // Abort in backend if started
        let txn_id = self.transaction_id.lock().unwrap();
        if let Some(id) = *txn_id {
            let mut backend = self.backend.lock().unwrap();
            backend.abort_transaction(id)?;
        }

        // Set state to finished
        *self.state.lock().unwrap() = TransactionState::Finished;
        Ok(())
    }

    /// Get objectStoreNames property
    pub(crate) fn get_object_store_names(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        let txn = obj.downcast_ref::<IDBTransaction>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        // Create array of store names
        use boa_engine::builtins::array::Array;
        let array = Array::array_create(txn.object_store_names.len() as u64, None, context)?;

        for (i, name) in txn.object_store_names.iter().enumerate() {
            array.set(i, JsValue::from(JsString::from(name.clone())), true, context)?;
        }

        Ok(array.into())
    }

    /// Get mode property
    pub(crate) fn get_mode(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        let txn = obj.downcast_ref::<IDBTransaction>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        let mode_str = match txn.mode {
            TransactionMode::ReadOnly => "readonly",
            TransactionMode::ReadWrite => "readwrite",
            TransactionMode::VersionChange => "versionchange",
        };

        Ok(JsValue::from(JsString::from(mode_str)))
    }

    /// Get error property
    pub(crate) fn get_error(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        let txn = obj.downcast_ref::<IDBTransaction>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        let error = txn.error.lock().unwrap();
        if let Some(err) = error.as_ref() {
            Ok(JsValue::from(JsString::from(err.clone())))
        } else {
            Ok(JsValue::null())
        }
    }

    /// objectStore(name)
    /// Returns an object store in the transaction's scope
    pub(crate) fn object_store(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        let txn = obj.downcast_ref::<IDBTransaction>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        // Check if transaction is active
        txn.ensure_active()
            .map_err(|e| JsNativeError::error().with_message(e))?;

        // Parse store name
        let name = args.get_or_undefined(0).to_string(context)?;
        let name_str = name.to_std_string_escaped();

        // Validate store is in transaction scope
        if !txn.object_store_names.contains(&name_str) {
            return Err(JsNativeError::error()
                .with_message(format!("Object store '{}' is not in transaction scope", name_str))
                .into());
        }

        // Get object store metadata from backend
        let metadata = {
            let backend = txn.backend.lock().unwrap();
            backend.get_object_store_metadata(&txn.db_name, &name_str)
                .map_err(|e| JsNativeError::error().with_message(e))?
        };

        // Create IDBObjectStore object
        let store = IDBObjectStore::new(
            name_str,
            metadata.key_path,
            metadata.auto_increment,
            txn.backend.clone(),
            txn.db_name.clone(),
        );

        let store_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            store,
        );

        Ok(store_obj.into())
    }

    /// commit()
    /// Commits the transaction
    pub(crate) fn commit_method(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        let txn = obj.downcast_ref::<IDBTransaction>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        txn.commit()
            .map_err(|e| JsNativeError::error().with_message(e))?;

        // Trigger oncomplete event
        let handler = txn.oncomplete.lock().unwrap();
        if let Some(callback) = handler.as_ref() {
            if callback.is_callable() {
                let event = Self::create_event("complete", context)?;
                callback.call(&JsValue::undefined(), &[event], context)?;
            }
        }

        Ok(JsValue::undefined())
    }

    /// abort()
    /// Aborts the transaction
    pub(crate) fn abort_method(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        let txn = obj.downcast_ref::<IDBTransaction>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        txn.abort()
            .map_err(|e| JsNativeError::error().with_message(e))?;

        // Trigger onabort event
        let handler = txn.onabort.lock().unwrap();
        if let Some(callback) = handler.as_ref() {
            if callback.is_callable() {
                let event = Self::create_event("abort", context)?;
                callback.call(&JsValue::undefined(), &[event], context)?;
            }
        }

        Ok(JsValue::undefined())
    }

    /// Create a simple event object
    fn create_event(event_type: &str, context: &mut Context) -> JsResult<JsValue> {
        let event = JsObject::with_object_proto(context.intrinsics());
        event.set(js_string!("type"), JsValue::from(JsString::from(event_type)), false, context)?;
        Ok(event.into())
    }

    /// Set onabort handler
    pub(crate) fn set_onabort(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        let txn = obj.downcast_ref::<IDBTransaction>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        let handler = args.get_or_undefined(0);
        let mut onabort = txn.onabort.lock().unwrap();

        if handler.is_callable() {
            *onabort = handler.as_object().map(|obj| obj.clone());
        } else {
            *onabort = None;
        }

        Ok(JsValue::undefined())
    }

    /// Set oncomplete handler
    pub(crate) fn set_oncomplete(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        let txn = obj.downcast_ref::<IDBTransaction>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        let handler = args.get_or_undefined(0);
        let mut oncomplete = txn.oncomplete.lock().unwrap();

        if handler.is_callable() {
            *oncomplete = handler.as_object().map(|obj| obj.clone());
        } else {
            *oncomplete = None;
        }

        Ok(JsValue::undefined())
    }

    /// Set onerror handler
    pub(crate) fn set_onerror(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        let txn = obj.downcast_ref::<IDBTransaction>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBTransaction object"))?;

        let handler = args.get_or_undefined(0);
        let mut onerror = txn.onerror.lock().unwrap();

        if handler.is_callable() {
            *onerror = handler.as_object().map(|obj| obj.clone());
        } else {
            *onerror = None;
        }

        Ok(JsValue::undefined())
    }
}

impl IntrinsicObject for IDBTransaction {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(realm.intrinsics().constructors().event_target().prototype()))
            // Properties
            .accessor(
                js_string!("objectStoreNames"),
                Some(BuiltInBuilder::callable(realm, Self::get_object_store_names)
                    .name(js_string!("get objectStoreNames"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("mode"),
                Some(BuiltInBuilder::callable(realm, Self::get_mode)
                    .name(js_string!("get mode"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("error"),
                Some(BuiltInBuilder::callable(realm, Self::get_error)
                    .name(js_string!("get error"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onabort"),
                None,
                Some(BuiltInBuilder::callable(realm, Self::set_onabort)
                    .name(js_string!("set onabort"))
                    .build()),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("oncomplete"),
                None,
                Some(BuiltInBuilder::callable(realm, Self::set_oncomplete)
                    .name(js_string!("set oncomplete"))
                    .build()),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onerror"),
                None,
                Some(BuiltInBuilder::callable(realm, Self::set_onerror)
                    .name(js_string!("set onerror"))
                    .build()),
                Attribute::CONFIGURABLE,
            )
            // Methods
            .method(Self::object_store, js_string!("objectStore"), 1)
            .method(Self::commit_method, js_string!("commit"), 0)
            .method(Self::abort_method, js_string!("abort"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for IDBTransaction {
    const NAME: JsString = js_string!("IDBTransaction");
}

impl BuiltInConstructor for IDBTransaction {
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;  // Estimated prototype property count
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;  // Constructor properties

    const CONSTRUCTOR_ARGUMENTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&boa_engine::context::intrinsics::StandardConstructors) -> &StandardConstructor =
        |intrinsics| intrinsics.idb_transaction();

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("IDBTransaction constructor cannot be called directly")
            .into())
    }
}

impl Default for IDBTransaction {
    fn default() -> Self {
        Self {
            mode: TransactionMode::ReadOnly,
            object_store_names: Vec::new(),
            backend: Arc::new(Mutex::new(Box::new(super::backend::memory::MemoryBackend::new()))),
            db_name: String::new(),
            state: Arc::new(Mutex::new(TransactionState::Active)),
            transaction_id: Arc::new(Mutex::new(None)),
            error: Arc::new(Mutex::new(None)),
            onabort: Arc::new(Mutex::new(None)),
            oncomplete: Arc::new(Mutex::new(None)),
            onerror: Arc::new(Mutex::new(None)),
        }
    }
}
