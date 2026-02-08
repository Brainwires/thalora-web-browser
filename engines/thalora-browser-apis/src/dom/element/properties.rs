//! Element property getters and setters (tagName, id, className, innerHTML, textContent, etc.)

use boa_engine::{
    builtins::{BuiltInBuilder, array::Array},
    object::JsObject,
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
    JsString,
};

use super::types::ElementData;
use super::helpers::{with_element_data, has_element_data};
use super::dom_manipulation::parse_html_elements_with_context;

/// `Element.prototype.tagName` getter
pub(super) fn get_tag_name(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.tagName called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_tag_name(), "Element.prototype.tagName called on non-Element object")?;
    Ok(JsString::from(value).into())
}

/// `Element.prototype.id` getter
pub(super) fn get_id(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.id called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_id(), "Element.prototype.id called on non-Element object")?;
    Ok(JsString::from(value).into())
}

/// `Element.prototype.id` setter
pub(super) fn set_id(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.id setter called on non-object")
    })?;

    let id = args.get_or_undefined(0).to_string(context)?;
    let id_string = id.to_std_string_escaped();

    with_element_data(&this_obj, |el| {
        el.set_id(id_string.clone());
        el.set_attribute("id".to_string(), id_string);
    }, "Element.prototype.id setter called on non-Element object")?;

    Ok(JsValue::undefined())
}

/// `Element.prototype.className` getter
pub(super) fn get_class_name(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.className called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_class_name(), "Element.prototype.className called on non-Element object")?;
    Ok(JsString::from(value).into())
}

/// `Element.prototype.className` setter
pub(super) fn set_class_name(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.className setter called on non-object")
    })?;

    let class_name = args.get_or_undefined(0).to_string(context)?;
    let class_name_string = class_name.to_std_string_escaped();

    with_element_data(&this_obj, |el| {
        el.set_class_name(class_name_string.clone());
        el.set_attribute("class".to_string(), class_name_string);
    }, "Element.prototype.className setter called on non-Element object")?;

    Ok(JsValue::undefined())
}

/// `Element.prototype.innerHTML` getter
pub(super) fn get_inner_html(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.innerHTML called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_inner_html(), "Element.prototype.innerHTML called on non-Element object")?;
    Ok(JsString::from(value).into())
}

/// `Element.prototype.innerHTML` setter
///
/// This now uses context-aware parsing to properly handle iframe elements.
/// When `<iframe>` tags are set via innerHTML, their contentWindow and contentDocument
/// are properly initialized, enabling postMessage and window hierarchy navigation.
pub(super) fn set_inner_html(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.innerHTML setter called on non-object")
    })?;

    let html = args.get_or_undefined(0).to_string(context)?;
    let html_string = html.to_std_string_escaped();

    // IMPORTANT: We can't hold a borrow on the ElementData while calling context methods
    // because creating iframes requires context access which would cause a borrow conflict.
    // Instead, we:
    // 1. Parse HTML with context first (creates iframe elements with proper context)
    // 2. Then update the ElementData's children (quick operation, no context needed)

    // Parse HTML with context-aware handling for iframes
    let parsed_elements = parse_html_elements_with_context(&html_string, context)?;

    // Now update the ElementData with the parsed children
    with_element_data(&this_obj, |el| {
        // Store the raw HTML
        *el.inner_html.lock().unwrap() = html_string;

        // Update children
        let mut children = el.children.lock().unwrap();
        children.clear();
        children.extend(parsed_elements);

        // Mark as modified by JS
        el.mark_modified_by_js();

        // Recompute text content
        drop(children); // Release children lock before calling method
        el.recompute_text_content();

        // Update document HTML
        el.update_document_html_content();
    }, "Element.prototype.innerHTML setter called on non-Element object")?;

    Ok(JsValue::undefined())
}

/// `Element.prototype.textContent` getter
pub(super) fn get_text_content(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.textContent called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_text_content(), "Element.prototype.textContent called on non-Element object")?;
    Ok(JsString::from(value).into())
}

/// `Element.prototype.textContent` setter
pub(super) fn set_text_content(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.textContent setter called on non-object")
    })?;

    let content = args.get_or_undefined(0).to_string(context)?;
    let content_string = content.to_std_string_escaped();

    with_element_data(&this_obj, |el| {
        el.set_text_content(content_string);
        el.mark_modified_by_js();
    }, "Element.prototype.textContent setter called on non-Element object")?;

    Ok(JsValue::undefined())
}

/// `Element.prototype.children` getter
pub(super) fn get_children(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.children called on non-object")
    })?;

    let children = with_element_data(&this_obj, |el| el.get_children(), "Element.prototype.children called on non-Element object")?;

    let children_values: Vec<JsValue> = children.into_iter().map(|child| child.into()).collect();
    let array = Array::create_array_from_list(children_values, context);
    Ok(array.into())
}

/// `Element.prototype.parentNode` getter
pub(super) fn get_parent_node(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.parentNode called on non-object")
    })?;

    let parent_node = with_element_data(&this_obj, |el| el.get_parent_node(), "Element.prototype.parentNode called on non-Element object")?;

    Ok(parent_node.map(|parent| parent.into()).unwrap_or(JsValue::null()))
}

/// `Element.prototype.style` getter
pub(super) fn get_style(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.style called on non-object")
    })?;

    // Verify it's an element
    if !has_element_data(&this_obj) {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.style called on non-Element object")
            .into());
    }

    // Create a proper CSSStyleDeclaration object with the correct prototype
    use boa_engine::builtins::BuiltInConstructor;
    let css_style_constructor = context.intrinsics().constructors().css_style_declaration().constructor();
    crate::browser::cssom::CSSStyleDeclaration::constructor(
        &css_style_constructor.into(),
        &[],
        context,
    )
}

/// `Element.prototype.dataset` getter
pub(super) fn get_dataset(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.dataset called on non-object")
    })?;

    // Verify it's an element
    if !has_element_data(&this_obj) {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.dataset called on non-Element object")
            .into());
    }

    // Create a DOMStringMap populated with current data-* attributes
    let map = crate::dom::domstringmap::DOMStringMap::create_for_element(this_obj.clone(), context)?;
    Ok(map.into())
}

/// `Element.prototype.classList` getter
pub(super) fn get_class_list(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.classList called on non-object")
    })?;

    // Verify it's an element
    if !has_element_data(&this_obj) {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.classList called on non-Element object")
            .into());
    }

    // Create or return a DOMTokenList bound to this element
    let list = crate::dom::domtokenlist::DOMTokenList::create_for_element(this_obj.clone(), context)?;
    Ok(list.into())
}

/// `Element.prototype.setAttribute(name, value)`
pub(super) fn set_attribute_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.setAttribute called on non-object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    let value = args.get_or_undefined(1).to_string(context)?;
    let name_str = name.to_std_string_escaped();
    let value_str = value.to_std_string_escaped();

    // Try dispatch through all element types
    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        element.set_attribute(name_str, value_str);
    } else if let Some(iframe) = this_obj.downcast_ref::<crate::dom::html_iframe_element::HTMLIFrameElementData>() {
        iframe.set_attribute(&name_str, value_str);
    } else if let Some(script) = this_obj.downcast_ref::<crate::dom::html_script_element::HTMLScriptElementData>() {
        script.set_attribute(&name_str, value_str);
    } else if let Some(_html_data) = this_obj.downcast_ref::<crate::dom::html_element::HTMLElementData>() {
        // HTMLElement objects - no-op for now (no attribute storage)
        return Ok(JsValue::undefined());
    } else {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.setAttribute called on non-Element object")
            .into());
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.getAttribute(name)`
pub(super) fn get_attribute_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.getAttribute called on non-object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    let name_str = name.to_std_string_escaped();

    // Dispatch: try ElementData, then iframe, then script
    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        if let Some(value) = element.get_attribute(&name_str) {
            Ok(JsString::from(value).into())
        } else {
            Ok(JsValue::null())
        }
    } else if let Some(iframe) = this_obj.downcast_ref::<crate::dom::html_iframe_element::HTMLIFrameElementData>() {
        if let Some(value) = iframe.get_attribute(&name_str) {
            Ok(JsString::from(value).into())
        } else {
            Ok(JsValue::null())
        }
    } else if let Some(script) = this_obj.downcast_ref::<crate::dom::html_script_element::HTMLScriptElementData>() {
        if let Some(value) = script.get_attribute(&name_str) {
            Ok(JsString::from(value).into())
        } else {
            Ok(JsValue::null())
        }
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.getAttribute called on non-Element object")
            .into())
    }
}

/// `Element.prototype.hasAttribute(name)`
pub(super) fn has_attribute_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.hasAttribute called on non-object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    let name_str = name.to_std_string_escaped();

    let result = with_element_data(&this_obj, |el| el.has_attribute(&name_str), "Element.prototype.hasAttribute called on non-Element object")?;
    Ok(result.into())
}

/// `Element.prototype.removeAttribute(name)`
pub(super) fn remove_attribute_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.removeAttribute called on non-object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    let name_str = name.to_std_string_escaped();

    with_element_data(&this_obj, |el| el.remove_attribute(&name_str), "Element.prototype.removeAttribute called on non-Element object")?;
    Ok(JsValue::undefined())
}

/// `Element.prototype.firstChild` getter
pub(super) fn get_first_child(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.firstChild called on non-object")
    })?;

    let child = with_element_data(&this_obj, |el| el.get_first_child(), "Element.prototype.firstChild called on non-Element object")?;
    Ok(child.map(|c| c.into()).unwrap_or(JsValue::null()))
}

/// `Element.prototype.lastChild` getter
pub(super) fn get_last_child(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.lastChild called on non-object")
    })?;

    let child = with_element_data(&this_obj, |el| el.get_last_child(), "Element.prototype.lastChild called on non-Element object")?;
    Ok(child.map(|c| c.into()).unwrap_or(JsValue::null()))
}

/// `Element.prototype.nextSibling` getter
pub(super) fn get_next_sibling(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.nextSibling called on non-object")
    })?;

    let sibling = with_element_data(&this_obj, |el| el.get_next_sibling(), "Element.prototype.nextSibling called on non-Element object")?;
    Ok(sibling.map(|c| c.into()).unwrap_or(JsValue::null()))
}

/// `Element.prototype.previousSibling` getter
pub(super) fn get_previous_sibling(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.previousSibling called on non-object")
    })?;

    let sibling = with_element_data(&this_obj, |el| el.get_previous_sibling(), "Element.prototype.previousSibling called on non-Element object")?;
    Ok(sibling.map(|c| c.into()).unwrap_or(JsValue::null()))
}

/// `Element.prototype.nodeType` getter (returns ELEMENT_NODE = 1)
pub(super) fn get_node_type(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.nodeType called on non-object")
    })?;

    // Verify it's an element
    if !has_element_data(&this_obj) {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.nodeType called on non-Element object")
            .into());
    }

    // ELEMENT_NODE = 1
    Ok(1.into())
}

/// `Element.prototype.nodeName` getter (returns tagName)
pub(super) fn get_node_name(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.nodeName called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_tag_name(), "Element.prototype.nodeName called on non-Element object")?;
    Ok(JsString::from(value).into())
}

/// `Element.prototype.outerHTML` getter
pub(super) fn get_outer_html(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.outerHTML called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.serialize_to_html(), "Element.prototype.outerHTML called on non-Element object")?;
    Ok(JsString::from(value).into())
}

/// `Element.prototype.outerHTML` setter
/// Uses context-aware parsing to properly handle iframe elements
pub(super) fn set_outer_html(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.outerHTML setter called on non-object")
    })?;

    let html = args.get_or_undefined(0).to_string(context)?;
    let html_string = html.to_std_string_escaped();

    // Parse HTML with context-aware handling for iframes
    let parsed_elements = parse_html_elements_with_context(&html_string, context)?;

    // Update the ElementData
    with_element_data(&this_obj, |el| {
        *el.inner_html.lock().unwrap() = html_string;
        let mut children = el.children.lock().unwrap();
        children.clear();
        children.extend(parsed_elements);
        drop(children);
        el.recompute_text_content();
        el.update_document_html_content();
    }, "Element.prototype.outerHTML setter called on non-Element object")?;

    Ok(JsValue::undefined())
}

/// `Element.prototype.childNodes` getter
pub(super) fn get_child_nodes(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.childNodes called on non-object")
    })?;

    let children = with_element_data(&this_obj, |el| el.get_children(), "Element.prototype.childNodes called on non-Element object")?;

    // Create a NodeList-like array
    let children_values: Vec<JsValue> = children.into_iter().map(|child| child.into()).collect();
    let array = Array::create_array_from_list(children_values, context);

    // Add item() method for NodeList compatibility
    let item_fn = BuiltInBuilder::callable(context.realm(), |this, args, ctx| {
        let index = args.get_or_undefined(0).to_u32(ctx)?;
        if let Some(arr) = this.as_object() {
            if let Ok(val) = arr.get(index, ctx) {
                if !val.is_undefined() {
                    return Ok(val);
                }
            }
        }
        Ok(JsValue::null())
    })
    .name(js_string!("item"))
    .build();
    array.set(js_string!("item"), item_fn, false, context)?;

    Ok(array.into())
}
