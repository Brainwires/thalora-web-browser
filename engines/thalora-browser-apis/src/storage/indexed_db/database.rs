//! IDBDatabase Implementation
//!
//! Represents a connection to a database.
//!
//! Spec: https://w3c.github.io/IndexedDB/#database-interface

use super::backend::{StorageBackend, TransactionMode};
use super::object_store::IDBObjectStore;
use super::transaction::IDBTransaction;
use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};

/// IDBDatabase represents a connection to a database
#[derive(Clone, Finalize)]
pub struct IDBDatabase {
    /// Database name
    name: String,

    /// Database version
    version: u32,

    /// List of object store names
    object_store_names: Vec<String>,

    /// Storage backend
    backend: Arc<Mutex<Box<dyn StorageBackend>>>,

    /// Whether the database is closed
    closed: Arc<Mutex<bool>>,

    /// onabort event handler
    onabort: Arc<Mutex<Option<JsObject>>>,

    /// onerror event handler
    onerror: Arc<Mutex<Option<JsObject>>>,

    /// onversionchange event handler
    onversionchange: Arc<Mutex<Option<JsObject>>>,
}

impl std::fmt::Debug for IDBDatabase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IDBDatabase")
            .field("name", &self.name)
            .field("version", &self.version)
            .field("object_store_names", &self.object_store_names)
            .finish()
    }
}

unsafe impl Trace for IDBDatabase {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // Backend doesn't contain GC'd objects
        // Event handlers are in Arc<Mutex> so not directly traced
    }

    unsafe fn trace_non_roots(&self) {
        // No non-root tracing needed
    }

    fn run_finalizer(&self) {
        // No finalizer needed
    }
}

impl JsData for IDBDatabase {}

impl IDBDatabase {
    /// Create a new IDBDatabase
    pub fn new(
        name: String,
        version: u32,
        object_store_names: Vec<String>,
        backend: Arc<Mutex<Box<dyn StorageBackend>>>,
    ) -> Self {
        Self {
            name,
            version,
            object_store_names,
            backend,
            closed: Arc::new(Mutex::new(false)),
            onabort: Arc::new(Mutex::new(None)),
            onerror: Arc::new(Mutex::new(None)),
            onversionchange: Arc::new(Mutex::new(None)),
        }
    }

    /// Get the database name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the database version
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Get object store names
    pub fn object_store_names(&self) -> &[String] {
        &self.object_store_names
    }

    /// Check if the database is closed
    pub fn is_closed(&self) -> bool {
        *self.closed.lock().unwrap()
    }

    /// Close the database connection
    pub fn close(&self) {
        *self.closed.lock().unwrap() = true;
    }

    /// Get name property
    pub(crate) fn get_name(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        let db = obj.downcast_ref::<IDBDatabase>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        Ok(JsValue::from(JsString::from(db.name.clone())))
    }

    /// Get version property
    pub(crate) fn get_version(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        let db = obj.downcast_ref::<IDBDatabase>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        Ok(JsValue::from(db.version))
    }

    /// Get objectStoreNames property
    pub(crate) fn get_object_store_names(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        let db = obj.downcast_ref::<IDBDatabase>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        // Create DOMStringList-like object
        use boa_engine::builtins::array::Array;
        let array = Array::array_create(db.object_store_names.len() as u64, None, context)?;

        for (i, name) in db.object_store_names.iter().enumerate() {
            array.set(
                i,
                JsValue::from(JsString::from(name.clone())),
                true,
                context,
            )?;
        }

        Ok(array.into())
    }

    /// createObjectStore(name, options?)
    /// Creates a new object store (only in versionchange transaction)
    pub(crate) fn create_object_store(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        let db = obj.downcast_ref::<IDBDatabase>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        // Parse arguments
        let name = args.get_or_undefined(0).to_string(context)?;
        let name_str = name.to_std_string_escaped();

        // Check if already exists
        if db.object_store_names.contains(&name_str) {
            return Err(JsNativeError::error()
                .with_message(format!("Object store '{}' already exists", name_str))
                .into());
        }

        // Parse options
        let options = args.get_or_undefined(1);
        let mut key_path: Option<String> = None;
        let mut auto_increment = false;

        if let Some(opts_obj) = options.as_object() {
            // Get keyPath
            if let Ok(kp) = opts_obj.get(js_string!("keyPath"), context)
                && !kp.is_undefined()
                && !kp.is_null()
            {
                let kp_str = kp.to_string(context)?;
                key_path = Some(kp_str.to_std_string_escaped());
            }

            // Get autoIncrement
            if let Ok(ai) = opts_obj.get(js_string!("autoIncrement"), context) {
                auto_increment = ai.to_boolean();
            }
        }

        // Create object store in backend
        {
            let mut backend = db.backend.lock().unwrap();
            backend
                .create_object_store(&db.name, &name_str, key_path.clone(), auto_increment)
                .map_err(|e| JsNativeError::error().with_message(e))?;
        }

        // Create and return IDBObjectStore object
        let store = IDBObjectStore::new(
            name_str.clone(),
            key_path,
            auto_increment,
            db.backend.clone(),
            db.name.clone(),
        );

        let store_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            store,
        );

        Ok(store_obj.into())
    }

    /// deleteObjectStore(name)
    /// Deletes an object store (only in versionchange transaction)
    pub(crate) fn delete_object_store(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        let db = obj.downcast_ref::<IDBDatabase>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        let name = args.get_or_undefined(0).to_string(context)?;
        let name_str = name.to_std_string_escaped();

        // Check if exists
        if !db.object_store_names.contains(&name_str) {
            return Err(JsNativeError::error()
                .with_message(format!("Object store '{}' does not exist", name_str))
                .into());
        }

        // Delete from backend
        {
            let mut backend = db.backend.lock().unwrap();
            backend
                .delete_object_store(&db.name, &name_str)
                .map_err(|e| JsNativeError::error().with_message(e))?;
        }

        Ok(JsValue::undefined())
    }

    /// transaction(storeNames, mode?)
    /// Creates a new transaction
    pub(crate) fn transaction(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        let db = obj.downcast_ref::<IDBDatabase>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        // Check if database is closed
        if db.is_closed() {
            return Err(JsNativeError::error()
                .with_message("Database connection is closed")
                .into());
        }

        // Parse store names
        let store_names_arg = args.get_or_undefined(0);
        let mut store_names = Vec::new();

        if let Some(arr) = store_names_arg.as_object() {
            if arr.is_array() {
                let length = arr.get(js_string!("length"), context)?.to_u32(context)?;

                for i in 0..length {
                    let name = arr
                        .get(i, context)?
                        .to_string(context)?
                        .to_std_string_escaped();
                    store_names.push(name);
                }
            } else {
                // Single store name
                let name = store_names_arg.to_string(context)?.to_std_string_escaped();
                store_names.push(name);
            }
        } else {
            let name = store_names_arg.to_string(context)?.to_std_string_escaped();
            store_names.push(name);
        }

        // Validate store names exist
        for name in &store_names {
            if !db.object_store_names.contains(name) {
                return Err(JsNativeError::error()
                    .with_message(format!("Object store '{}' does not exist", name))
                    .into());
            }
        }

        // Parse mode
        let mode_arg = args.get_or_undefined(1);
        let mode_str = if mode_arg.is_undefined() {
            "readonly"
        } else {
            &mode_arg.to_string(context)?.to_std_string_escaped()
        };

        let mode = match mode_str {
            "readonly" => TransactionMode::ReadOnly,
            "readwrite" => TransactionMode::ReadWrite,
            "versionchange" => {
                return Err(JsNativeError::error()
                    .with_message("Cannot create versionchange transaction")
                    .into());
            }
            _ => {
                return Err(JsNativeError::typ()
                    .with_message(format!("Invalid transaction mode: {}", mode_str))
                    .into());
            }
        };

        // Create transaction
        let txn = IDBTransaction::new(db.backend.clone(), db.name.clone(), store_names, mode);

        let txn_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            txn,
        );

        Ok(txn_obj.into())
    }

    /// close()
    /// Closes the database connection
    pub(crate) fn close_method(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        let db = obj.downcast_ref::<IDBDatabase>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        db.close();
        Ok(JsValue::undefined())
    }

    /// Set onabort handler
    pub(crate) fn set_onabort(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        let db = obj.downcast_ref::<IDBDatabase>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        let handler = args.get_or_undefined(0);
        let mut onabort = db.onabort.lock().unwrap();

        if handler.is_callable() {
            *onabort = handler.as_object();
        } else {
            *onabort = None;
        }

        Ok(JsValue::undefined())
    }

    /// Set onerror handler
    pub(crate) fn set_onerror(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        let db = obj.downcast_ref::<IDBDatabase>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        let handler = args.get_or_undefined(0);
        let mut onerror = db.onerror.lock().unwrap();

        if handler.is_callable() {
            *onerror = handler.as_object();
        } else {
            *onerror = None;
        }

        Ok(JsValue::undefined())
    }

    /// Set onversionchange handler
    pub(crate) fn set_onversionchange(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        let db = obj.downcast_ref::<IDBDatabase>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBDatabase object")
        })?;

        let handler = args.get_or_undefined(0);
        let mut onversionchange = db.onversionchange.lock().unwrap();

        if handler.is_callable() {
            *onversionchange = handler.as_object();
        } else {
            *onversionchange = None;
        }

        Ok(JsValue::undefined())
    }
}

impl IntrinsicObject for IDBDatabase {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Properties
            .accessor(
                js_string!("name"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_name)
                        .name(js_string!("get name"))
                        .build(),
                ),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("version"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_version)
                        .name(js_string!("get version"))
                        .build(),
                ),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("objectStoreNames"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_object_store_names)
                        .name(js_string!("get objectStoreNames"))
                        .build(),
                ),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onabort"),
                None,
                Some(
                    BuiltInBuilder::callable(realm, Self::set_onabort)
                        .name(js_string!("set onabort"))
                        .build(),
                ),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onerror"),
                None,
                Some(
                    BuiltInBuilder::callable(realm, Self::set_onerror)
                        .name(js_string!("set onerror"))
                        .build(),
                ),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onversionchange"),
                None,
                Some(
                    BuiltInBuilder::callable(realm, Self::set_onversionchange)
                        .name(js_string!("set onversionchange"))
                        .build(),
                ),
                Attribute::CONFIGURABLE,
            )
            // Methods
            .method(
                Self::create_object_store,
                js_string!("createObjectStore"),
                1,
            )
            .method(
                Self::delete_object_store,
                js_string!("deleteObjectStore"),
                1,
            )
            .method(Self::transaction, js_string!("transaction"), 1)
            .method(Self::close_method, js_string!("close"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for IDBDatabase {
    const NAME: JsString = js_string!("IDBDatabase");
}

impl BuiltInConstructor for IDBDatabase {
    const PROTOTYPE_STORAGE_SLOTS: usize = 100; // Estimated prototype property count
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100; // Constructor properties

    const CONSTRUCTOR_ARGUMENTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(
        &boa_engine::context::intrinsics::StandardConstructors,
    ) -> &StandardConstructor = |intrinsics| intrinsics.idb_database();

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("IDBDatabase constructor cannot be called directly")
            .into())
    }
}

impl Default for IDBDatabase {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: 0,
            object_store_names: Vec::new(),
            backend: Arc::new(Mutex::new(Box::new(
                super::backend::memory::MemoryBackend::new(),
            ))),
            closed: Arc::new(Mutex::new(false)),
            onabort: Arc::new(Mutex::new(None)),
            onerror: Arc::new(Mutex::new(None)),
            onversionchange: Arc::new(Mutex::new(None)),
        }
    }
}
