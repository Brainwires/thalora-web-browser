//! IDBObjectStore Implementation
//!
//! Represents an object store in a database.
//!
//! Spec: https://w3c.github.io/IndexedDB/#object-store

use super::backend::StorageBackend;
use super::cursor::{CursorDirection, IDBCursor};
use super::key::IDBKey;
use super::key_range::IDBKeyRange;
use super::request::IDBRequest;
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

/// Helper: Serialize a JsValue to JSON string
fn json_stringify(value: &JsValue, context: &mut Context) -> JsResult<String> {
    // Get JSON global object
    let global = context.global_object();
    let json_obj = global.get(js_string!("JSON"), context)?;
    let json_obj = json_obj.as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("JSON is not an object"))?;

    // Call JSON.stringify
    let stringify_fn = json_obj.get(js_string!("stringify"), context)?;
    let stringify_fn = stringify_fn.as_callable()
        .ok_or_else(|| JsNativeError::typ().with_message("JSON.stringify is not callable"))?;

    let result = stringify_fn.call(&JsValue::undefined(), &[value.clone()], context)?;
    Ok(result.to_string(context)?.to_std_string_escaped())
}

/// Helper: Parse a JSON string to JsValue
fn json_parse(json_str: &str, context: &mut Context) -> JsResult<JsValue> {
    // Get JSON global object
    let global = context.global_object();
    let json_obj = global.get(js_string!("JSON"), context)?;
    let json_obj = json_obj.as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("JSON is not an object"))?;

    // Call JSON.parse
    let parse_fn = json_obj.get(js_string!("parse"), context)?;
    let parse_fn = parse_fn.as_callable()
        .ok_or_else(|| JsNativeError::typ().with_message("JSON.parse is not callable"))?;

    parse_fn.call(&JsValue::undefined(), &[JsValue::from(JsString::from(json_str))], context)
}

/// IDBObjectStore represents an object store
#[derive(Clone, Finalize)]
pub struct IDBObjectStore {
    name: String,
    key_path: Option<String>,
    auto_increment: bool,
    backend: Arc<Mutex<Box<dyn StorageBackend>>>,
    db_name: String,
}

impl std::fmt::Debug for IDBObjectStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IDBObjectStore")
            .field("name", &self.name)
            .field("key_path", &self.key_path)
            .field("auto_increment", &self.auto_increment)
            .finish()
    }
}

unsafe impl Trace for IDBObjectStore {
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

impl JsData for IDBObjectStore {}

impl IDBObjectStore {
    pub fn new(
        name: String,
        key_path: Option<String>,
        auto_increment: bool,
        backend: Arc<Mutex<Box<dyn StorageBackend>>>,
        db_name: String,
    ) -> Self {
        Self {
            name,
            key_path,
            auto_increment,
            backend,
            db_name,
        }
    }

    /// Get name property
    pub(crate) fn get_name(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        let store = obj.downcast_ref::<IDBObjectStore>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        Ok(JsValue::from(JsString::from(store.name.clone())))
    }

    /// Get keyPath property
    pub(crate) fn get_key_path(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        let store = obj.downcast_ref::<IDBObjectStore>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        if let Some(key_path) = &store.key_path {
            Ok(JsValue::from(JsString::from(key_path.clone())))
        } else {
            Ok(JsValue::null())
        }
    }

    /// Get autoIncrement property
    pub(crate) fn get_auto_increment(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        let store = obj.downcast_ref::<IDBObjectStore>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        Ok(JsValue::from(store.auto_increment))
    }

    /// add(value, key?)
    /// Adds a new record, fails if key exists
    pub(crate) fn add(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        let store = obj.downcast_ref::<IDBObjectStore>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        // Parse value
        let value = args.get_or_undefined(0);
        let value_json_str = json_stringify(value, context)?;
        let value_bytes = value_json_str.as_bytes();

        // Parse key (if provided)
        let key = if args.len() > 1 && !args[1].is_undefined() {
            IDBKey::from_js_value(&args[1], context)?
        } else if store.auto_increment {
            // Auto-increment will generate key
            IDBKey::Number(0.0) // Placeholder, backend will generate
        } else {
            return Err(JsNativeError::error()
                .with_message("Key is required when autoIncrement is false")
                .into());
        };

        // Create request
        let request = IDBRequest::new();

        // Add to backend
        let backend = store.backend.clone();
        let db_name = store.db_name.clone();
        let store_name = store.name.clone();

        let result_key = {
            let mut backend = backend.lock().unwrap();
            backend.add(&db_name, &store_name, &key, value_bytes)
                .map_err(|e| JsNativeError::error().with_message(e))?
        };

        // Set result
        let result_js = result_key.to_js_value(context)?;
        request.set_result(result_js);

        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            request,
        );

        Ok(request_obj.into())
    }

    /// put(value, key?)
    /// Adds or updates a record
    pub(crate) fn put(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        let store = obj.downcast_ref::<IDBObjectStore>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        // Parse value
        let value = args.get_or_undefined(0);
        let value_json_str = json_stringify(value, context)?;
        let value_bytes = value_json_str.as_bytes();

        // Parse key (if provided)
        let key = if args.len() > 1 && !args[1].is_undefined() {
            IDBKey::from_js_value(&args[1], context)?
        } else if store.auto_increment {
            IDBKey::Number(0.0) // Placeholder
        } else {
            return Err(JsNativeError::error()
                .with_message("Key is required when autoIncrement is false")
                .into());
        };

        // Create request
        let request = IDBRequest::new();

        // Put to backend
        let backend = store.backend.clone();
        let db_name = store.db_name.clone();
        let store_name = store.name.clone();

        let result_key = {
            let mut backend = backend.lock().unwrap();
            backend.put(&db_name, &store_name, &key, value_bytes)
                .map_err(|e| JsNativeError::error().with_message(e))?
        };

        // Set result
        let result_js = result_key.to_js_value(context)?;
        request.set_result(result_js);

        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            request,
        );

        Ok(request_obj.into())
    }

    /// get(key)
    /// Retrieves a record by key
    pub(crate) fn get(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        let store = obj.downcast_ref::<IDBObjectStore>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        // Parse key
        let key = IDBKey::from_js_value(args.get_or_undefined(0), context)?;

        // Create request
        let request = IDBRequest::new();

        // Get from backend
        let backend = store.backend.clone();
        let db_name = store.db_name.clone();
        let store_name = store.name.clone();

        let value_bytes = {
            let backend = backend.lock().unwrap();
            backend.get(&db_name, &store_name, &key)
                .map_err(|e| JsNativeError::error().with_message(e))?
        };

        // Parse value
        let result = if let Some(bytes) = value_bytes {
            let json_str = String::from_utf8(bytes)
                .map_err(|e| JsNativeError::error().with_message(e.to_string()))?;
            json_parse(&json_str, context)?
        } else {
            JsValue::undefined()
        };

        request.set_result(result);

        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            request,
        );

        Ok(request_obj.into())
    }

    /// delete(key)
    /// Deletes a record by key
    pub(crate) fn delete(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        let store = obj.downcast_ref::<IDBObjectStore>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        // Parse key
        let key = IDBKey::from_js_value(args.get_or_undefined(0), context)?;

        // Create request
        let request = IDBRequest::new();

        // Delete from backend
        let backend = store.backend.clone();
        let db_name = store.db_name.clone();
        let store_name = store.name.clone();

        {
            let mut backend = backend.lock().unwrap();
            backend.delete(&db_name, &store_name, &key)
                .map_err(|e| JsNativeError::error().with_message(e))?;
        }

        request.set_result(JsValue::undefined());

        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            request,
        );

        Ok(request_obj.into())
    }

    /// clear()
    /// Clears all records in the store
    pub(crate) fn clear(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        let store = obj.downcast_ref::<IDBObjectStore>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        // Create request
        let request = IDBRequest::new();

        // Clear backend
        let backend = store.backend.clone();
        let db_name = store.db_name.clone();
        let store_name = store.name.clone();

        {
            let mut backend = backend.lock().unwrap();
            backend.clear(&db_name, &store_name)
                .map_err(|e| JsNativeError::error().with_message(e))?;
        }

        request.set_result(JsValue::undefined());

        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            request,
        );

        Ok(request_obj.into())
    }

    /// count(key?)
    /// Counts records matching key or all records
    pub(crate) fn count(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        let store = obj.downcast_ref::<IDBObjectStore>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        // Parse key range (if provided)
        let range = if args.len() > 0 && !args[0].is_undefined() {
            // Try to parse as IDBKeyRange
            if let Some(range_obj) = args[0].as_object() {
                if let Some(range) = range_obj.downcast_ref::<IDBKeyRange>() {
                    Some(range.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Create request
        let request = IDBRequest::new();

        // Count in backend
        let backend = store.backend.clone();
        let db_name = store.db_name.clone();
        let store_name = store.name.clone();

        let count = {
            let backend = backend.lock().unwrap();
            backend.count(&db_name, &store_name, range.as_ref())
                .map_err(|e| JsNativeError::error().with_message(e))?
        };

        request.set_result(JsValue::from(count));

        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            request,
        );

        Ok(request_obj.into())
    }

    /// getAll(query?, count?)
    /// Gets all values matching query
    pub(crate) fn get_all(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        let store = obj.downcast_ref::<IDBObjectStore>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        // Parse range and count
        let range = if args.len() > 0 && !args[0].is_undefined() {
            if let Some(range_obj) = args[0].as_object() {
                range_obj.downcast_ref::<IDBKeyRange>().map(|r| (*r).clone())
            } else {
                None
            }
        } else {
            None
        };

        let count = if args.len() > 1 && !args[1].is_undefined() {
            Some(args[1].to_u32(context)?)
        } else {
            None
        };

        // Create request
        let request = IDBRequest::new();

        // Get all from backend
        let backend = store.backend.clone();
        let db_name = store.db_name.clone();
        let store_name = store.name.clone();

        let values = {
            let backend = backend.lock().unwrap();
            backend.get_all(&db_name, &store_name, range.as_ref(), count)
                .map_err(|e| JsNativeError::error().with_message(e))?
        };

        // Parse values to JS array
        use boa_engine::builtins::array::Array;
        let array = Array::array_create(values.len() as u64, None, context)?;

        for (i, bytes) in values.iter().enumerate() {
            let json_str = String::from_utf8(bytes.clone())
                .map_err(|e| JsNativeError::error().with_message(e.to_string()))?;
            let value = json_parse(&json_str, context)?;
            array.set(i, value, true, context)?;
        }

        request.set_result(array.into());

        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            request,
        );

        Ok(request_obj.into())
    }

    /// store.openCursor(range?, direction?)
    /// Opens a cursor for iterating over records
    pub(crate) fn open_cursor(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        let store = obj.downcast_ref::<IDBObjectStore>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBObjectStore object"))?;

        // Parse optional range
        let range = if args.len() > 0 && !args[0].is_undefined() {
            // Try to parse as key or key range
            if args[0].is_object() {
                // Check if it's an IDBKeyRange
                if let Some(range_obj) = args[0].as_object() {
                    if let Some(range) = range_obj.downcast_ref::<IDBKeyRange>() {
                        Some((*range).clone())
                    } else {
                        // Try to parse as key and create only range
                        let key = IDBKey::from_js_value(&args[0], context)?;
                        // Create an "only" range (single value)
                        Some(IDBKeyRange::new(Some(key.clone()), Some(key), false, false)
                            .map_err(|e| JsNativeError::error().with_message(e))?)
                    }
                } else {
                    None
                }
            } else {
                // Parse as key and create only range
                let key = IDBKey::from_js_value(&args[0], context)?;
                // Create an "only" range (single value)
                Some(IDBKeyRange::new(Some(key.clone()), Some(key), false, false)
                    .map_err(|e| JsNativeError::error().with_message(e))?)
            }
        } else {
            None
        };

        // Parse optional direction
        let direction = if args.len() > 1 && !args[1].is_undefined() {
            let dir_str = args[1].to_string(context)?;
            let dir_string = dir_str.to_std_string_escaped();
            CursorDirection::from_string(&dir_string)
                .map_err(|e| JsNativeError::typ().with_message(e))?
        } else {
            CursorDirection::Next
        };

        // Create cursor
        let cursor = IDBCursor::new(
            store.backend.clone(),
            store.db_name.clone(),
            store.name.clone(),
            range,
            direction,
        ).map_err(|e| JsNativeError::error().with_message(e))?;

        // Create cursor object
        let cursor_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            cursor,
        );

        // Create request
        let request = IDBRequest::new();
        request.set_result(cursor_obj.into());

        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            request,
        );

        Ok(request_obj.into())
    }
}

impl IntrinsicObject for IDBObjectStore {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Properties
            .accessor(
                js_string!("name"),
                Some(BuiltInBuilder::callable(realm, Self::get_name)
                    .name(js_string!("get name"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("keyPath"),
                Some(BuiltInBuilder::callable(realm, Self::get_key_path)
                    .name(js_string!("get keyPath"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("autoIncrement"),
                Some(BuiltInBuilder::callable(realm, Self::get_auto_increment)
                    .name(js_string!("get autoIncrement"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            // Methods
            .method(Self::add, js_string!("add"), 1)
            .method(Self::put, js_string!("put"), 1)
            .method(Self::get, js_string!("get"), 1)
            .method(Self::delete, js_string!("delete"), 1)
            .method(Self::clear, js_string!("clear"), 0)
            .method(Self::count, js_string!("count"), 0)
            .method(Self::get_all, js_string!("getAll"), 0)
            .method(Self::open_cursor, js_string!("openCursor"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for IDBObjectStore {
    const NAME: JsString = js_string!("IDBObjectStore");
}

impl BuiltInConstructor for IDBObjectStore {
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;  // Estimated prototype property count
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;  // Constructor properties

    const CONSTRUCTOR_ARGUMENTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&boa_engine::context::intrinsics::StandardConstructors) -> &StandardConstructor =
        |intrinsics| intrinsics.idb_object_store();

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("IDBObjectStore constructor cannot be called directly")
            .into())
    }
}

impl Default for IDBObjectStore {
    fn default() -> Self {
        Self {
            name: String::new(),
            key_path: None,
            auto_increment: false,
            backend: Arc::new(Mutex::new(Box::new(super::backend::memory::MemoryBackend::new()))),
            db_name: String::new(),
        }
    }
}
