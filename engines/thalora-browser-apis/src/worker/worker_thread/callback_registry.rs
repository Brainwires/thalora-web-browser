//! Callback registry for storing JavaScript functions
//!
//! This module provides a thread-local storage for JavaScript callbacks
//! that can be referenced by ID from the event loop.

use boa_engine::{Context, JsResult, JsValue, JsObject, JsNativeError};
use std::collections::HashMap;
use std::cell::RefCell;

thread_local! {
    /// Thread-local callback storage
    /// Maps callback ID -> (callback_function, arguments)
    static CALLBACKS: RefCell<HashMap<u32, (JsObject, Vec<JsValue>)>> = RefCell::new(HashMap::new());
}

/// Store a callback and return its ID
pub fn store_callback(callback: JsObject, args: Vec<JsValue>) -> u32 {
    CALLBACKS.with(|callbacks| {
        let mut map = callbacks.borrow_mut();
        // Find next available ID
        let mut id = 1;
        while map.contains_key(&id) {
            id += 1;
        }
        map.insert(id, (callback, args));
        id
    })
}

/// Remove a callback by ID
pub fn remove_callback(callback_id: u32) {
    CALLBACKS.with(|callbacks| {
        let mut map = callbacks.borrow_mut();
        map.remove(&callback_id);
    });
}

/// Execute a callback by ID
pub fn execute_callback(callback_id: u32, context: &mut Context) -> JsResult<()> {
    // Get the callback
    let (callback, args) = CALLBACKS.with(|callbacks| {
        let map = callbacks.borrow();
        map.get(&callback_id).cloned()
    }).ok_or_else(|| {
        JsNativeError::error().with_message(format!("Callback {} not found", callback_id))
    })?;

    // Execute it
    if callback.is_callable() {
        let this = JsValue::undefined();
        let _ = callback.call(&this, &args, context);
        // Ignore errors to prevent one callback from blocking others
    }

    Ok(())
}

/// Clear all callbacks (used when worker terminates)
pub fn clear_all_callbacks() {
    CALLBACKS.with(|callbacks| {
        let mut map = callbacks.borrow_mut();
        map.clear();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::object::FunctionObjectBuilder;

    #[test]
    fn test_store_and_execute_callback() {
        let mut context = Context::default();

        // Create a simple callback
        let callback_fn = FunctionObjectBuilder::new(
            context.realm(),
            boa_engine::NativeFunction::from_fn_ptr(|_, _, _| Ok(JsValue::from(42)))
        ).build();
        let callback = callback_fn.into();  // Convert JsFunction to JsObject

        // Store it
        let id = store_callback(callback, vec![]);
        assert_eq!(id, 1);

        // Execute it
        let result = execute_callback(id, &mut context);
        assert!(result.is_ok());

        // Remove it
        remove_callback(id);
    }
}
