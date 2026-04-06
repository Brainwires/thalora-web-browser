//! Implementation of the `LockManager` Web API.

use boa_engine::builtins::{BuiltInConstructor, BuiltInObject, IntrinsicObject};
use boa_engine::context::intrinsics::StandardConstructor;
use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, JsValue,
    builtins::BuiltInBuilder,
    context::intrinsics::Intrinsics,
    js_string,
    object::{JsObject, JsPromise},
    realm::Realm,
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::lock::{Lock, LockMode};
use super::lock_info::{LockInfo, LockManagerSnapshot};

/// `LockManager` implementation for the Web Locks API.
#[derive(Debug, Clone, Finalize)]
pub struct LockManager {
    /// Track held locks by resource name
    held_locks: Arc<RwLock<HashMap<String, Vec<LockInfo>>>>,
}

unsafe impl Trace for LockManager {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // No GC'd objects, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC'd objects, nothing to trace
    }

    fn run_finalizer(&self) {
        // No cleanup needed
    }
}

impl JsData for LockManager {}

impl LockManager {
    /// Creates a new `LockManager` instance.
    pub fn new() -> Self {
        Self {
            held_locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for LockManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LockManager {
    /// `navigator.locks.request(name, callback)`
    /// `navigator.locks.request(name, options, callback)`
    fn request(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("LockManager.prototype.request called on non-LockManager object")
        })?;

        let lock_manager = obj.downcast_ref::<LockManager>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("LockManager.prototype.request called on non-LockManager object")
        })?;

        // Parse arguments - support both 2-arg and 3-arg forms
        let (name, _options, callback) = match args.len() {
            0 | 1 => {
                return Err(JsNativeError::typ()
                    .with_message("LockManager.request requires at least 2 arguments")
                    .into());
            }
            2 => {
                // request(name, callback)
                let name = args.get_or_undefined(0).to_string(context)?;
                let callback = args.get_or_undefined(1);
                (name, None, callback)
            }
            _ => {
                // request(name, options, callback)
                let name = args.get_or_undefined(0).to_string(context)?;
                let options = args.get_or_undefined(1);
                let callback = args.get_or_undefined(2);
                (name, Some(options), callback)
            }
        };

        // Extract mode from options (default to exclusive)
        let mode = if let Some(options) = _options {
            if let Some(options_obj) = options.as_object() {
                let mode_val = options_obj.get(js_string!("mode"), context)?;
                if let Some(mode_str) = mode_val.as_string() {
                    LockMode::from_str(&mode_str.to_std_string_escaped())
                } else {
                    LockMode::Exclusive
                }
            } else {
                LockMode::Exclusive
            }
        } else {
            LockMode::Exclusive
        };

        // Create lock info
        let lock_info = LockInfo::new(name.to_std_string_escaped(), mode);

        // Add to held locks
        {
            let mut held = lock_manager.held_locks.write().unwrap();
            held.entry(lock_info.name.clone())
                .or_insert_with(Vec::new)
                .push(lock_info.clone());
        }

        // Execute callback immediately (simplified - no actual queueing)
        let callback_result = if callback.is_callable() {
            // Create a Lock object to pass to callback
            let lock = Lock::new(lock_info.name.clone(), mode);
            let lock_proto = context.intrinsics().constructors().object().prototype();
            let lock_obj = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                lock_proto,
                lock,
            );

            // Call the callback with the lock
            callback
                .as_callable()
                .ok_or_else(|| JsNativeError::typ().with_message("callback is not callable"))?
                .call(&JsValue::undefined(), &[lock_obj.into()], context)?
        } else {
            return Err(JsNativeError::typ()
                .with_message("callback must be a function")
                .into());
        };

        // Remove from held locks after callback completes
        {
            let mut held = lock_manager.held_locks.write().unwrap();
            if let Some(locks) = held.get_mut(&lock_info.name) {
                locks.retain(|l| l.client_id != lock_info.client_id);
                if locks.is_empty() {
                    held.remove(&lock_info.name);
                }
            }
        }

        // Return a Promise that resolves with the callback result
        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers
            .resolve
            .call(&JsValue::undefined(), &[callback_result], context)?;
        Ok(JsValue::from(promise))
    }

    /// `navigator.locks.query()`
    fn query(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("LockManager.prototype.query called on non-LockManager object")
        })?;

        let lock_manager = obj.downcast_ref::<LockManager>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("LockManager.prototype.query called on non-LockManager object")
        })?;

        // Get current held locks
        let held = {
            let held_map = lock_manager.held_locks.read().unwrap();
            held_map
                .values()
                .flat_map(|v| v.iter())
                .cloned()
                .collect::<Vec<_>>()
        };

        // Create snapshot (no pending locks in simplified implementation)
        let snapshot = LockManagerSnapshot::new(held, Vec::new());
        let snapshot_obj = snapshot.to_js_object(context)?;

        // Return a Promise that resolves with the snapshot
        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers
            .resolve
            .call(&JsValue::undefined(), &[snapshot_obj], context)?;
        Ok(JsValue::from(promise))
    }
}

impl IntrinsicObject for LockManager {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::request, js_string!("request"), 2)
            .method(Self::query, js_string!("query"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for LockManager {
    const NAME: JsString = js_string!("LockManager");
}

impl BuiltInConstructor for LockManager {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(
        &boa_engine::context::intrinsics::StandardConstructors,
    ) -> &StandardConstructor = |constructors| constructors.lock_manager();

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // LockManager constructor cannot be called directly
        Err(JsNativeError::typ()
            .with_message("LockManager constructor cannot be called directly")
            .into())
    }
}
