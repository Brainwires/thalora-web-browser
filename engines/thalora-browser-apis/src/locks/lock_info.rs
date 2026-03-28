//! Lock information structures for query() method.

use super::lock::LockMode;
use boa_engine::{Context, JsResult, JsValue, js_string, object::ObjectInitializer};

/// Information about a lock (held or pending).
#[derive(Debug, Clone)]
pub struct LockInfo {
    pub name: String,
    pub mode: LockMode,
    pub client_id: String,
}

impl LockInfo {
    pub fn new(name: String, mode: LockMode) -> Self {
        Self {
            name,
            mode,
            client_id: "default-client".to_string(), // Simplified for single-context
        }
    }

    /// Convert to JavaScript object
    pub fn to_js_object(&self, context: &mut Context) -> JsResult<JsValue> {
        let obj = ObjectInitializer::new(context)
            .property(
                js_string!("name"),
                js_string!(self.name.clone()),
                boa_engine::property::Attribute::all(),
            )
            .property(
                js_string!("mode"),
                js_string!(self.mode.as_str()),
                boa_engine::property::Attribute::all(),
            )
            .property(
                js_string!("clientId"),
                js_string!(self.client_id.clone()),
                boa_engine::property::Attribute::all(),
            )
            .build();

        Ok(obj.into())
    }
}

/// Snapshot of current lock state.
#[derive(Debug, Clone)]
pub struct LockManagerSnapshot {
    pub held: Vec<LockInfo>,
    pub pending: Vec<LockInfo>,
}

impl LockManagerSnapshot {
    pub fn new(held: Vec<LockInfo>, pending: Vec<LockInfo>) -> Self {
        Self { held, pending }
    }

    /// Convert to JavaScript object
    pub fn to_js_object(&self, context: &mut Context) -> JsResult<JsValue> {
        // Convert held locks to JS array
        let held_array = boa_engine::builtins::array::Array::array_create(
            self.held.len() as u64,
            None,
            context,
        )?;

        for (i, lock_info) in self.held.iter().enumerate() {
            held_array.set(i, lock_info.to_js_object(context)?, true, context)?;
        }

        // Convert pending locks to JS array
        let pending_array = boa_engine::builtins::array::Array::array_create(
            self.pending.len() as u64,
            None,
            context,
        )?;

        for (i, lock_info) in self.pending.iter().enumerate() {
            pending_array.set(i, lock_info.to_js_object(context)?, true, context)?;
        }

        // Create snapshot object
        let obj = ObjectInitializer::new(context)
            .property(
                js_string!("held"),
                held_array,
                boa_engine::property::Attribute::all(),
            )
            .property(
                js_string!("pending"),
                pending_array,
                boa_engine::property::Attribute::all(),
            )
            .build();

        Ok(obj.into())
    }
}
