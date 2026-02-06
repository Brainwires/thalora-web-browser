//! querySelector/All, closest, matches, event listeners, focus/blur/click,
//! attachShadow, and form helpers

use boa_engine::{
    builtins::{BuiltInBuilder, array::Array},
    object::JsObject,
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
    JsString,
};

use super::types::ElementData;
use super::helpers::{with_element_data, has_element_data};
use crate::events::propagation::dispatch_event_with_propagation;

/// `Element.prototype.closest(selector)`
pub(super) fn closest_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.closest called on non-object")
    })?;

    let selector = args.get_or_undefined(0).to_string(context)?;
    let selector_str = selector.to_std_string_escaped();

    let result = with_element_data(&this_obj, |el| {
        el.find_closest(&selector_str, &this_obj)
    }, "Element.prototype.closest called on non-Element object")?;

    if let Some(found) = result {
        Ok(found.into())
    } else {
        Ok(JsValue::null())
    }
}

/// `Element.prototype.matches(selector)`
pub(super) fn matches_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.matches called on non-object")
    })?;

    let selector = args.get_or_undefined(0).to_string(context)?;
    let selector_str = selector.to_std_string_escaped();

    let matches = with_element_data(&this_obj, |el| {
        el.matches_selector(&selector_str)
    }, "Element.prototype.matches called on non-Element object")?;

    Ok(matches.into())
}

/// `Element.prototype.querySelector(selector)` - find first matching descendant
pub(super) fn query_selector_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.querySelector called on non-object")
    })?;

    let selector = args.get_or_undefined(0).to_string(context)?;
    let selector_str = selector.to_std_string_escaped();

    // Use the element's query_selector method
    let result = with_element_data(&this_obj, |el| {
        el.query_selector(&selector_str)
    }, "Element.prototype.querySelector called on non-Element object")?;

    if let Some(found) = result {
        Ok(found.into())
    } else {
        Ok(JsValue::null())
    }
}

/// `Element.prototype.querySelectorAll(selector)` - find all matching descendants
pub(super) fn query_selector_all_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.querySelectorAll called on non-object")
    })?;

    let selector = args.get_or_undefined(0).to_string(context)?;
    let selector_str = selector.to_std_string_escaped();

    // Use the element's query_selector_all method
    let results = with_element_data(&this_obj, |el| {
        el.query_selector_all(&selector_str)
    }, "Element.prototype.querySelectorAll called on non-Element object")?;

    // Convert to JS array
    let array = Array::create_array_from_list(
        results.into_iter().map(|obj| obj.into()).collect::<Vec<_>>(),
        context,
    );
    Ok(array.into())
}

/// `Element.prototype.addEventListener(type, listener[, options])`
/// JavaScript wrapper for EventTarget functionality
pub(super) fn add_event_listener_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.addEventListener called on non-object")
    })?;

    let event_type = args.get_or_undefined(0).to_string(context)?;
    let listener = args.get_or_undefined(1);

    with_element_data(&this_obj, |el| {
        el.add_event_listener(event_type.to_std_string_escaped(), listener.clone());
    }, "Element.prototype.addEventListener called on non-Element object")?;

    Ok(JsValue::undefined())
}

/// `Element.prototype.removeEventListener(type, listener[, options])`
/// JavaScript wrapper for EventTarget functionality
pub(super) fn remove_event_listener_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.removeEventListener called on non-object")
    })?;

    let event_type = args.get_or_undefined(0).to_string(context)?;
    let listener = args.get_or_undefined(1);

    with_element_data(&this_obj, |el| {
        el.remove_event_listener(&event_type.to_std_string_escaped(), &listener);
    }, "Element.prototype.removeEventListener called on non-Element object")?;

    Ok(JsValue::undefined())
}

/// `Element.prototype.dispatchEvent(event)`
/// JavaScript wrapper for EventTarget functionality with full event propagation
pub(super) fn dispatch_event_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.dispatchEvent called on non-object")
    })?;

    // Verify this is an Element
    if !has_element_data(&this_obj) {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.dispatchEvent called on non-Element object")
            .into());
    }

    let event = args.get_or_undefined(0);

    // Get event object
    if event.is_object() {
        if let Some(event_obj) = event.as_object() {
            // Get the 'type' property from the event object to validate it's an event
            let event_type_value = event_obj.get(js_string!("type"), context)
                .unwrap_or(JsValue::undefined());

            if !event_type_value.is_undefined() {
                // Use full event propagation system (capturing, at-target, bubbling)
                let result = dispatch_event_with_propagation(&event_obj, &this_obj, context)?;
                Ok(JsValue::from(result))
            } else {
                Err(JsNativeError::typ()
                    .with_message("Event object must have a 'type' property")
                    .into())
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("dispatchEvent requires an Event object")
                .into())
        }
    } else {
        Err(JsNativeError::typ()
            .with_message("dispatchEvent requires an Event object")
            .into())
    }
}

/// `Element.prototype.focus()`
/// Focuses the element and dispatches appropriate focus events
pub(super) fn focus(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.focus called on non-object")
    })?;

    // Verify it's an element
    if !has_element_data(&this_obj) {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.focus called on non-Element object")
            .into());
    }

    // Use the focus manager to properly handle focus with events
    crate::browser::focus_manager::focus_element(&this_obj, context)?;

    Ok(JsValue::undefined())
}

/// `Element.prototype.blur()`
/// Blurs (unfocuses) the element and dispatches appropriate blur events
pub(super) fn blur(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.blur called on non-object")
    })?;

    // Verify it's an element
    if !has_element_data(&this_obj) {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.blur called on non-Element object")
            .into());
    }

    // Use the focus manager to properly handle blur with events
    crate::browser::focus_manager::blur_element(&this_obj, context)?;

    Ok(JsValue::undefined())
}

/// `Element.prototype.click()`
/// Simulates a mouse click on the element with full event propagation
pub(super) fn click(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.click called on non-object")
    })?;

    // Create a proper MouseEvent using the MouseEventData structure
    use crate::events::event::EventData;
    use crate::events::ui_events::{MouseEventData, UIEventData};

    // Create trusted MouseEvent (click events from element.click() are trusted)
    let mut event_base = EventData::new_trusted("click".to_string(), true, true);
    event_base.set_target(Some(this_obj.clone()));

    let mut ui_event = UIEventData::new("click".to_string(), true, true);
    ui_event.event = event_base;
    ui_event.detail = 1; // Click count

    let mouse_event_data = MouseEventData {
        ui_event,
        screen_x: 0.0,
        screen_y: 0.0,
        client_x: 0.0,
        client_y: 0.0,
        page_x: 0.0,
        page_y: 0.0,
        offset_x: 0.0,
        offset_y: 0.0,
        movement_x: 0.0,
        movement_y: 0.0,
        ctrl_key: false,
        shift_key: false,
        alt_key: false,
        meta_key: false,
        button: 0,
        buttons: 1,
        related_target: None,
    };

    // Create JsObject from MouseEventData
    let mouse_event_proto = context.intrinsics().constructors().mouse_event().prototype();
    let click_event = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        mouse_event_proto,
        mouse_event_data,
    );

    // Dispatch with full propagation (capturing, at-target, bubbling)
    let default_not_prevented = dispatch_event_with_propagation(&click_event.upcast(), &this_obj, context)?;

    // Handle default action if not prevented
    // Extract needed data from element inside with_element_data to avoid holding borrow
    if default_not_prevented {
        let (tag_name, href, button_type) = with_element_data(&this_obj, |el| {
            let tag = el.get_tag_name().to_uppercase();
            let href = el.get_attribute("href");
            let btype = el.get_attribute("type").unwrap_or_default().to_lowercase();
            (tag, href, btype)
        }, "Element.prototype.click called on non-Element object")?;

        match tag_name.as_str() {
            "A" => {
                // For anchor elements, queue navigation
                if let Some(href) = href {
                    // Queue navigation through the browser bridge
                    crate::browser::navigation_bridge::queue_navigation(&href);
                }
            }
            "BUTTON" | "INPUT" => {
                if button_type == "submit" {
                    // Find and submit parent form
                    if let Some(form) = find_parent_form(&this_obj) {
                        submit_form(&form, context)?;
                    }
                }
            }
            _ => {}
        }
    }

    Ok(JsValue::undefined())
}

/// Find the parent form element for a given element
fn find_parent_form(element: &JsObject) -> Option<JsObject> {
    let mut current = with_element_data(element, |el| {
        el.get_parent_node()
    }, "").ok()?;

    while let Some(parent) = current {
        // Check if this parent is a FORM element
        let is_form = with_element_data(&parent, |el| {
            el.get_tag_name().to_uppercase() == "FORM"
        }, "").unwrap_or(false);

        if is_form {
            return Some(parent);
        }

        // Get next parent
        current = with_element_data(&parent, |el| {
            el.get_parent_node()
        }, "").ok()?;
    }

    None
}

/// Submit a form element
fn submit_form(form: &JsObject, context: &mut Context) -> JsResult<()> {
    use crate::events::event::EventData;

    // Create submit event
    let submit_event_data = EventData::new("submit".to_string(), true, true);
    let submit_event_proto = context.intrinsics().constructors().event().prototype();
    let submit_event = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        submit_event_proto,
        submit_event_data,
    );

    // Dispatch submit event on the form with propagation
    let default_not_prevented = dispatch_event_with_propagation(&submit_event.upcast(), form, context)?;

    // If not prevented, queue form submission
    if default_not_prevented {
        let form_info = with_element_data(form, |el| {
            let action = el.get_attribute("action");
            let method = el.get_attribute("method").unwrap_or_else(|| "GET".to_string());
            (action, method)
        }, "submit_form called on non-Element object")?;

        if let Some(action) = form_info.0 {
            crate::browser::navigation_bridge::queue_form_submission(&action, &form_info.1);
        }
    }

    Ok(())
}

// ============================================================================
// Shadow DOM: attachShadow and helpers
// ============================================================================

/// `Element.prototype.attachShadow(options)` - Shadow DOM API
/// Check if an element can have a shadow root attached according to WHATWG spec
/// https://dom.spec.whatwg.org/#dom-element-attachshadow
pub fn can_have_shadow_root(element: &ElementData) -> bool {
    let tag_name = element.get_tag_name().to_lowercase();
    let namespace = element.get_namespace_uri().unwrap_or_default();

    // Per WHATWG spec, only these elements can have shadow roots attached:

    // 1. HTML namespace elements that are valid shadow hosts
    if namespace == "http://www.w3.org/1999/xhtml" || namespace.is_empty() {
        return match tag_name.as_str() {
            // Custom elements (any element with a hyphen in the name)
            name if name.contains('-') => true,

            // Standard HTML elements that can host shadow roots
            "article" | "aside" | "blockquote" | "body" | "div" |
            "footer" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" |
            "header" | "main" | "nav" | "p" | "section" | "span" => true,

            // Form elements that can host shadow roots
            "form" | "fieldset" => true,

            // Other valid shadow hosts
            "details" | "dialog" => true,

            // All other HTML elements cannot host shadow roots
            _ => false,
        };
    }

    // 2. Elements in other namespaces
    // Per spec, elements in non-HTML namespaces can also be shadow hosts
    // if they are valid custom elements or meet certain criteria
    if namespace == "http://www.w3.org/2000/svg" {
        // SVG elements that can be shadow hosts
        return match tag_name.as_str() {
            "g" | "svg" | "foreignObject" => true,
            name if name.contains('-') => true, // Custom SVG elements
            _ => false,
        };
    }

    // Elements in other namespaces can be shadow hosts if they're custom elements
    tag_name.contains('-')
}

/// Check if element has forbidden shadow root characteristics
fn has_forbidden_shadow_characteristics(element: &ElementData) -> bool {
    let tag_name = element.get_tag_name().to_lowercase();

    // Elements that must never have shadow roots for security/functionality reasons
    match tag_name.as_str() {
        // Form controls that have special UA behavior
        "input" | "textarea" | "select" | "button" => true,

        // Media elements with special UA behavior
        "audio" | "video" | "img" | "canvas" => true,

        // Elements that affect document structure
        "html" | "head" | "title" | "meta" | "link" | "style" | "script" => true,

        // Interactive elements that could cause security issues
        "a" | "area" | "iframe" | "object" | "embed" => true,

        // Table elements with complex UA behavior
        "table" | "thead" | "tbody" | "tfoot" | "tr" | "td" | "th" |
        "col" | "colgroup" | "caption" => true,

        // List elements
        "ol" | "ul" | "li" | "dl" | "dt" | "dd" => true,

        // Other elements with special semantics
        "option" | "optgroup" | "legend" | "label" => true,

        _ => false,
    }
}

/// Check if element is a valid custom element name
fn is_valid_custom_element_name(name: &str) -> bool {
    // Per WHATWG spec, custom element names must:
    // 1. Contain a hyphen
    // 2. Start with lowercase ASCII letter
    // 3. Contain only lowercase ASCII letters, digits, hyphens, periods, underscores
    // 4. Not be one of the reserved names

    if !name.contains('-') {
        return false;
    }

    let first_char = name.chars().next().unwrap_or('\0');
    if !first_char.is_ascii_lowercase() {
        return false;
    }

    if !name.chars().all(|c| {
        c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.' || c == '_'
    }) {
        return false;
    }

    // Reserved names that cannot be custom elements
    const RESERVED_NAMES: &[&str] = &[
        "annotation-xml",
        "color-profile",
        "font-face",
        "font-face-src",
        "font-face-uri",
        "font-face-format",
        "font-face-name",
        "missing-glyph",
    ];

    !RESERVED_NAMES.contains(&name)
}

pub(super) fn attach_shadow(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.attachShadow called on non-object")
    })?;

    // First, check if this is an ElementData and perform validation
    let (shadow_init, has_shadow_root, can_have_shadow) = {
        let options = args.get_or_undefined(0);

        // Parse options object according to WHATWG spec
        let shadow_init = if let Some(options_obj) = options.as_object() {
            let mode = if let Ok(mode_value) = options_obj.get(js_string!("mode"), context) {
                let mode_str = mode_value.to_string(context)?.to_std_string_escaped();
                crate::dom::shadow::shadow_root::ShadowRootMode::from_string(&mode_str)
                    .ok_or_else(|| JsNativeError::typ()
                        .with_message("attachShadow mode must be 'open' or 'closed'"))?
            } else {
                return Err(JsNativeError::typ()
                    .with_message("attachShadow options must include a mode")
                    .into());
            };

            let clonable = if let Ok(clonable_value) = options_obj.get(js_string!("clonable"), context) {
                clonable_value.to_boolean()
            } else {
                false
            };

            let serializable = if let Ok(serializable_value) = options_obj.get(js_string!("serializable"), context) {
                serializable_value.to_boolean()
            } else {
                false
            };

            let delegates_focus = if let Ok(delegates_focus_value) = options_obj.get(js_string!("delegatesFocus"), context) {
                delegates_focus_value.to_boolean()
            } else {
                false
            };

            crate::dom::shadow::shadow_root::ShadowRootInit {
                mode,
                clonable,
                serializable,
                delegates_focus,
            }
        } else {
            return Err(JsNativeError::typ()
                .with_message("attachShadow requires an options object")
                .into());
        };

        // Check if element already has a shadow root and validate element
        let (has_shadow_root, can_have_shadow) = with_element_data(&this_obj, |el| {
            let has_sr = el.get_shadow_root().is_some();
            let can_shadow = can_have_shadow_root(el);
            (has_sr, can_shadow)
        }, "Element.prototype.attachShadow called on non-Element object")?;

        (shadow_init, has_shadow_root, can_have_shadow)
    }; // Release the borrow here

    // Now perform validation without holding any borrows
    if has_shadow_root {
        return Err(JsNativeError::error()
            .with_message("Element already has a shadow root")
            .into());
    }

    if !can_have_shadow {
        return Err(JsNativeError::error()
            .with_message("Operation not supported")
            .into());
    }

    // Create a proper ShadowRoot using the new implementation
    let shadow_root = crate::dom::shadow::shadow_root::ShadowRoot::create_shadow_root(
        shadow_init.mode.clone(),
        &shadow_init,
        context,
    )?;

    // Set the host element for the shadow root
    if let Some(shadow_data) = shadow_root.downcast_ref::<crate::dom::shadow::shadow_root::ShadowRootData>() {
        shadow_data.set_host(this_obj.clone());
    }

    // Set shadowRoot property on the element according to mode
    match shadow_init.mode {
        crate::dom::shadow::shadow_root::ShadowRootMode::Open => {
            this_obj.define_property_or_throw(
                js_string!("shadowRoot"),
                boa_engine::property::PropertyDescriptorBuilder::new()
                    .value(shadow_root.clone())
                    .writable(false)
                    .enumerable(false)
                    .configurable(true)
                    .build(),
                context,
            )?;
        }
        crate::dom::shadow::shadow_root::ShadowRootMode::Closed => {
            // For 'closed' mode, shadowRoot property should be null
            this_obj.define_property_or_throw(
                js_string!("shadowRoot"),
                boa_engine::property::PropertyDescriptorBuilder::new()
                    .value(JsValue::null())
                    .writable(false)
                    .enumerable(false)
                    .configurable(true)
                    .build(),
                context,
            )?;
        }
    }

    // Store the shadow root internally in element data (get a fresh borrow)
    with_element_data(&this_obj, |el| {
        el.attach_shadow_root(shadow_root.clone());
    }, "Element.prototype.attachShadow called on non-Element object")?;

    Ok(shadow_root.into())
}
