//! Event handling methods for Document
//!
//! addEventListener, removeEventListener, dispatchEvent, startViewTransition

use boa_engine::{
    builtins::BuiltInBuilder,
    object::JsObject,
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
    property::PropertyDescriptorBuilder
};

use super::types::DocumentData;
use crate::events::event::EventData;
use crate::events::custom_event::CustomEventData;
use crate::events::ui_events::{
    UIEventData, MouseEventData, KeyboardEventData, FocusEventData, InputEventData,
};

/// `Document.prototype.addEventListener(type, listener)`
pub(super) fn add_event_listener(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.addEventListener called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.addEventListener called on non-Document object")
    })?;

    let event_type = args.get_or_undefined(0).to_string(context)?;
    let listener = args.get_or_undefined(1).clone();

    document.add_event_listener(event_type.to_std_string_escaped(), listener);
    Ok(JsValue::undefined())
}

/// `Document.prototype.removeEventListener(type, listener)`
pub(super) fn remove_event_listener(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.removeEventListener called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.removeEventListener called on non-Document object")
    })?;

    let event_type = args.get_or_undefined(0).to_string(context)?;
    let listener = args.get_or_undefined(1);

    document.remove_event_listener(&event_type.to_std_string_escaped(), listener);
    Ok(JsValue::undefined())
}

/// `Document.prototype.dispatchEvent(event)`
pub(super) fn dispatch_event(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.dispatchEvent called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.dispatchEvent called on non-Document object")
    })?;

    let event = args.get_or_undefined(0);

    // Get event type from event object
    if event.is_object() {
        if let Some(event_obj) = event.as_object() {
            if let Ok(type_val) = event_obj.get(js_string!("type"), context) {
                let event_type = type_val.to_string(context)?;
                let listeners = document.get_event_listeners(&event_type.to_std_string_escaped());

                // Call each listener
                for listener in listeners {
                    if listener.is_callable() {
                        let _ = listener.as_callable().unwrap().call(
                            &this_obj.clone().into(),
                            &[event.clone()],
                            context,
                        );
                    }
                }
            }
        }
    }

    Ok(true.into())
}

/// `Document.prototype.startViewTransition(callback)`
pub(super) fn start_view_transition(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.startViewTransition called on non-object")
    })?;

    let _document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.startViewTransition called on non-Document object")
    })?;

    let callback = args.get_or_undefined(0);

    // Create transition object
    let transition = JsObject::default(context.intrinsics());

    // Add finished property as resolved Promise
    let finished_promise = context.eval(boa_engine::Source::from_bytes("Promise.resolve()"))?;
    transition.define_property_or_throw(
        js_string!("finished"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(false)
            .value(finished_promise)
            .build(),
        context,
    )?;

    // Add ready property as resolved Promise
    let ready_promise = context.eval(boa_engine::Source::from_bytes("Promise.resolve()"))?;
    transition.define_property_or_throw(
        js_string!("ready"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(false)
            .value(ready_promise)
            .build(),
        context,
    )?;

    // Handle callback if provided
    let mut callback_promise = context.eval(boa_engine::Source::from_bytes("Promise.resolve()"))?;
    if !callback.is_undefined() && callback.is_callable() {
        // Call the callback function
        if let Ok(result) = callback.as_callable()
            .unwrap()
            .call(&JsValue::undefined(), &[], context) {

            // Check if result is a promise
            if result.is_object() {
                if let Some(obj) = result.as_object() {
                    if obj.has_property(js_string!("then"), context)? {
                        callback_promise = result;
                    }
                }
            }
        }
    }

    // Add updateCallbackDone property
    transition.define_property_or_throw(
        js_string!("updateCallbackDone"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(false)
            .value(callback_promise)
            .build(),
        context,
    )?;

    // Add skipTransition method
    let skip_function = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
        Ok(JsValue::undefined())
    })
    .name(js_string!("skipTransition"))
    .build();

    transition.define_property_or_throw(
        js_string!("skipTransition"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(false)
            .value(skip_function)
            .build(),
        context,
    )?;

    Ok(transition.into())
}

/// `Document.prototype.createEvent(interface)`
///
/// Legacy method for creating events. Creates an uninitialized event of the
/// specified type. The caller must then call initEvent() (or initCustomEvent(),
/// initMouseEvent(), etc.) to initialize it before dispatching.
///
/// Per the W3C DOM spec: https://dom.spec.whatwg.org/#dom-document-createevent
pub(super) fn create_event(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let interface = args.get_or_undefined(0).to_string(context)?;
    let interface_str = interface.to_std_string_escaped();

    // Per spec, the interface string is case-insensitive
    let interface_lower = interface_str.to_ascii_lowercase();

    match interface_lower.as_str() {
        // Event, Events, HTMLEvents → base Event
        "event" | "events" | "htmlevents" => {
            let data = EventData::new(String::new(), false, false);
            let proto = context.intrinsics().constructors().event().prototype();
            let event = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                proto,
                data,
            );
            Ok(event.into())
        }

        // CustomEvent → CustomEvent
        "customevent" => {
            let event_data = EventData::new(String::new(), false, false);
            let data = CustomEventData::new(event_data, JsValue::null());
            let proto = context.intrinsics().constructors().custom_event().prototype();
            let event = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                proto,
                data,
            );
            Ok(event.into())
        }

        // UIEvent, UIEvents → UIEvent
        "uievent" | "uievents" => {
            let data = UIEventData::new(String::new(), false, false);
            let proto = context.intrinsics().constructors().ui_event().prototype();
            let event = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                proto,
                data,
            );
            Ok(event.into())
        }

        // MouseEvent, MouseEvents → MouseEvent
        "mouseevent" | "mouseevents" => {
            let data = MouseEventData::new(String::new(), false, false);
            let proto = context.intrinsics().constructors().mouse_event().prototype();
            let event = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                proto,
                data,
            );
            Ok(event.into())
        }

        // KeyboardEvent, KeyboardEvents → KeyboardEvent
        "keyboardevent" | "keyboardevents" => {
            let data = KeyboardEventData::new(String::new(), false, false);
            let proto = context.intrinsics().constructors().keyboard_event().prototype();
            let event = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                proto,
                data,
            );
            Ok(event.into())
        }

        // FocusEvent → FocusEvent
        "focusevent" => {
            let data = FocusEventData::new(String::new(), false, false);
            let proto = context.intrinsics().constructors().focus_event().prototype();
            let event = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                proto,
                data,
            );
            Ok(event.into())
        }

        // InputEvent → InputEvent
        "inputevent" => {
            let data = InputEventData::new(String::new(), false, false);
            let proto = context.intrinsics().constructors().input_event().prototype();
            let event = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                proto,
                data,
            );
            Ok(event.into())
        }

        _ => {
            Err(JsNativeError::error()
                .with_message(format!("The provided event type ('{}') is not supported", interface_str))
                .into())
        }
    }
}
