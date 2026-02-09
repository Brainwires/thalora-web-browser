//! EventTarget interface implementation for DOM Level 4
//!
//! The EventTarget interface represents a target to which an event can be dispatched.
//! It is implemented by all objects that can receive and handle events.
//! https://dom.spec.whatwg.org/#interface-eventtarget

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    string::{StaticJsStrings, JsString},
    Context, JsArgs, JsData, JsNativeError, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace, GcRefCell};
use std::collections::HashMap;

/// Event listener entry with options
#[derive(Debug, Clone, Trace, Finalize)]
pub struct EventListener {
    /// The callback function
    callback: JsValue,
    /// Capture flag
    capture: bool,
    /// Once flag - remove after first invocation
    once: bool,
    /// Passive flag - cannot call preventDefault
    passive: bool,
    /// AbortSignal for bulk removal (simplified)
    aborted: bool,
}

impl EventListener {
    pub fn new(callback: JsValue, capture: bool, once: bool, passive: bool) -> Self {
        Self {
            callback,
            capture,
            once,
            passive,
            aborted: false,
        }
    }

    pub fn matches(&self, callback: &JsValue, capture: bool) -> bool {
        // For removal, we only check callback and capture flag
        JsValue::same_value(&self.callback, callback) && self.capture == capture
    }

    pub fn is_active(&self) -> bool {
        !self.aborted
    }

    pub fn abort(&mut self) {
        self.aborted = true;
    }
}

/// The EventTarget data implementation
#[derive(Debug, Trace, Finalize, JsData)]
pub struct EventTargetData {
    /// Event listener list - maps event type to list of listeners
    listeners: GcRefCell<HashMap<String, Vec<EventListener>>>,
}

impl EventTargetData {
    /// Create a new EventTarget
    pub fn new() -> Self {
        Self {
            listeners: GcRefCell::new(HashMap::new()),
        }
    }

    /// Add an event listener (prevents duplicates per spec)
    pub fn add_event_listener(
        &self,
        event_type: String,
        callback: JsValue,
        capture: bool,
        once: bool,
        passive: bool,
    ) {
        if callback.is_null() || callback.is_undefined() {
            return;
        }

        let mut listeners = self.listeners.borrow_mut();
        let list = listeners.entry(event_type).or_insert_with(Vec::new);

        // Per spec: if an identical listener (same callback + capture) exists, don't add
        if list.iter().any(|l| l.matches(&callback, capture)) {
            return;
        }

        list.push(EventListener::new(callback, capture, once, passive));
    }

    /// Remove an event listener
    pub fn remove_event_listener(&self, event_type: &str, callback: &JsValue, capture: bool) {
        if let Some(listeners) = self.listeners.borrow_mut().get_mut(event_type) {
            listeners.retain(|listener| !listener.matches(callback, capture));
        }
    }

    /// Get listeners for a specific event type and phase
    /// If capture is true, returns only capture listeners; if false, returns only bubble listeners
    pub fn get_listeners_for_phase(&self, event_type: &str, capture: bool) -> Vec<JsValue> {
        if let Some(listeners) = self.listeners.borrow().get(event_type) {
            listeners
                .iter()
                .filter(|l| l.is_active() && l.capture == capture)
                .map(|l| l.callback.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all listeners for a specific event type (both capture and bubble)
    pub fn get_all_listeners(&self, event_type: &str) -> Vec<JsValue> {
        if let Some(listeners) = self.listeners.borrow().get(event_type) {
            listeners
                .iter()
                .filter(|l| l.is_active())
                .map(|l| l.callback.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Dispatch an event to all matching listeners.
    /// `this_obj` is the EventTarget object (used to set target/currentTarget).
    pub fn dispatch_event_with_target(&self, event: &JsObject, this_obj: &JsObject, context: &mut Context) -> JsResult<bool> {
        // Get event type
        let event_type = if let Ok(type_prop) = event.get(js_string!("type"), context) {
            if let Ok(type_str) = type_prop.to_string(context) {
                type_str.to_std_string_escaped()
            } else {
                return Ok(true);
            }
        } else {
            return Ok(true);
        };

        // Set event.target and event.currentTarget to this EventTarget
        let _ = event.set(js_string!("target"), this_obj.clone(), false, context);
        let _ = event.set(js_string!("currentTarget"), this_obj.clone(), false, context);

        // Also set on native event data if applicable
        if let Some(mut event_data) = event.downcast_mut::<super::event::EventData>() {
            event_data.set_target(Some(this_obj.clone()));
            event_data.set_current_target(Some(this_obj.clone()));
            event_data.set_phase(super::event::EventPhase::AtTarget);
        }

        let mut prevent_default_called = false;
        let mut indices_to_remove = Vec::new();

        if let Some(listeners) = self.listeners.borrow().get(&event_type) {
            let listeners_copy = listeners.clone();

            for (index, listener) in listeners_copy.iter().enumerate() {
                if !listener.is_active() {
                    continue;
                }

                if let Some(func) = listener.callback.as_callable() {
                    let _ = func.call(&this_obj.clone().into(), &[event.clone().into()], context);

                    if let Ok(default_prevented) = event.get(js_string!("defaultPrevented"), context) {
                        if default_prevented.to_boolean() {
                            prevent_default_called = true;
                        }
                    }
                }

                if listener.once {
                    indices_to_remove.push(index);
                }
            }
        }

        // Remove "once" listeners in reverse order
        indices_to_remove.reverse();
        for index in indices_to_remove {
            if let Some(listeners) = self.listeners.borrow_mut().get_mut(&event_type) {
                if index < listeners.len() {
                    listeners.remove(index);
                }
            }
        }

        Ok(!prevent_default_called)
    }

    /// Dispatch an event to all matching listeners (legacy API without target)
    pub fn dispatch_event(&self, event: &JsObject, context: &mut Context) -> JsResult<bool> {
        // Get event type
        let event_type = if let Ok(type_prop) = event.get(js_string!("type"), context) {
            if let Ok(type_str) = type_prop.to_string(context) {
                type_str.to_std_string_escaped()
            } else {
                return Ok(true);
            }
        } else {
            return Ok(true);
        };

        let mut prevent_default_called = false;
        let mut indices_to_remove = Vec::new();

        if let Some(listeners) = self.listeners.borrow().get(&event_type) {
            let listeners_copy = listeners.clone();

            for (index, listener) in listeners_copy.iter().enumerate() {
                if !listener.is_active() {
                    continue;
                }

                if let Some(func) = listener.callback.as_callable() {
                    let _ = func.call(&JsValue::undefined(), &[event.clone().into()], context);

                    if let Ok(default_prevented) = event.get(js_string!("defaultPrevented"), context) {
                        if default_prevented.to_boolean() {
                            prevent_default_called = true;
                        }
                    }
                }

                if listener.once {
                    indices_to_remove.push(index);
                }
            }
        }

        indices_to_remove.reverse();
        for index in indices_to_remove {
            if let Some(listeners) = self.listeners.borrow_mut().get_mut(&event_type) {
                if index < listeners.len() {
                    listeners.remove(index);
                }
            }
        }

        Ok(!prevent_default_called)
    }

}

/// The `EventTarget` object
#[derive(Debug, Trace, Finalize)]
pub struct EventTarget;

impl EventTarget {
    /// Create a new EventTarget
    pub fn create(context: &mut Context) -> JsResult<JsObject> {
        let target_data = EventTargetData::new();

        let target_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().event_target().prototype(),
            target_data,
        );

        Ok(target_obj.upcast())
    }

    /// Static method implementations for BuiltInBuilder
    pub fn add_event_listener(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("EventTarget.addEventListener called on non-object")
        })?;

        let target_data = this_obj.downcast_ref::<EventTargetData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("EventTarget.addEventListener called on non-EventTarget object")
        })?;

        let event_type = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let callback = args.get_or_undefined(1);
        let options = args.get_or_undefined(2);

        // Parse options (can be boolean for capture or object)
        let (capture, once, passive) = if options.is_boolean() {
            (options.to_boolean(), false, false)
        } else if let Some(options_obj) = options.as_object() {
            let capture = if let Ok(cap) = options_obj.get(js_string!("capture"), context) {
                cap.to_boolean()
            } else {
                false
            };

            let once = if let Ok(once_prop) = options_obj.get(js_string!("once"), context) {
                once_prop.to_boolean()
            } else {
                false
            };

            let passive = if let Ok(passive_prop) = options_obj.get(js_string!("passive"), context) {
                passive_prop.to_boolean()
            } else {
                false
            };

            (capture, once, passive)
        } else {
            (false, false, false)
        };

        target_data.add_event_listener(event_type, callback.clone(), capture, once, passive);
        Ok(JsValue::undefined())
    }

    pub fn remove_event_listener(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("EventTarget.removeEventListener called on non-object")
        })?;

        let target_data = this_obj.downcast_ref::<EventTargetData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("EventTarget.removeEventListener called on non-EventTarget object")
        })?;

        let event_type = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let callback = args.get_or_undefined(1);
        let options = args.get_or_undefined(2);

        // Parse capture option (can be boolean for capture or object)
        let capture = if options.is_boolean() {
            options.to_boolean()
        } else if let Some(options_obj) = options.as_object() {
            if let Ok(cap) = options_obj.get(js_string!("capture"), context) {
                cap.to_boolean()
            } else {
                false
            }
        } else {
            false
        };

        target_data.remove_event_listener(&event_type, &callback, capture);
        Ok(JsValue::undefined())
    }

    pub fn dispatch_event(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("EventTarget.dispatchEvent called on non-object")
        })?;

        let target_data = this_obj.downcast_ref::<EventTargetData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("EventTarget.dispatchEvent called on non-EventTarget object")
        })?;

        let event_arg = args.get_or_undefined(0);

        if let Some(event_obj) = event_arg.as_object() {
            let result = target_data.dispatch_event_with_target(&event_obj, &this_obj, context)?;
            Ok(JsValue::new(result))
        } else {
            Err(JsNativeError::typ()
                .with_message("EventTarget.dispatchEvent requires an Event object")
                .into())
        }
    }
}

impl IntrinsicObject for EventTarget {
    fn init(realm: &Realm) {
        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Core EventTarget methods
            .method(Self::add_event_listener, js_string!("addEventListener"), 2)
            .method(Self::remove_event_listener, js_string!("removeEventListener"), 2)
            .method(Self::dispatch_event, js_string!("dispatchEvent"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for EventTarget {
    const NAME: JsString = StaticJsStrings::EVENT_TARGET;
}

impl BuiltInConstructor for EventTarget {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::event_target;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // EventTarget constructor should be called with 'new'
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Constructor EventTarget requires 'new'")
                .into());
        }

        // Create a new EventTarget object
        let target_data = EventTargetData::new();

        let target_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().event_target().prototype(),
            target_data,
        );

        Ok(target_obj.upcast().into())
    }
}

