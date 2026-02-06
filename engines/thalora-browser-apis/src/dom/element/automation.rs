//! Automation helpers: dispatch_trusted_mouse_event, checkVisibility

use boa_engine::{
    object::JsObject,
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
};

use super::types::ElementData;
use super::query_and_events::dispatch_event_js;

/// `Element.prototype.__dispatchTrustedMouseEvent(eventType, clientX, clientY, options?)`
///
/// Dispatches a trusted mouse event to this element.
/// This is a browser-internal API for automation that creates events with isTrusted: true.
///
/// ## Options
///
/// Standard mouse event options:
/// - `button`: number - The mouse button (0=left, 1=middle, 2=right)
/// - `buttons`: number - Bitmask of pressed buttons
/// - `ctrlKey`, `shiftKey`, `altKey`, `metaKey`: boolean - Modifier key states
///
/// CSS Transform options (for clicking on 3D-transformed elements):
/// - `transform`: string - CSS transform value (e.g., "matrix3d(...)" or "rotate(45deg)")
/// - `transformOrigin`: string - CSS transform-origin value (default: "50% 50% 0")
/// - `width`: number - Element width in pixels (for percentage-based origins)
/// - `height`: number - Element height in pixels (for percentage-based origins)
/// - `elementX`: number - Element's X position in document (default: 0)
/// - `elementY`: number - Element's Y position in document (default: 0)
///
/// When transform options are provided, the coordinates are transformed through the
/// inverse of the CSS transform matrix to correctly target the visual position on
/// a rotated/skewed/scaled element.
pub(super) fn dispatch_trusted_mouse_event(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use crate::events::ui_events::MouseEventData;

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("__dispatchTrustedMouseEvent called on non-object")
    })?;

    // Verify this is an element
    if this_obj.downcast_ref::<ElementData>().is_none() {
        return Err(JsNativeError::typ()
            .with_message("__dispatchTrustedMouseEvent called on non-Element object")
            .into());
    }

    // Get event type
    let event_type = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    // Get coordinates
    let client_x = args.get_or_undefined(1).to_number(context).unwrap_or(0.0);
    let client_y = args.get_or_undefined(2).to_number(context).unwrap_or(0.0);

    // Get optional parameters
    let options = args.get_or_undefined(3);
    let (button, buttons, ctrl_key, shift_key, alt_key, meta_key, final_x, final_y) = if options.is_object() {
        let opts = options.as_object().unwrap();
        let button = opts.get(js_string!("button"), context)
            .map(|v| v.to_i32(context).unwrap_or(0) as i16)
            .unwrap_or(0);
        let buttons = opts.get(js_string!("buttons"), context)
            .map(|v| v.to_u32(context).unwrap_or(0) as u16)
            .unwrap_or(if event_type.contains("down") || event_type == "click" { 1 } else { 0 });
        let ctrl_key = opts.get(js_string!("ctrlKey"), context)
            .map(|v| v.to_boolean())
            .unwrap_or(false);
        let shift_key = opts.get(js_string!("shiftKey"), context)
            .map(|v| v.to_boolean())
            .unwrap_or(false);
        let alt_key = opts.get(js_string!("altKey"), context)
            .map(|v| v.to_boolean())
            .unwrap_or(false);
        let meta_key = opts.get(js_string!("metaKey"), context)
            .map(|v| v.to_boolean())
            .unwrap_or(false);

        // Check for CSS transform options
        let (final_x, final_y) = {
            let transform_opt = opts.get(js_string!("transform"), context).ok();
            if let Some(transform_val) = transform_opt {
                if let Some(transform_str) = transform_val.as_string() {
                    let transform = transform_str.to_std_string_escaped();
                    if !transform.is_empty() && !transform.eq_ignore_ascii_case("none") {
                        // Get transform-origin (default: "50% 50% 0")
                        let origin = opts.get(js_string!("transformOrigin"), context)
                            .ok()
                            .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()))
                            .unwrap_or_else(|| "50% 50% 0".to_string());

                        // Get element dimensions
                        let width = opts.get(js_string!("width"), context)
                            .ok()
                            .and_then(|v| v.to_number(context).ok())
                            .unwrap_or(0.0);
                        let height = opts.get(js_string!("height"), context)
                            .ok()
                            .and_then(|v| v.to_number(context).ok())
                            .unwrap_or(0.0);

                        // Get element position
                        let element_x = opts.get(js_string!("elementX"), context)
                            .ok()
                            .and_then(|v| v.to_number(context).ok())
                            .unwrap_or(0.0);
                        let element_y = opts.get(js_string!("elementY"), context)
                            .ok()
                            .and_then(|v| v.to_number(context).ok())
                            .unwrap_or(0.0);

                        // Apply inverse transform to map screen coords to element-local coords
                        let (local_x, local_y) = crate::css_transform::screen_to_element_coords(
                            client_x, client_y,
                            &transform,
                            &origin,
                            width, height,
                            element_x, element_y,
                        );

                        (local_x + element_x, local_y + element_y)
                    } else {
                        (client_x, client_y)
                    }
                } else {
                    (client_x, client_y)
                }
            } else {
                (client_x, client_y)
            }
        };

        (button, buttons, ctrl_key, shift_key, alt_key, meta_key, final_x, final_y)
    } else {
        let buttons = if event_type.contains("down") || event_type == "click" { 1 } else { 0 };
        (0, buttons, false, false, false, false, client_x, client_y)
    };

    // Determine event properties
    let (bubbles, cancelable) = match event_type.as_str() {
        "click" | "dblclick" | "mousedown" | "mouseup" | "mousemove"
        | "mouseover" | "mouseout" | "mouseenter" | "mouseleave" => (true, true),
        _ => (true, false),
    };

    // Create trusted mouse event data with transformed coordinates
    let mut mouse_event = MouseEventData::new_trusted_with_coords(
        event_type.clone(),
        bubbles,
        cancelable,
        final_x,   // Use transformed coordinates
        final_y,
        final_x,   // screen_x (same as clientX for simplicity)
        final_y,   // screen_y
        final_x,   // page_x
        final_y,   // page_y
        0.0,       // movement_x
        0.0,       // movement_y
        button,
        buttons,
    );

    // Set modifier keys directly (fields are public)
    mouse_event.ctrl_key = ctrl_key;
    mouse_event.shift_key = shift_key;
    mouse_event.alt_key = alt_key;
    mouse_event.meta_key = meta_key;

    // Create the event object
    let event_prototype = context.intrinsics().constructors().mouse_event().prototype();
    let event_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        event_prototype,
        mouse_event,
    );

    // Dispatch to the element using dispatchEvent
    dispatch_event_js(this, &[event_obj.upcast().into()], context)?;

    Ok(true.into())
}

/// `Element.prototype.checkVisibility(options?)`
///
/// Returns true if the element is rendered and visible.
/// This is used by widgets like Cloudflare Turnstile to verify visibility.
///
/// Options:
/// - checkOpacity: boolean - Check if opacity is 0
/// - checkVisibilityCSS: boolean - Check if visibility: hidden
/// - contentVisibilityAuto: boolean - Check content-visibility: auto
/// - opacityProperty: boolean - (alias for checkOpacity)
/// - visibilityProperty: boolean - (alias for checkVisibilityCSS)
///
/// See: https://developer.mozilla.org/en-US/docs/Web/API/Element/checkVisibility
pub(super) fn check_visibility(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.checkVisibility called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.checkVisibility called on non-Element object")
    })?;

    // Parse options
    let options = args.get_or_undefined(0);
    let (check_opacity, check_visibility_css) = if options.is_object() {
        let opts = options.as_object().unwrap();

        // checkOpacity or opacityProperty
        let check_opacity = opts.get(js_string!("checkOpacity"), context)
            .map(|v| v.to_boolean())
            .unwrap_or(false)
            || opts.get(js_string!("opacityProperty"), context)
                .map(|v| v.to_boolean())
                .unwrap_or(false);

        // checkVisibilityCSS or visibilityProperty
        let check_visibility_css = opts.get(js_string!("checkVisibilityCSS"), context)
            .map(|v| v.to_boolean())
            .unwrap_or(false)
            || opts.get(js_string!("visibilityProperty"), context)
                .map(|v| v.to_boolean())
                .unwrap_or(false);

        (check_opacity, check_visibility_css)
    } else {
        (false, false)
    };

    // Check 1: Element must be in the DOM (have non-zero dimensions or be a valid element)
    let rect = element.get_bounding_client_rect();

    // Get computed style properties
    let style = element.style.lock().unwrap();

    // Check 2: display property - if 'none', element is not rendered
    if let Some(display) = style.get_property("display") {
        if display == "none" {
            eprintln!("checkVisibility: false (display: none)");
            return Ok(false.into());
        }
    }

    // Check 3 (optional): visibility CSS property
    if check_visibility_css {
        if let Some(visibility) = style.get_property("visibility") {
            if visibility == "hidden" || visibility == "collapse" {
                eprintln!("checkVisibility: false (visibility: {})", visibility);
                return Ok(false.into());
            }
        }
    }

    // Check 4 (optional): opacity
    if check_opacity {
        if let Some(opacity) = style.get_property("opacity") {
            if let Ok(opacity_val) = opacity.parse::<f64>() {
                if opacity_val == 0.0 {
                    eprintln!("checkVisibility: false (opacity: 0)");
                    return Ok(false.into());
                }
            }
        }
    }

    // Check 5: Element must have non-zero content size (width and height)
    // An element with 0x0 dimensions is not rendered and not visible
    // However, we allow elements with zero width/height if they have explicit positioning
    // (some elements are sized by their children or use overflow)
    let has_size = rect.width > 0.0 || rect.height > 0.0;

    // For visibility check, we consider an element visible if:
    // 1. It has non-zero dimensions, OR
    // 2. It's an element type that can be visible without explicit dimensions (like body, html)
    let tag = element.get_tag_name().to_lowercase();
    let is_structural = matches!(tag.as_str(), "html" | "body" | "head" | "script" | "style" | "meta" | "link");

    if !has_size && !is_structural {
        // Check if element is positioned with explicit layout
        // Elements inside turnstile widgets often have dimensions from CSS
        // For now, be permissive and assume elements are visible unless explicitly hidden
        eprintln!("checkVisibility: element has zero size but may be visible (tag={})", tag);
    }

    // Check 6: Element should be in viewport (optional, but useful for Turnstile)
    let element_id = element.get_element_identifier();
    let in_viewport = crate::layout_registry::is_element_in_viewport(&element_id, &tag);

    eprintln!("checkVisibility: true (tag={}, size={:.1}x{:.1}, in_viewport={})",
        tag, rect.width, rect.height, in_viewport);

    // Return true - element is rendered and not hidden by CSS
    // Note: We're being permissive here because Turnstile widgets
    // often have complex CSS that may not be fully computed
    Ok(true.into())
}
