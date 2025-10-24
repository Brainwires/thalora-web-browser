//! IDBVersionChangeEvent Implementation
//!
//! Fired when a database structure change is requested.
//!
//! Spec: https://w3c.github.io/IndexedDB/#idbversionchangeevent

use boa_engine::{JsValue, Context, JsResult};

/// Version change event
#[derive(Debug, Clone)]
pub struct IDBVersionChangeEvent {
    pub event_type: String,
    pub old_version: u32,
    pub new_version: Option<u32>,
}

impl IDBVersionChangeEvent {
    /// Create a new version change event
    pub fn new(event_type: &str, old_version: u32, new_version: Option<u32>) -> Self {
        Self {
            event_type: event_type.to_string(),
            old_version,
            new_version,
        }
    }

    /// Convert to JsValue
    pub fn to_js_value(&self, context: &mut Context) -> JsResult<JsValue> {
        use boa_engine::{js_string, JsString, object::JsObject};

        let event = JsObject::with_object_proto(context.intrinsics());
        event.set(js_string!("type"), JsValue::from(JsString::from(self.event_type.clone())), false, context)?;
        event.set(js_string!("oldVersion"), JsValue::from(self.old_version), false, context)?;

        if let Some(new_version) = self.new_version {
            event.set(js_string!("newVersion"), JsValue::from(new_version), false, context)?;
        } else {
            event.set(js_string!("newVersion"), JsValue::null(), false, context)?;
        }

        Ok(event.into())
    }
}
