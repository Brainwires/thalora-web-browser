//! IDBCursor Implementation
//!
//! Iterates over object stores and indexes.
//!
//! Spec: https://w3c.github.io/IndexedDB/#cursor-interface

use super::backend::StorageBackend;
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

/// Cursor direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorDirection {
    Next,
    NextUnique,
    Prev,
    PrevUnique,
}

impl CursorDirection {
    pub fn from_string(s: &str) -> Result<Self, String> {
        match s {
            "next" => Ok(CursorDirection::Next),
            "nextunique" => Ok(CursorDirection::NextUnique),
            "prev" => Ok(CursorDirection::Prev),
            "prevunique" => Ok(CursorDirection::PrevUnique),
            _ => Err(format!("Invalid cursor direction: {}", s)),
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            CursorDirection::Next => "next",
            CursorDirection::NextUnique => "nextunique",
            CursorDirection::Prev => "prev",
            CursorDirection::PrevUnique => "prevunique",
        }
    }
}

/// Internal cursor state
struct CursorState {
    keys: Vec<IDBKey>,
    values: Vec<Vec<u8>>,
    current_index: usize,
    got_value: bool,
}

/// IDBCursor iterates over records
#[derive(Clone, Finalize)]
pub struct IDBCursor {
    direction: CursorDirection,
    backend: Arc<Mutex<Box<dyn StorageBackend>>>,
    db_name: String,
    store_name: String,
    range: Option<IDBKeyRange>,
    state: Arc<Mutex<CursorState>>,
}

impl std::fmt::Debug for IDBCursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IDBCursor")
            .field("direction", &self.direction)
            .field("db_name", &self.db_name)
            .field("store_name", &self.store_name)
            .finish()
    }
}

unsafe impl Trace for IDBCursor {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // Backend doesn't contain GC'd objects
    }

    unsafe fn trace_non_roots(&self) {}

    fn run_finalizer(&self) {}
}

impl JsData for IDBCursor {}

impl IDBCursor {
    /// Create a new cursor
    pub fn new(
        backend: Arc<Mutex<Box<dyn StorageBackend>>>,
        db_name: String,
        store_name: String,
        range: Option<IDBKeyRange>,
        direction: CursorDirection,
    ) -> Result<Self, String> {
        // Fetch initial data
        let (keys, values) = {
            let backend = backend.lock().unwrap();
            let keys = backend.get_all_keys(&db_name, &store_name, range.as_ref(), None)?;
            let values = backend.get_all(&db_name, &store_name, range.as_ref(), None)?;
            (keys, values)
        };

        // Reverse for prev directions
        let (keys, values) = match direction {
            CursorDirection::Prev | CursorDirection::PrevUnique => {
                let mut keys = keys;
                let mut values = values;
                keys.reverse();
                values.reverse();
                (keys, values)
            }
            _ => (keys, values),
        };

        Ok(Self {
            direction,
            backend,
            db_name,
            store_name,
            range,
            state: Arc::new(Mutex::new(CursorState {
                keys,
                values,
                current_index: 0,
                got_value: false,
            })),
        })
    }

    /// Get current key
    pub fn key(&self) -> Option<IDBKey> {
        let state = self.state.lock().unwrap();
        if state.current_index < state.keys.len() {
            Some(state.keys[state.current_index].clone())
        } else {
            None
        }
    }

    /// Get primary key (same as key for object stores)
    pub fn primary_key(&self) -> Option<IDBKey> {
        self.key()
    }

    /// Get current value
    pub fn value(&self) -> Option<Vec<u8>> {
        let state = self.state.lock().unwrap();
        if state.current_index < state.values.len() {
            Some(state.values[state.current_index].clone())
        } else {
            None
        }
    }

    /// cursor.direction getter
    pub(crate) fn get_direction(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        let cursor = obj.downcast_ref::<IDBCursor>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        Ok(JsValue::from(JsString::from(cursor.direction.to_string())))
    }

    /// cursor.key getter
    pub(crate) fn get_key(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        let cursor = obj.downcast_ref::<IDBCursor>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        match cursor.key() {
            Some(key) => key.to_js_value(context),
            None => Ok(JsValue::undefined()),
        }
    }

    /// cursor.primaryKey getter
    pub(crate) fn get_primary_key(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        let cursor = obj.downcast_ref::<IDBCursor>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        match cursor.primary_key() {
            Some(key) => key.to_js_value(context),
            None => Ok(JsValue::undefined()),
        }
    }

    /// cursor.value getter
    pub(crate) fn get_value(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        let cursor = obj.downcast_ref::<IDBCursor>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        match cursor.value() {
            Some(value_bytes) => {
                // Deserialize from JSON
                let value_str = String::from_utf8_lossy(&value_bytes);
                json_parse(&value_str, context)
            }
            None => Ok(JsValue::undefined()),
        }
    }

    /// cursor.advance(count)
    /// Advances the cursor by count records
    pub(crate) fn advance(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        let cursor = obj.downcast_ref::<IDBCursor>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        let count = args.get_or_undefined(0)
            .to_u32(_context)
            .unwrap_or(1) as usize;

        if count == 0 {
            return Err(JsNativeError::typ()
                .with_message("Count must be greater than zero")
                .into());
        }

        let mut state = cursor.state.lock().unwrap();

        // Check if already got value
        if state.got_value {
            return Err(JsNativeError::error()
                .with_message("The cursor is being iterated or has iterated past its end")
                .into());
        }

        // Advance index
        match cursor.direction {
            CursorDirection::NextUnique | CursorDirection::PrevUnique => {
                // Skip duplicate keys
                if state.current_index < state.keys.len() {
                    let current_key = state.keys[state.current_index].clone();
                    let mut advanced = 0;

                    while state.current_index < state.keys.len() && advanced < count {
                        state.current_index += 1;
                        if state.current_index >= state.keys.len()
                            || state.keys[state.current_index] != current_key {
                            advanced += 1;
                            if advanced < count && state.current_index < state.keys.len() {
                                // Update current key for next iteration
                            }
                        }
                    }
                }
            }
            _ => {
                state.current_index += count;
            }
        }

        state.got_value = true;

        Ok(JsValue::undefined())
    }

    /// cursor.continue(key?)
    /// Continues iteration to the next record
    pub(crate) fn continue_method(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        let cursor = obj.downcast_ref::<IDBCursor>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        let mut state = cursor.state.lock().unwrap();

        // Check if already got value
        if state.got_value {
            return Err(JsNativeError::error()
                .with_message("The cursor is being iterated or has iterated past its end")
                .into());
        }

        // Parse optional key
        let target_key = if args.len() > 0 && !args[0].is_undefined() {
            Some(IDBKey::from_js_value(&args[0], context)?)
        } else {
            None
        };

        // Move to next record
        if let Some(key) = target_key {
            // Continue to specific key
            match cursor.direction {
                CursorDirection::Next | CursorDirection::NextUnique => {
                    // Find first key >= target
                    while state.current_index < state.keys.len()
                        && state.keys[state.current_index] < key {
                        state.current_index += 1;
                    }
                }
                CursorDirection::Prev | CursorDirection::PrevUnique => {
                    // Find first key <= target
                    while state.current_index < state.keys.len()
                        && state.keys[state.current_index] > key {
                        state.current_index += 1;
                    }
                }
            }
        } else {
            // Continue to next
            state.current_index += 1;

            // Skip duplicates for unique directions
            match cursor.direction {
                CursorDirection::NextUnique | CursorDirection::PrevUnique => {
                    if state.current_index > 0 && state.current_index < state.keys.len() {
                        let prev_key = state.keys[state.current_index - 1].clone();
                        while state.current_index < state.keys.len()
                            && state.keys[state.current_index] == prev_key {
                            state.current_index += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        state.got_value = true;

        Ok(JsValue::undefined())
    }

    /// cursor.continuePrimaryKey(key, primaryKey)
    /// Continues to a specific key and primary key
    pub(crate) fn continue_primary_key(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        let cursor = obj.downcast_ref::<IDBCursor>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        // For object stores, primary key is the same as key
        // So this just calls continue with the key
        let key_arg = args.get_or_undefined(0);
        Self::continue_method(this, &[key_arg.clone()], context)
    }

    /// cursor.update(value)
    /// Updates the current record
    pub(crate) fn update(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        let cursor = obj.downcast_ref::<IDBCursor>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        let state = cursor.state.lock().unwrap();

        // Check if cursor is valid
        if state.current_index >= state.keys.len() {
            return Err(JsNativeError::error()
                .with_message("Cursor is at end")
                .into());
        }

        if state.got_value {
            return Err(JsNativeError::error()
                .with_message("The cursor is being iterated")
                .into());
        }

        let current_key = state.keys[state.current_index].clone();
        drop(state);

        // Serialize value
        let value = args.get_or_undefined(0);
        let value_json_str = json_stringify(value, context)?;
        let value_bytes = value_json_str.as_bytes();

        // Create request
        let request = IDBRequest::new();

        // Update in backend
        let result = {
            let mut backend = cursor.backend.lock().unwrap();
            backend.put(&cursor.db_name, &cursor.store_name, &current_key, value_bytes)
        };

        match result {
            Ok(key) => {
                let key_js = key.to_js_value(context)?;
                request.set_result(key_js);
                request.trigger_success(context)?;
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

    /// cursor.delete()
    /// Deletes the current record
    pub(crate) fn delete_method(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        let cursor = obj.downcast_ref::<IDBCursor>()
            .ok_or_else(|| JsNativeError::typ()
                .with_message("'this' is not an IDBCursor object"))?;

        let state = cursor.state.lock().unwrap();

        // Check if cursor is valid
        if state.current_index >= state.keys.len() {
            return Err(JsNativeError::error()
                .with_message("Cursor is at end")
                .into());
        }

        if state.got_value {
            return Err(JsNativeError::error()
                .with_message("The cursor is being iterated")
                .into());
        }

        let current_key = state.keys[state.current_index].clone();
        drop(state);

        // Create request
        let request = IDBRequest::new();

        // Delete from backend
        let result = {
            let mut backend = cursor.backend.lock().unwrap();
            backend.delete(&cursor.db_name, &cursor.store_name, &current_key)
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

        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            request,
        );

        Ok(request_obj.into())
    }
}

impl IntrinsicObject for IDBCursor {
    fn init(realm: &Realm) {
        let _intrinsics = realm.intrinsics();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Properties
            .accessor(
                js_string!("direction"),
                Some(BuiltInBuilder::callable(realm, Self::get_direction)
                    .name(js_string!("get direction"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("key"),
                Some(BuiltInBuilder::callable(realm, Self::get_key)
                    .name(js_string!("get key"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("primaryKey"),
                Some(BuiltInBuilder::callable(realm, Self::get_primary_key)
                    .name(js_string!("get primaryKey"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("value"),
                Some(BuiltInBuilder::callable(realm, Self::get_value)
                    .name(js_string!("get value"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            // Methods
            .method(Self::advance, js_string!("advance"), 1)
            .method(Self::continue_method, js_string!("continue"), 0)
            .method(Self::continue_primary_key, js_string!("continuePrimaryKey"), 2)
            .method(Self::update, js_string!("update"), 1)
            .method(Self::delete_method, js_string!("delete"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for IDBCursor {
    const NAME: JsString = js_string!("IDBCursor");
}

impl BuiltInConstructor for IDBCursor {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;  // Estimated prototype property count
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100; // Constructor properties

    const STANDARD_CONSTRUCTOR: fn(&boa_engine::context::intrinsics::StandardConstructors) -> &StandardConstructor =
        |constructors| {
            constructors.idb_cursor()
        };

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // IDBCursor is not directly constructible
        Err(JsNativeError::typ()
            .with_message("Illegal constructor")
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_direction() {
        let next = CursorDirection::from_string("next").unwrap();
        assert_eq!(next, CursorDirection::Next);
        assert_eq!(next.to_string(), "next");

        let prev = CursorDirection::from_string("prev").unwrap();
        assert_eq!(prev, CursorDirection::Prev);
    }
}
