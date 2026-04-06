//! EventTarget interface implementation for DOM Level 4
//!
//! The EventTarget interface represents a target to which an event can be dispatched.
//! It is implemented by all objects that can receive and handle events.
//! https://dom.spec.whatwg.org/#interface-eventtarget

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    realm::Realm,
    string::{JsString, StaticJsStrings},
};
use boa_gc::{Finalize, GcRefCell, Trace};
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
}

impl Default for EventTargetData {
    fn default() -> Self {
        Self::new()
    }
}

impl EventTargetData {
    /// Add an event listener
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

        let listener = EventListener::new(callback, capture, once, passive);
        self.listeners
            .borrow_mut()
            .entry(event_type)
            .or_default()
            .push(listener);
    }

    /// Remove an event listener
    pub fn remove_event_listener(&self, event_type: &str, callback: &JsValue, capture: bool) {
        if let Some(listeners) = self.listeners.borrow_mut().get_mut(event_type) {
            listeners.retain(|listener| !listener.matches(callback, capture));
        }
    }

    /// Fire listeners on this target for the given event type and phase.
    /// Returns true if stopImmediatePropagation was called.
    pub(crate) fn fire_listeners(
        &self,
        event: &JsObject,
        event_type: &str,
        phase: &super::event::EventPhase,
        context: &mut Context,
    ) -> JsResult<bool> {
        let listeners_copy = {
            let borrowed = self.listeners.borrow();
            match borrowed.get(event_type) {
                Some(listeners) => listeners.clone(),
                None => return Ok(false),
            }
        };

        let mut once_indices = Vec::new();
        let mut stop_immediate = false;

        for (index, listener) in listeners_copy.iter().enumerate() {
            if !listener.is_active() {
                continue;
            }

            // Filter by phase: capture listeners fire during capture phase,
            // non-capture listeners fire during bubble phase.
            // At target phase, ALL listeners fire regardless of capture flag.
            match phase {
                super::event::EventPhase::CapturingPhase => {
                    if !listener.capture {
                        continue;
                    }
                }
                super::event::EventPhase::BubblingPhase => {
                    if listener.capture {
                        continue;
                    }
                }
                super::event::EventPhase::AtTarget => {
                    // Fire all listeners at target phase
                }
                super::event::EventPhase::None => continue,
            }

            // Call the listener
            if let Some(func) = listener.callback.as_callable() {
                let _ = func.call(&JsValue::undefined(), &[event.clone().into()], context);
            }

            if listener.once {
                once_indices.push(index);
            }

            // Check stopImmediatePropagation on the EventData
            if let Some(event_data) = event.downcast_ref::<super::event::EventData>()
                && event_data.should_stop_immediate_propagation()
            {
                stop_immediate = true;
                break;
            }
        }

        // Remove "once" listeners
        if !once_indices.is_empty() {
            once_indices.reverse();
            if let Ok(mut map) = self.listeners.try_borrow_mut()
                && let Some(listeners) = map.get_mut(event_type)
            {
                for index in once_indices {
                    if index < listeners.len() {
                        listeners.remove(index);
                    }
                }
            }
        }

        Ok(stop_immediate)
    }

    /// Dispatch an event with full DOM spec 3-phase propagation.
    ///
    /// 1. Build event path: walk parentNode from target to root
    /// 2. Capture phase: root → target (exclusive), fire capture listeners
    /// 3. Target phase: fire ALL listeners on target
    /// 4. Bubble phase: target parent → root, fire non-capture listeners (if bubbles)
    ///
    /// Sets event.target, event.currentTarget, event.eventPhase at each step.
    /// Respects stopPropagation and stopImmediatePropagation.
    pub fn dispatch_event(
        &self,
        event: &JsObject,
        target: &JsObject,
        context: &mut Context,
    ) -> JsResult<bool> {
        use super::event::{EventData, EventPhase};

        // Extract event type
        let event_type = if let Ok(type_prop) = event.get(js_string!("type"), context) {
            type_prop
                .to_string(context)
                .map(|s| s.to_std_string_escaped())
                .unwrap_or_default()
        } else {
            return Ok(true);
        };

        if event_type.is_empty() {
            return Ok(true);
        }

        // Read bubbles flag
        let bubbles = event
            .downcast_ref::<EventData>()
            .map(|d| d.get_bubbles())
            .unwrap_or(false);

        // Set event.target
        if let Some(mut event_data) = event.downcast_mut::<EventData>() {
            event_data.set_target(Some(target.clone()));
        }
        // Also set on the JS object for code that reads .target directly
        let _ = event.set(js_string!("target"), target.clone(), false, context);

        // Build event path: walk parentNode from target to root
        let mut path: Vec<JsObject> = Vec::new();
        {
            let mut current = target.clone();
            loop {
                let parent = current.get(js_string!("parentNode"), context);
                match parent {
                    Ok(ref val) if !val.is_null() && !val.is_undefined() => {
                        if let Some(parent_obj) = val.as_object() {
                            path.push(parent_obj.clone());
                            current = parent_obj.clone();
                        } else {
                            break;
                        }
                    }
                    _ => break,
                }
            }
        }
        path.reverse(); // Now root → ... → target's parent

        // === CAPTURE PHASE: root → target (exclusive) ===
        for node in &path {
            if let Some(mut event_data) = event.downcast_mut::<EventData>() {
                event_data.set_phase(EventPhase::CapturingPhase);
                event_data.set_current_target(Some(node.clone()));
            }
            let _ = event.set(js_string!("currentTarget"), node.clone(), false, context);
            let _ = event.set(js_string!("eventPhase"), JsValue::from(1), false, context);

            // Fire capture listeners on this node
            if let Some(target_data) = node.downcast_ref::<EventTargetData>() {
                target_data.fire_listeners(
                    event,
                    &event_type,
                    &EventPhase::CapturingPhase,
                    context,
                )?;
            }

            // Check stopPropagation
            if let Some(ed) = event.downcast_ref::<EventData>()
                && ed.should_stop_propagation()
            {
                break;
            }
        }

        // === TARGET PHASE ===
        let stopped = event
            .downcast_ref::<EventData>()
            .map(|d| d.should_stop_propagation())
            .unwrap_or(false);

        if !stopped {
            if let Some(mut event_data) = event.downcast_mut::<EventData>() {
                event_data.set_phase(EventPhase::AtTarget);
                event_data.set_current_target(Some(target.clone()));
            }
            let _ = event.set(js_string!("currentTarget"), target.clone(), false, context);
            let _ = event.set(js_string!("eventPhase"), JsValue::from(2), false, context);

            // Fire ALL listeners on target (capture and non-capture)
            self.fire_listeners(event, &event_type, &EventPhase::AtTarget, context)?;
        }

        // === BUBBLE PHASE: target parent → root ===
        if bubbles {
            let stopped = event
                .downcast_ref::<EventData>()
                .map(|d| d.should_stop_propagation())
                .unwrap_or(false);

            if !stopped {
                // path is root→...→target_parent, iterate in reverse for bubble
                for node in path.iter().rev() {
                    if let Some(mut event_data) = event.downcast_mut::<EventData>() {
                        event_data.set_phase(EventPhase::BubblingPhase);
                        event_data.set_current_target(Some(node.clone()));
                    }
                    let _ = event.set(js_string!("currentTarget"), node.clone(), false, context);
                    let _ = event.set(js_string!("eventPhase"), JsValue::from(3), false, context);

                    if let Some(target_data) = node.downcast_ref::<EventTargetData>() {
                        target_data.fire_listeners(
                            event,
                            &event_type,
                            &EventPhase::BubblingPhase,
                            context,
                        )?;
                    }

                    if let Some(ed) = event.downcast_ref::<EventData>()
                        && ed.should_stop_propagation()
                    {
                        break;
                    }
                }
            }
        }

        // Reset phase and currentTarget after dispatch
        if let Some(mut event_data) = event.downcast_mut::<EventData>() {
            event_data.set_phase(EventPhase::None);
            event_data.set_current_target(None);
        }
        let _ = event.set(js_string!("currentTarget"), JsValue::null(), false, context);
        let _ = event.set(js_string!("eventPhase"), JsValue::from(0), false, context);

        // Return true if preventDefault was NOT called
        if let Some(event_data) = event.downcast_ref::<EventData>() {
            Ok(!event_data.get_default_prevented())
        } else {
            Ok(true)
        }
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
            context
                .intrinsics()
                .constructors()
                .event_target()
                .prototype(),
            target_data,
        );

        Ok(target_obj.upcast())
    }

    /// Static method implementations for BuiltInBuilder
    pub fn add_event_listener(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("EventTarget.addEventListener called on non-object")
        })?;

        let target_data = this_obj.downcast_ref::<EventTargetData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("EventTarget.addEventListener called on non-EventTarget object")
        })?;

        let event_type = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
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

            let passive = if let Ok(passive_prop) = options_obj.get(js_string!("passive"), context)
            {
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

    pub fn remove_event_listener(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("EventTarget.removeEventListener called on non-object")
        })?;

        let target_data = this_obj.downcast_ref::<EventTargetData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("EventTarget.removeEventListener called on non-EventTarget object")
        })?;

        let event_type = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
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

        target_data.remove_event_listener(&event_type, callback, capture);
        Ok(JsValue::undefined())
    }

    pub fn dispatch_event(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("EventTarget.dispatchEvent called on non-object")
        })?;

        let target_data = this_obj.downcast_ref::<EventTargetData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("EventTarget.dispatchEvent called on non-EventTarget object")
        })?;

        let event_arg = args.get_or_undefined(0);

        if let Some(event_obj) = event_arg.as_object() {
            let result = target_data.dispatch_event(&event_obj, &this_obj, context)?;
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
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Core EventTarget methods
            .method(Self::add_event_listener, js_string!("addEventListener"), 2)
            .method(
                Self::remove_event_listener,
                js_string!("removeEventListener"),
                2,
            )
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
            context
                .intrinsics()
                .constructors()
                .event_target()
                .prototype(),
            target_data,
        );

        Ok(target_obj.upcast().into())
    }
}
