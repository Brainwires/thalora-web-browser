//! Layout dimension getters/setters (offset*, client*, scroll*, getBoundingClientRect, scrollTo)

use boa_engine::{
    builtins::BuiltInBuilder,
    object::JsObject,
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
};

use super::helpers::with_element_data;

// =============================================================================
// Layout dimension getters (read-only properties)
// =============================================================================

/// `Element.prototype.offsetWidth` - returns layout width including borders
#[allow(unused_variables)]
pub(super) fn get_offset_width(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("offsetWidth getter called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_offset_width(), "offsetWidth called on non-Element").unwrap_or(0.0);
    Ok(JsValue::from(value as i32))
}

/// `Element.prototype.offsetHeight` - returns layout height including borders
#[allow(unused_variables)]
pub(super) fn get_offset_height(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("offsetHeight getter called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_offset_height(), "offsetHeight called on non-Element").unwrap_or(0.0);
    Ok(JsValue::from(value as i32))
}

/// `Element.prototype.offsetTop` - returns top offset from offsetParent
#[allow(unused_variables)]
pub(super) fn get_offset_top(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("offsetTop getter called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_offset_top(), "offsetTop called on non-Element").unwrap_or(0.0);
    Ok(JsValue::from(value as i32))
}

/// `Element.prototype.offsetLeft` - returns left offset from offsetParent
#[allow(unused_variables)]
pub(super) fn get_offset_left(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("offsetLeft getter called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_offset_left(), "offsetLeft called on non-Element").unwrap_or(0.0);
    Ok(JsValue::from(value as i32))
}

/// `Element.prototype.offsetParent` - returns nearest positioned ancestor
#[allow(unused_variables)]
pub(super) fn get_offset_parent(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("offsetParent getter called on non-object")
    })?;

    // For now return null - proper implementation would walk up DOM tree
    // to find positioned ancestor
    let parent = with_element_data(&this_obj, |el| el.get_parent_node(), "offsetParent called on non-Element")
        .unwrap_or(None);

    match parent {
        Some(p) => Ok(JsValue::from(p)),
        None => Ok(JsValue::null()),
    }
}

/// `Element.prototype.clientWidth` - returns inner width (excluding borders, scrollbar)
#[allow(unused_variables)]
pub(super) fn get_client_width(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("clientWidth getter called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_client_width(), "clientWidth called on non-Element").unwrap_or(0.0);
    Ok(JsValue::from(value as i32))
}

/// `Element.prototype.clientHeight` - returns inner height (excluding borders, scrollbar)
#[allow(unused_variables)]
pub(super) fn get_client_height(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("clientHeight getter called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_client_height(), "clientHeight called on non-Element").unwrap_or(0.0);
    Ok(JsValue::from(value as i32))
}

/// `Element.prototype.clientTop` - returns top border width
#[allow(unused_variables)]
pub(super) fn get_client_top(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("clientTop getter called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_client_top(), "clientTop called on non-Element").unwrap_or(0.0);
    Ok(JsValue::from(value as i32))
}

/// `Element.prototype.clientLeft` - returns left border width
#[allow(unused_variables)]
pub(super) fn get_client_left(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("clientLeft getter called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_client_left(), "clientLeft called on non-Element").unwrap_or(0.0);
    Ok(JsValue::from(value as i32))
}

// =============================================================================
// Scroll dimension getters
// =============================================================================

/// `Element.prototype.scrollWidth` - returns total width of content (including overflow)
#[allow(unused_variables)]
pub(super) fn get_scroll_width(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("scrollWidth getter called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_scroll_width(), "scrollWidth called on non-Element").unwrap_or(0.0);
    Ok(JsValue::from(value as i32))
}

/// `Element.prototype.scrollHeight` - returns total height of content (including overflow)
#[allow(unused_variables)]
pub(super) fn get_scroll_height(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("scrollHeight getter called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_scroll_height(), "scrollHeight called on non-Element").unwrap_or(0.0);
    Ok(JsValue::from(value as i32))
}

// =============================================================================
// Scroll position getters and setters
// =============================================================================

/// `Element.prototype.scrollTop` getter - returns scroll position from top
#[allow(unused_variables)]
pub(super) fn get_scroll_top(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("scrollTop getter called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_scroll_top(), "scrollTop called on non-Element").unwrap_or(0.0);
    Ok(JsValue::from(value))
}

/// `Element.prototype.scrollTop` setter - sets scroll position from top
pub(super) fn set_scroll_top(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("scrollTop setter called on non-object")
    })?;

    let value = args.get_or_undefined(0).to_number(context).unwrap_or(0.0);

    let _ = with_element_data(&this_obj, |el| el.set_scroll_top(value), "scrollTop setter called on non-Element");

    Ok(JsValue::undefined())
}

/// `Element.prototype.scrollLeft` getter - returns scroll position from left
#[allow(unused_variables)]
pub(super) fn get_scroll_left(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("scrollLeft getter called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_scroll_left(), "scrollLeft called on non-Element").unwrap_or(0.0);
    Ok(JsValue::from(value))
}

/// `Element.prototype.scrollLeft` setter - sets scroll position from left
pub(super) fn set_scroll_left(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("scrollLeft setter called on non-object")
    })?;

    let value = args.get_or_undefined(0).to_number(context).unwrap_or(0.0);

    let _ = with_element_data(&this_obj, |el| el.set_scroll_left(value), "scrollLeft setter called on non-Element");

    Ok(JsValue::undefined())
}

/// `Element.prototype.scrollTo(x, y)` or `Element.prototype.scrollTo(options)`
/// Scrolls the element's content to the specified position
pub(super) fn scroll_to_element(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.scrollTo called on non-object")
    })?;

    // Parse arguments - supports both scrollTo(x, y) and scrollTo(options) forms
    let (x, y) = if args.len() >= 2 {
        // scrollTo(x, y) form
        let x = args.get_or_undefined(0).to_number(context).unwrap_or(0.0);
        let y = args.get_or_undefined(1).to_number(context).unwrap_or(0.0);
        (x, y)
    } else if let Some(options) = args.get(0).and_then(|v| v.as_object()) {
        // scrollTo(options) form
        let x = options.get(js_string!("left"), context)
            .ok()
            .and_then(|v| v.to_number(context).ok())
            .unwrap_or(0.0);
        let y = options.get(js_string!("top"), context)
            .ok()
            .and_then(|v| v.to_number(context).ok())
            .unwrap_or(0.0);
        (x, y)
    } else {
        (0.0, 0.0)
    };

    // Update the element's scroll position
    let _ = with_element_data(&this_obj, |el| {
        el.set_scroll_left(x);
        el.set_scroll_top(y);
    }, "Element.prototype.scrollTo called on non-Element");

    Ok(JsValue::undefined())
}

/// `Element.prototype.getBoundingClientRect()`
pub(super) fn get_bounding_client_rect_js(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.getBoundingClientRect called on non-object")
    })?;

    let rect = with_element_data(
        &this_obj,
        |el| el.get_bounding_client_rect(),
        "Element.prototype.getBoundingClientRect called on non-Element object",
    )?;

    // Create DOMRect object
    let rect_obj = JsObject::default(context.intrinsics());
    rect_obj.set(js_string!("x"), rect.x, false, context)?;
    rect_obj.set(js_string!("y"), rect.y, false, context)?;
    rect_obj.set(js_string!("width"), rect.width, false, context)?;
    rect_obj.set(js_string!("height"), rect.height, false, context)?;
    rect_obj.set(js_string!("top"), rect.top, false, context)?;
    rect_obj.set(js_string!("right"), rect.right, false, context)?;
    rect_obj.set(js_string!("bottom"), rect.bottom, false, context)?;
    rect_obj.set(js_string!("left"), rect.left, false, context)?;

    // Add toJSON method
    let to_json = BuiltInBuilder::callable(context.realm(), |this, _args, ctx| {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("toJSON called on non-object")
        })?;
        let result = JsObject::default(ctx.intrinsics());
        for prop in ["x", "y", "width", "height", "top", "right", "bottom", "left"] {
            if let Ok(val) = obj.get(js_string!(prop), ctx) {
                result.set(js_string!(prop), val, false, ctx)?;
            }
        }
        Ok(result.into())
    })
    .name(js_string!("toJSON"))
    .build();
    rect_obj.set(js_string!("toJSON"), to_json, false, context)?;

    Ok(rect_obj.into())
}

/// `Element.prototype.scrollIntoView(options)`
#[allow(unused_variables)]
pub(super) fn scroll_into_view(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.scrollIntoView called on non-object")
    })?;

    // Verify it's an element - propagate the error if not
    with_element_data(
        &this_obj,
        |_el| {},
        "Element.prototype.scrollIntoView called on non-Element object",
    )?;

    // In a headless browser, scrollIntoView is effectively a no-op
    // but we should still accept the call without error
    let _options = args.get_or_undefined(0);

    // No-op in headless mode

    Ok(JsValue::undefined())
}
