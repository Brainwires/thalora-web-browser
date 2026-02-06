//! Focus Management System
//!
//! Implements proper focus tracking for DOM elements according to the HTML specification.
//! This manages the currently focused element (document.activeElement) and handles
//! focus/blur events with proper event propagation.
//!
//! https://html.spec.whatwg.org/multipage/interaction.html#focus

use boa_engine::{
    js_string,
    object::JsObject,
    Context, JsResult, JsValue,
};
use std::cell::RefCell;

use crate::dom::element::ElementData;
use crate::events::event::EventData;
use crate::events::propagation::dispatch_event_with_propagation;
use crate::events::ui_events::FocusEventData;

// Use thread-local storage for the focus manager since JsObject is not Send+Sync
thread_local! {
    /// The currently active (focused) element stored per-thread
    static ACTIVE_ELEMENT: RefCell<Option<JsObject>> = const { RefCell::new(None) };
}

/// Focus Manager - provides static methods for focus tracking
/// Uses thread-local storage since this runs in single-threaded JS context
pub struct FocusManager;

impl FocusManager {
    /// Get the currently active element
    pub fn get_active_element() -> Option<JsObject> {
        ACTIVE_ELEMENT.with(|cell| cell.borrow().clone())
    }

    /// Set the active element directly (internal use)
    pub fn set_active_element(element: Option<JsObject>) {
        ACTIVE_ELEMENT.with(|cell| {
            *cell.borrow_mut() = element;
        });
    }

    /// Check if a specific element is the currently active element
    pub fn is_active(element: &JsObject) -> bool {
        if let Some(active) = Self::get_active_element() {
            JsObject::equals(&active, element)
        } else {
            false
        }
    }

    /// Clear focus (no element focused)
    pub fn clear() {
        ACTIVE_ELEMENT.with(|cell| {
            *cell.borrow_mut() = None;
        });
    }
}

/// Get the focus manager (for API compatibility, returns static reference)
pub fn get_focus_manager() -> &'static FocusManager {
    // Return a static reference - all operations are actually thread-local
    static MANAGER: FocusManager = FocusManager;
    &MANAGER
}

/// Focus an element, properly dispatching blur on previous and focus on new element
///
/// This follows the HTML specification for focus processing:
/// 1. If there's a currently focused element, blur it (dispatch blur, then focusout)
/// 2. Set the new element as active
/// 3. Dispatch focus on the new element (focus, then focusin)
///
/// Focus and blur events don't bubble, but focusin and focusout do.
pub fn focus_element(element: &JsObject, context: &mut Context) -> JsResult<()> {
    // Check if this element is already focused
    if FocusManager::is_active(element) {
        return Ok(());
    }

    // Get the previous active element
    let prev_active = FocusManager::get_active_element();

    // 1. Blur the previous element if there was one
    if let Some(prev_element) = prev_active {
        // Only blur if it's a different element
        if !JsObject::equals(&prev_element, element) {
            // Dispatch 'blur' event (doesn't bubble)
            dispatch_focus_event(&prev_element, "blur", false, Some(element.clone()), context)?;

            // Dispatch 'focusout' event (bubbles)
            dispatch_focus_event(&prev_element, "focusout", true, Some(element.clone()), context)?;
        }
    }

    // 2. Set the new active element
    FocusManager::set_active_element(Some(element.clone()));

    // 3. Dispatch focus events on the new element
    // Dispatch 'focus' event (doesn't bubble)
    dispatch_focus_event(element, "focus", false, None, context)?;

    // Dispatch 'focusin' event (bubbles)
    dispatch_focus_event(element, "focusin", true, None, context)?;

    Ok(())
}

/// Blur an element, removing focus from it
pub fn blur_element(element: &JsObject, context: &mut Context) -> JsResult<()> {
    // Only blur if this element is currently focused
    if !FocusManager::is_active(element) {
        return Ok(());
    }

    // Dispatch 'blur' event (doesn't bubble)
    dispatch_focus_event(element, "blur", false, None, context)?;

    // Dispatch 'focusout' event (bubbles)
    dispatch_focus_event(element, "focusout", true, None, context)?;

    // Clear the active element
    FocusManager::clear();

    Ok(())
}

/// Dispatch a focus-related event (focus, blur, focusin, focusout)
fn dispatch_focus_event(
    target: &JsObject,
    event_type: &str,
    bubbles: bool,
    related_target: Option<JsObject>,
    context: &mut Context,
) -> JsResult<bool> {
    use crate::events::ui_events::UIEventData;

    // Create FocusEvent
    let mut event_base = EventData::new(event_type.to_string(), bubbles, false); // Focus events are not cancelable
    event_base.set_target(Some(target.clone()));

    let mut ui_event = UIEventData::new(event_type.to_string(), bubbles, false);
    ui_event.event = event_base;

    let focus_event_data = FocusEventData {
        ui_event,
        related_target,
    };

    // Create JsObject from FocusEventData
    let focus_event_proto = context.intrinsics().constructors().focus_event().prototype();
    let focus_event = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        focus_event_proto,
        focus_event_data,
    );

    // For non-bubbling events (focus, blur), we still want to call listeners on the target
    // but skip the bubbling phase. The propagation system handles this via the bubbles flag.
    dispatch_event_with_propagation(&focus_event.upcast(), target, context)
}

/// Check if an element is focusable
///
/// Elements are focusable if they are:
/// - Input, textarea, select, button elements (not disabled)
/// - Anchor elements with href attribute
/// - Elements with tabindex attribute
/// - Elements with contenteditable attribute
pub fn is_focusable(element: &JsObject) -> bool {
    if let Some(element_data) = element.downcast_ref::<ElementData>() {
        let tag_name = element_data.get_tag_name().to_uppercase();

        // Check if element is disabled
        if element_data.get_attribute("disabled").is_some() {
            return false;
        }

        // Inherently focusable elements
        match tag_name.as_str() {
            "INPUT" | "TEXTAREA" | "SELECT" | "BUTTON" => return true,
            "A" => {
                // Anchor is focusable if it has href
                return element_data.get_attribute("href").is_some();
            }
            _ => {}
        }

        // Check for tabindex attribute (makes any element focusable)
        if element_data.get_attribute("tabindex").is_some() {
            return true;
        }

        // Check for contenteditable
        if let Some(editable) = element_data.get_attribute("contenteditable") {
            if editable == "true" || editable.is_empty() {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_manager_singleton() {
        let fm1 = get_focus_manager();
        let fm2 = get_focus_manager();
        // Should be the same instance
        assert!(std::ptr::eq(fm1, fm2));
    }
}
