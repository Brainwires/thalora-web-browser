//! IDBIndex Implementation
//!
//! Provides access to a database index.
//!
//! Spec: https://w3c.github.io/IndexedDB/#index-interface

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

/// Helper: Parse a JSON string to JsValue
fn json_parse(json_str: &str, context: &mut Context) -> JsResult<JsValue> {
    let global = context.global_object();
    let json_obj = global.get(js_string!("JSON"), context)?;
    let json_obj = json_obj.as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("JSON is not an object"))?;

    let parse_fn = json_obj.get(js_string!("parse"), context)?;
    let parse_fn = parse_fn.as_callable()
        .ok_or_else(|| JsNativeError::typ().with_message("JSON.parse is not callable"))?;

    parse_fn.call(&JsValue::undefined(), &[JsValue::from(JsString::from(json_str))], context)
}

/// IDBIndex represents a database index
#[derive(Clone, Finalize)]
pub struct IDBIndex {
    name: String,
    key_path: String,
    unique: bool,
    multi_entry: bool,
    backend: Arc<Mutex<Box<dyn StorageBackend>>>,
    db_name: String,
    store_name: String,
}

impl std::fmt::Debug for IDBIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IDBIndex")
            .field("name", &self.name)
            .field("key_path", &self.key_path)
            .field("unique", &self.unique)
            .field("multi_entry", &self.multi_entry)
            .finish()
    }
}

unsafe impl Trace for IDBIndex {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // Backend doesn't contain GC'd objects
    }

    unsafe fn trace_non_roots(&self) {}

    fn run_finalizer(&self) {}
}

impl JsData for IDBIndex {}

impl IDBIndex {
    /// Create a new index
    pub fn new(
        name: String,
        key_path: String,
        unique: bool,
        multi_entry: bool,
        backend: Arc<Mutex<Box<dyn StorageBackend>>>,
        db_name: String,
        store_name: String,
    ) -> Self {
        Self {
            name,
            key_path,
            unique,
            multi_entry,
            backend,
            db_name,
            store_name,
        }
    }

    /// index.name getter
    pub(crate) fn get_name(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        let index = obj.downcast_ref::<IDBIndex>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        Ok(JsValue::from(JsString::from(index.name.clone())))
    }

    /// index.keyPath getter
    pub(crate) fn get_key_path(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        let index = obj.downcast_ref::<IDBIndex>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        Ok(JsValue::from(JsString::from(index.key_path.clone())))
    }

    /// index.unique getter
    pub(crate) fn get_unique(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        let index = obj.downcast_ref::<IDBIndex>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        Ok(JsValue::from(index.unique))
    }

    /// index.multiEntry getter
    pub(crate) fn get_multi_entry(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        let index = obj.downcast_ref::<IDBIndex>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        Ok(JsValue::from(index.multi_entry))
    }

    /// index.get(key)
    /// Retrieves a record by index key
    pub(crate) fn get(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        let index = obj.downcast_ref::<IDBIndex>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        // Parse key or range
        let query = args.get_or_undefined(0);
        let key = IDBKey::from_js_value(query, context)?;

        // Create request
        let request = IDBRequest::new();

        // Get from backend via index
        let result = {
            let backend = index.backend.lock().unwrap();
            backend.get_by_index(
                &index.db_name,
                &index.store_name,
                &index.name,
                &key
            )
        };

        match result {
            Ok(Some(bytes)) => {
                // Deserialize value
                let json_str = String::from_utf8(bytes)
                    .map_err(|e| JsNativeError::error().with_message(e.to_string()))?;
                let value = json_parse(&json_str, context)?;
                request.set_result(value);
            }
            Ok(None) => {
                request.set_result(JsValue::undefined());
            }
            Err(e) => {
                request.set_error(e);
                request.trigger_error(context)?;
            }
        }

        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            request,
        );

        Ok(request_obj.into())
    }

    /// index.getKey(key)
    /// Retrieves the primary key for an index key
    pub(crate) fn get_key(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        let index = obj.downcast_ref::<IDBIndex>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        // Parse key
        let query = args.get_or_undefined(0);
        let key = IDBKey::from_js_value(query, context)?;

        // Create request
        let request = IDBRequest::new();

        // Get key from index
        let result = {
            let backend = index.backend.lock().unwrap();
            backend.get_key_from_index(
                &index.db_name,
                &index.store_name,
                &index.name,
                &key
            )
        };

        match result {
            Ok(Some(primary_key)) => {
                let key_js = primary_key.to_js_value(context)?;
                request.set_result(key_js);
            }
            Ok(None) => {
                request.set_result(JsValue::undefined());
            }
            Err(e) => {
                request.set_error(e);
                request.trigger_error(context)?;
            }
        }

        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            request,
        );

        Ok(request_obj.into())
    }

    /// index.getAll(query?, count?)
    /// Retrieves all matching records
    pub(crate) fn get_all(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        let index = obj.downcast_ref::<IDBIndex>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        // Parse optional range
        let range = if args.len() > 0 && !args[0].is_undefined() {
            if let Some(range_obj) = args[0].as_object() {
                range_obj.downcast_ref::<IDBKeyRange>().map(|r| (*r).clone())
            } else {
                None
            }
        } else {
            None
        };

        // Parse optional count
        let count = if args.len() > 1 && !args[1].is_undefined() {
            Some(args[1].to_u32(context)?)
        } else {
            None
        };

        // Create request
        let request = IDBRequest::new();

        // Get all from index
        let values = {
            let backend = index.backend.lock().unwrap();
            backend.get_all_from_index(
                &index.db_name,
                &index.store_name,
                &index.name,
                range.as_ref(),
                count
            ).map_err(|e| JsNativeError::error().with_message(e))?
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

    /// index.getAllKeys(query?, count?)
    /// Retrieves all matching primary keys
    pub(crate) fn get_all_keys(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        let index = obj.downcast_ref::<IDBIndex>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        // Parse optional range
        let range = if args.len() > 0 && !args[0].is_undefined() {
            if let Some(range_obj) = args[0].as_object() {
                range_obj.downcast_ref::<IDBKeyRange>().map(|r| (*r).clone())
            } else {
                None
            }
        } else {
            None
        };

        // Parse optional count
        let count = if args.len() > 1 && !args[1].is_undefined() {
            Some(args[1].to_u32(context)?)
        } else {
            None
        };

        // Create request
        let request = IDBRequest::new();

        // Get all keys from index
        let keys = {
            let backend = index.backend.lock().unwrap();
            backend.get_all_keys_from_index(
                &index.db_name,
                &index.store_name,
                &index.name,
                range.as_ref(),
                count
            ).map_err(|e| JsNativeError::error().with_message(e))?
        };

        // Convert keys to JS array
        use boa_engine::builtins::array::Array;
        let array = Array::array_create(keys.len() as u64, None, context)?;

        for (i, key) in keys.iter().enumerate() {
            let key_js = key.to_js_value(context)?;
            array.set(i, key_js, true, context)?;
        }

        request.set_result(array.into());

        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            request,
        );

        Ok(request_obj.into())
    }

    /// index.count(query?)
    /// Counts matching records
    pub(crate) fn count(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        let index = obj.downcast_ref::<IDBIndex>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        // Parse optional range
        let range = if args.len() > 0 && !args[0].is_undefined() {
            if let Some(range_obj) = args[0].as_object() {
                range_obj.downcast_ref::<IDBKeyRange>().map(|r| (*r).clone())
            } else {
                None
            }
        } else {
            None
        };

        // Create request
        let request = IDBRequest::new();

        // Count in index
        let count = {
            let backend = index.backend.lock().unwrap();
            backend.count_index(
                &index.db_name,
                &index.store_name,
                &index.name,
                range.as_ref()
            ).map_err(|e| JsNativeError::error().with_message(e))?
        };

        request.set_result(JsValue::from(count));

        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            request,
        );

        Ok(request_obj.into())
    }

    /// index.openCursor(range?, direction?)
    /// Opens a cursor for iterating over index records
    pub(crate) fn open_cursor(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        let index = obj.downcast_ref::<IDBIndex>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBIndex object"))?;

        // Parse optional range
        let range = if args.len() > 0 && !args[0].is_undefined() {
            if args[0].is_object() {
                if let Some(range_obj) = args[0].as_object() {
                    if let Some(range) = range_obj.downcast_ref::<IDBKeyRange>() {
                        Some((*range).clone())
                    } else {
                        let key = IDBKey::from_js_value(&args[0], context)?;
                        Some(IDBKeyRange::new(Some(key.clone()), Some(key), false, false)
                            .map_err(|e| JsNativeError::error().with_message(e))?)
                    }
                } else {
                    None
                }
            } else {
                let key = IDBKey::from_js_value(&args[0], context)?;
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

        // Create cursor over index
        // Note: This uses the object store name, but backend should handle index queries
        let cursor = IDBCursor::new(
            index.backend.clone(),
            index.db_name.clone(),
            index.store_name.clone(),
            range,
            direction,
        ).map_err(|e| JsNativeError::error().with_message(e))?;

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

    /// index.openKeyCursor(range?, direction?)
    /// Opens a cursor for iterating over index keys only
    pub(crate) fn open_key_cursor(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // For now, implement same as openCursor
        // In a full implementation, this would return a cursor without values
        Self::open_cursor(this, args, context)
    }
}

impl IntrinsicObject for IDBIndex {
    fn init(realm: &Realm) {
        let _intrinsics = realm.intrinsics();

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
                js_string!("unique"),
                Some(BuiltInBuilder::callable(realm, Self::get_unique)
                    .name(js_string!("get unique"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("multiEntry"),
                Some(BuiltInBuilder::callable(realm, Self::get_multi_entry)
                    .name(js_string!("get multiEntry"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            // Methods
            .method(Self::get, js_string!("get"), 1)
            .method(Self::get_key, js_string!("getKey"), 1)
            .method(Self::get_all, js_string!("getAll"), 0)
            .method(Self::get_all_keys, js_string!("getAllKeys"), 0)
            .method(Self::count, js_string!("count"), 0)
            .method(Self::open_cursor, js_string!("openCursor"), 0)
            .method(Self::open_key_cursor, js_string!("openKeyCursor"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for IDBIndex {
    const NAME: JsString = js_string!("IDBIndex");
}

impl BuiltInConstructor for IDBIndex {
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;  // Estimated prototype property count
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;  // Constructor properties

    const CONSTRUCTOR_ARGUMENTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&boa_engine::context::intrinsics::StandardConstructors) -> &StandardConstructor =
        |constructors| {
            constructors.idb_index()  // Use Object constructor as placeholder
        };

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // IDBIndex is not directly constructible
        Err(JsNativeError::typ()
            .with_message("Illegal constructor")
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_creation() {
        use super::super::backend::memory::MemoryBackend;

        let backend = Arc::new(Mutex::new(Box::new(MemoryBackend::new()) as Box<dyn StorageBackend>));
        let index = IDBIndex::new(
            "email_idx".to_string(),
            "email".to_string(),
            true,
            false,
            backend,
            "test_db".to_string(),
            "users".to_string(),
        );

        assert_eq!(index.name, "email_idx");
        assert_eq!(index.key_path, "email");
        assert_eq!(index.unique, true);
        assert_eq!(index.multi_entry, false);
    }
}
