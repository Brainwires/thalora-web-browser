//! Event Propagation System
//!
//! Implements DOM Level 4 event propagation with proper phases:
//! - Capturing phase: From document root down to target
//! - At-target phase: At the target element itself
//! - Bubbling phase: From target back up to document root
//!
//! https://dom.spec.whatwg.org/#concept-event-dispatch

use boa_engine::{
    js_string,
    object::JsObject,
    Context, JsResult, JsValue,
};

use super::event::{EventData, EventPhase};
use super::event_target::EventTargetData;
use super::ui_events::{MouseEventData, KeyboardEventData, FocusEventData, InputEventData, UIEventData};
use crate::dom::element::{with_element_data, has_element_data};

/// Dispatch an event with proper DOM event propagation (capturing, at-target, bubbling phases)
///
/// This implements the full W3C/WHATWG event dispatch algorithm:
/// 1. Build the propagation path from target to document root
/// 2. Set event target
/// 3. Capturing phase: Walk down from root to target, call capture listeners
/// 4. At-target phase: Call all listeners on target
/// 5. Bubbling phase: Walk up from target to root, call bubble listeners (if event bubbles)
pub fn dispatch_event_with_propagation(
    event: &JsObject,
    target: &JsObject,
    context: &mut Context,
) -> JsResult<bool> {
    // 1. Build the propagation path (ancestors from target to root)
    let ancestors = build_propagation_path(target);

    // 2. Set the event target (remains constant throughout dispatch)
    set_event_target(event, target.clone())?;

    // Track if default was prevented
    let mut default_prevented = false;

    // Check if event bubbles
    let bubbles = get_event_bubbles(event, context)?;

    // 3. Capturing phase - walk DOWN from root to target
    set_event_phase(event, EventPhase::CapturingPhase)?;
    for ancestor in ancestors.iter().rev() {
        if should_stop_propagation(event)? {
            break;
        }

        set_event_current_target(event, ancestor.clone())?;

        // Call capturing listeners (capture: true)
        if let Some(listeners) = get_capture_listeners(ancestor, event, context)? {
            for listener in listeners {
                if should_stop_immediate_propagation(event)? {
                    break;
                }
                call_listener(&listener, event, context)?;
                if is_default_prevented(event, context)? {
                    default_prevented = true;
                }
            }
        }
    }

    // 4. At-target phase - call ALL listeners on target (both capture and bubble)
    if !should_stop_propagation(event)? {
        set_event_phase(event, EventPhase::AtTarget)?;
        set_event_current_target(event, target.clone())?;

        // Get all listeners for at-target phase
        if let Some(listeners) = get_all_listeners(target, event, context)? {
            for listener in listeners {
                if should_stop_immediate_propagation(event)? {
                    break;
                }
                call_listener(&listener, event, context)?;
                if is_default_prevented(event, context)? {
                    default_prevented = true;
                }
            }
        }
    }

    // 5. Bubbling phase - walk UP from target to root (if event bubbles)
    if bubbles && !should_stop_propagation(event)? {
        set_event_phase(event, EventPhase::BubblingPhase)?;

        for ancestor in ancestors.iter() {
            if should_stop_propagation(event)? {
                break;
            }

            set_event_current_target(event, ancestor.clone())?;

            // Call bubbling listeners (capture: false)
            if let Some(listeners) = get_bubble_listeners(ancestor, event, context)? {
                for listener in listeners {
                    if should_stop_immediate_propagation(event)? {
                        break;
                    }
                    call_listener(&listener, event, context)?;
                    if is_default_prevented(event, context)? {
                        default_prevented = true;
                    }
                }
            }
        }
    }

    // 6. Reset event phase to NONE after dispatch
    set_event_phase(event, EventPhase::None)?;

    // Return true if default was NOT prevented
    Ok(!default_prevented)
}

/// Build the propagation path from target to document root
/// Returns ancestors in order from target's parent to root (closest first)
fn build_propagation_path(target: &JsObject) -> Vec<JsObject> {
    let mut path = Vec::new();

    // Walk up the parent chain
    let mut current = get_parent_node(target);

    while let Some(parent) = current {
        path.push(parent.clone());
        current = get_parent_node(&parent);
    }

    path
}

/// Get parent node from element
fn get_parent_node(element: &JsObject) -> Option<JsObject> {
    // Try ElementData first (dispatches across all element types)
    if let Ok(parent) = with_element_data(element, |el| el.get_parent_node(), "not an element") {
        return parent;
    }

    // Could also check EventTargetData for document/window
    None
}

/// Set the event target property
fn set_event_target(event: &JsObject, target: JsObject) -> JsResult<()> {
    if let Some(mut event_data) = event.downcast_mut::<EventData>() {
        event_data.set_target(Some(target));
        return Ok(());
    }
    if let Some(mut mouse_data) = event.downcast_mut::<MouseEventData>() {
        mouse_data.ui_event.event.set_target(Some(target));
        return Ok(());
    }
    if let Some(mut keyboard_data) = event.downcast_mut::<KeyboardEventData>() {
        keyboard_data.ui_event.event.set_target(Some(target));
        return Ok(());
    }
    if let Some(mut focus_data) = event.downcast_mut::<FocusEventData>() {
        focus_data.ui_event.event.set_target(Some(target));
        return Ok(());
    }
    if let Some(mut input_data) = event.downcast_mut::<InputEventData>() {
        input_data.ui_event.event.set_target(Some(target));
        return Ok(());
    }
    if let Some(mut ui_data) = event.downcast_mut::<UIEventData>() {
        ui_data.event.set_target(Some(target));
        return Ok(());
    }

    Ok(())
}

/// Set the event current target property
fn set_event_current_target(event: &JsObject, target: JsObject) -> JsResult<()> {
    if let Some(mut event_data) = event.downcast_mut::<EventData>() {
        event_data.set_current_target(Some(target));
        return Ok(());
    }
    if let Some(mut mouse_data) = event.downcast_mut::<MouseEventData>() {
        mouse_data.ui_event.event.set_current_target(Some(target));
        return Ok(());
    }
    if let Some(mut keyboard_data) = event.downcast_mut::<KeyboardEventData>() {
        keyboard_data.ui_event.event.set_current_target(Some(target));
        return Ok(());
    }
    if let Some(mut focus_data) = event.downcast_mut::<FocusEventData>() {
        focus_data.ui_event.event.set_current_target(Some(target));
        return Ok(());
    }
    if let Some(mut input_data) = event.downcast_mut::<InputEventData>() {
        input_data.ui_event.event.set_current_target(Some(target));
        return Ok(());
    }
    if let Some(mut ui_data) = event.downcast_mut::<UIEventData>() {
        ui_data.event.set_current_target(Some(target));
        return Ok(());
    }

    Ok(())
}

/// Set the event phase property
fn set_event_phase(event: &JsObject, phase: EventPhase) -> JsResult<()> {
    if let Some(mut event_data) = event.downcast_mut::<EventData>() {
        event_data.set_phase(phase);
        return Ok(());
    }
    if let Some(mut mouse_data) = event.downcast_mut::<MouseEventData>() {
        mouse_data.ui_event.event.set_phase(phase);
        return Ok(());
    }
    if let Some(mut keyboard_data) = event.downcast_mut::<KeyboardEventData>() {
        keyboard_data.ui_event.event.set_phase(phase);
        return Ok(());
    }
    if let Some(mut focus_data) = event.downcast_mut::<FocusEventData>() {
        focus_data.ui_event.event.set_phase(phase);
        return Ok(());
    }
    if let Some(mut input_data) = event.downcast_mut::<InputEventData>() {
        input_data.ui_event.event.set_phase(phase);
        return Ok(());
    }
    if let Some(mut ui_data) = event.downcast_mut::<UIEventData>() {
        ui_data.event.set_phase(phase);
        return Ok(());
    }

    Ok(())
}

/// Get whether event bubbles
fn get_event_bubbles(event: &JsObject, context: &mut Context) -> JsResult<bool> {
    // Try getting from EventData types
    if let Some(event_data) = event.downcast_ref::<EventData>() {
        return Ok(event_data.get_bubbles());
    }
    if let Some(mouse_data) = event.downcast_ref::<MouseEventData>() {
        return Ok(mouse_data.ui_event.event.get_bubbles());
    }
    if let Some(keyboard_data) = event.downcast_ref::<KeyboardEventData>() {
        return Ok(keyboard_data.ui_event.event.get_bubbles());
    }
    if let Some(focus_data) = event.downcast_ref::<FocusEventData>() {
        return Ok(focus_data.ui_event.event.get_bubbles());
    }
    if let Some(input_data) = event.downcast_ref::<InputEventData>() {
        return Ok(input_data.ui_event.event.get_bubbles());
    }
    if let Some(ui_data) = event.downcast_ref::<UIEventData>() {
        return Ok(ui_data.event.get_bubbles());
    }

    // Fallback: try reading bubbles property from object
    if let Ok(bubbles) = event.get(js_string!("bubbles"), context) {
        return Ok(bubbles.to_boolean());
    }

    Ok(false)
}

/// Check if stopPropagation was called
fn should_stop_propagation(event: &JsObject) -> JsResult<bool> {
    if let Some(event_data) = event.downcast_ref::<EventData>() {
        return Ok(event_data.should_stop_propagation());
    }
    if let Some(mouse_data) = event.downcast_ref::<MouseEventData>() {
        return Ok(mouse_data.ui_event.event.should_stop_propagation());
    }
    if let Some(keyboard_data) = event.downcast_ref::<KeyboardEventData>() {
        return Ok(keyboard_data.ui_event.event.should_stop_propagation());
    }
    if let Some(focus_data) = event.downcast_ref::<FocusEventData>() {
        return Ok(focus_data.ui_event.event.should_stop_propagation());
    }
    if let Some(input_data) = event.downcast_ref::<InputEventData>() {
        return Ok(input_data.ui_event.event.should_stop_propagation());
    }
    if let Some(ui_data) = event.downcast_ref::<UIEventData>() {
        return Ok(ui_data.event.should_stop_propagation());
    }

    Ok(false)
}

/// Check if stopImmediatePropagation was called
fn should_stop_immediate_propagation(event: &JsObject) -> JsResult<bool> {
    if let Some(event_data) = event.downcast_ref::<EventData>() {
        return Ok(event_data.should_stop_immediate_propagation());
    }
    if let Some(mouse_data) = event.downcast_ref::<MouseEventData>() {
        return Ok(mouse_data.ui_event.event.should_stop_immediate_propagation());
    }
    if let Some(keyboard_data) = event.downcast_ref::<KeyboardEventData>() {
        return Ok(keyboard_data.ui_event.event.should_stop_immediate_propagation());
    }
    if let Some(focus_data) = event.downcast_ref::<FocusEventData>() {
        return Ok(focus_data.ui_event.event.should_stop_immediate_propagation());
    }
    if let Some(input_data) = event.downcast_ref::<InputEventData>() {
        return Ok(input_data.ui_event.event.should_stop_immediate_propagation());
    }
    if let Some(ui_data) = event.downcast_ref::<UIEventData>() {
        return Ok(ui_data.event.should_stop_immediate_propagation());
    }

    Ok(false)
}

/// Check if preventDefault was called
fn is_default_prevented(event: &JsObject, context: &mut Context) -> JsResult<bool> {
    if let Some(event_data) = event.downcast_ref::<EventData>() {
        return Ok(event_data.get_default_prevented());
    }
    if let Some(mouse_data) = event.downcast_ref::<MouseEventData>() {
        return Ok(mouse_data.ui_event.event.get_default_prevented());
    }
    if let Some(keyboard_data) = event.downcast_ref::<KeyboardEventData>() {
        return Ok(keyboard_data.ui_event.event.get_default_prevented());
    }
    if let Some(focus_data) = event.downcast_ref::<FocusEventData>() {
        return Ok(focus_data.ui_event.event.get_default_prevented());
    }
    if let Some(input_data) = event.downcast_ref::<InputEventData>() {
        return Ok(input_data.ui_event.event.get_default_prevented());
    }
    if let Some(ui_data) = event.downcast_ref::<UIEventData>() {
        return Ok(ui_data.event.get_default_prevented());
    }

    // Fallback: try reading defaultPrevented property
    if let Ok(prevented) = event.get(js_string!("defaultPrevented"), context) {
        return Ok(prevented.to_boolean());
    }

    Ok(false)
}

/// Get event type
fn get_event_type(event: &JsObject, context: &mut Context) -> JsResult<String> {
    if let Some(event_data) = event.downcast_ref::<EventData>() {
        return Ok(event_data.get_type().to_string());
    }
    if let Some(mouse_data) = event.downcast_ref::<MouseEventData>() {
        return Ok(mouse_data.ui_event.event.get_type().to_string());
    }
    if let Some(keyboard_data) = event.downcast_ref::<KeyboardEventData>() {
        return Ok(keyboard_data.ui_event.event.get_type().to_string());
    }
    if let Some(focus_data) = event.downcast_ref::<FocusEventData>() {
        return Ok(focus_data.ui_event.event.get_type().to_string());
    }
    if let Some(input_data) = event.downcast_ref::<InputEventData>() {
        return Ok(input_data.ui_event.event.get_type().to_string());
    }
    if let Some(ui_data) = event.downcast_ref::<UIEventData>() {
        return Ok(ui_data.event.get_type().to_string());
    }

    // Fallback: try reading type property
    if let Ok(type_val) = event.get(js_string!("type"), context) {
        if let Ok(type_str) = type_val.to_string(context) {
            return Ok(type_str.to_std_string_escaped());
        }
    }

    Ok(String::new())
}

/// Listener entry with capture flag
struct ListenerEntry {
    callback: JsValue,
    capture: bool,
    once: bool,
}

/// Get capture listeners (capture: true) from target
fn get_capture_listeners(
    target: &JsObject,
    event: &JsObject,
    context: &mut Context,
) -> JsResult<Option<Vec<JsValue>>> {
    let event_type = get_event_type(event, context)?;

    // Try EventTargetData (for proper EventTarget objects)
    if let Some(target_data) = target.downcast_ref::<EventTargetData>() {
        let listeners = target_data.get_listeners_for_phase(&event_type, true);
        if !listeners.is_empty() {
            return Ok(Some(listeners));
        }
    }

    // Try ElementData (for DOM elements - dispatches across all element types)
    if has_element_data(target) {
        // ElementData currently doesn't track capture flag, so we return empty for capture phase
        // This will be updated when we add capture support to ElementData
        return Ok(None);
    }

    Ok(None)
}

/// Get bubble listeners (capture: false) from target
fn get_bubble_listeners(
    target: &JsObject,
    event: &JsObject,
    context: &mut Context,
) -> JsResult<Option<Vec<JsValue>>> {
    let event_type = get_event_type(event, context)?;

    // Try EventTargetData
    if let Some(target_data) = target.downcast_ref::<EventTargetData>() {
        let listeners = target_data.get_listeners_for_phase(&event_type, false);
        if !listeners.is_empty() {
            return Ok(Some(listeners));
        }
    }

    // Try ElementData - these are bubble listeners by default (dispatches across all element types)
    if let Ok(listeners) = with_element_data(target, |el| el.get_event_listeners(&event_type), "not an element") {
        if let Some(listeners) = listeners {
            if !listeners.is_empty() {
                return Ok(Some(listeners));
            }
        }
    }

    Ok(None)
}

/// Get all listeners for at-target phase (both capture and bubble)
fn get_all_listeners(
    target: &JsObject,
    event: &JsObject,
    context: &mut Context,
) -> JsResult<Option<Vec<JsValue>>> {
    let event_type = get_event_type(event, context)?;
    let mut all_listeners = Vec::new();

    // Try EventTargetData
    if let Some(target_data) = target.downcast_ref::<EventTargetData>() {
        // Get both capture and bubble listeners
        let capture_listeners = target_data.get_listeners_for_phase(&event_type, true);
        let bubble_listeners = target_data.get_listeners_for_phase(&event_type, false);
        all_listeners.extend(capture_listeners);
        all_listeners.extend(bubble_listeners);
    }

    // Try ElementData (dispatches across all element types)
    if let Ok(listeners) = with_element_data(target, |el| el.get_event_listeners(&event_type), "not an element") {
        if let Some(listeners) = listeners {
            all_listeners.extend(listeners);
        }
    }

    if all_listeners.is_empty() {
        Ok(None)
    } else {
        Ok(Some(all_listeners))
    }
}

/// Call a listener function
fn call_listener(listener: &JsValue, event: &JsObject, context: &mut Context) -> JsResult<()> {
    if let Some(callable) = listener.as_callable() {
        // Call with event as both this and argument (common pattern)
        let _ = callable.call(&event.clone().into(), &[event.clone().into()], context);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_propagation_path() {
        // Test that propagation path is built correctly
        // This would require setting up mock elements with parent relationships
    }
}
