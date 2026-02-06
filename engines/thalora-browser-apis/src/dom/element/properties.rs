//! Element property getters and setters (tagName, id, className, innerHTML, textContent, etc.)

use boa_engine::{
    builtins::{BuiltInBuilder, array::Array},
    object::JsObject,
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
    JsString,
};

use super::types::ElementData;
use super::dom_manipulation::parse_html_elements_with_context;

/// `Element.prototype.tagName` getter
pub(super) fn get_tag_name(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.tagName called on non-object")
    })?;

    let value = {


        let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {


            JsNativeError::typ()


                .with_message("Element.prototype.tagName called on non-Element object")


        })?;


        element.get_tag_name()


    };
    Ok(JsString::from(value).into())
}

/// `Element.prototype.id` getter
pub(super) fn get_id(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.id called on non-object")
    })?;

    // Try ElementData first, then HTMLIFrameElementData
    let value = if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        element.get_id()
    } else if let Some(iframe) = this_obj.downcast_ref::<crate::dom::html_iframe_element::HTMLIFrameElementData>() {
        iframe.get_id()
    } else {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.id called on non-Element object")
            .into());
    };
    Ok(JsString::from(value).into())
}

/// `Element.prototype.id` setter
pub(super) fn set_id(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.id setter called on non-object")
    })?;

    let id = args.get_or_undefined(0).to_string(context)?;
    let id_string = id.to_std_string_escaped();

    // Try ElementData first, then HTMLIFrameElementData
    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        element.set_id(id_string);
    } else if let Some(iframe) = this_obj.downcast_ref::<crate::dom::html_iframe_element::HTMLIFrameElementData>() {
        iframe.set_id(id_string);
    } else {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.id setter called on non-Element object")
            .into());
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.className` getter
pub(super) fn get_class_name(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.className called on non-object")
    })?;

    // Try ElementData first, then HTMLIFrameElementData
    let value = if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        element.get_class_name()
    } else if let Some(iframe) = this_obj.downcast_ref::<crate::dom::html_iframe_element::HTMLIFrameElementData>() {
        iframe.get_class_name()
    } else {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.className called on non-Element object")
            .into());
    };
    Ok(JsString::from(value).into())
}

/// `Element.prototype.className` setter
pub(super) fn set_class_name(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.className setter called on non-object")
    })?;

    let class_name = args.get_or_undefined(0).to_string(context)?;
    let class_name_string = class_name.to_std_string_escaped();

    // Try ElementData first, then HTMLIFrameElementData
    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        element.set_class_name(class_name_string);
    } else if let Some(iframe) = this_obj.downcast_ref::<crate::dom::html_iframe_element::HTMLIFrameElementData>() {
        iframe.set_class_name(class_name_string);
    } else {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.className setter called on non-Element object")
            .into());
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.innerHTML` getter
pub(super) fn get_inner_html(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.innerHTML called on non-object")
    })?;

    let value = {


        let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {


            JsNativeError::typ()


                .with_message("Element.prototype.innerHTML called on non-Element object")


        })?;


        element.get_inner_html()


    };
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
    // This must be done in a separate scope to avoid holding the borrow during context calls
    {
        let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Element.prototype.innerHTML setter called on non-Element object")
        })?;

        // Store the raw HTML
        *element.inner_html.lock().unwrap() = html_string;

        // Update children
        let mut children = element.children.lock().unwrap();
        children.clear();
        children.extend(parsed_elements);

        // Recompute text content
        drop(children); // Release children lock before calling method
        element.recompute_text_content();

        // Update document HTML
        element.update_document_html_content();
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.textContent` getter
pub(super) fn get_text_content(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.textContent called on non-object")
    })?;

    let value = {


        let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {


            JsNativeError::typ()


                .with_message("Element.prototype.textContent called on non-Element object")


        })?;


        element.get_text_content()


    };
    Ok(JsString::from(value).into())
}

/// `Element.prototype.textContent` setter
pub(super) fn set_text_content(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.textContent setter called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.textContent setter called on non-Element object")
    })?;

    let content = args.get_or_undefined(0).to_string(context)?;
    let content_string = content.to_std_string_escaped();

    element.set_text_content(content_string);

    Ok(JsValue::undefined())
}

/// `Element.prototype.children` getter
pub(super) fn get_children(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.children called on non-object")
    })?;

    let children = {


        let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {


            JsNativeError::typ()


                .with_message("Element.prototype.children called on non-Element object")


        })?;


        element.get_children()


    };

    let children_values: Vec<JsValue> = children.into_iter().map(|child| child.into()).collect();
    let array = Array::create_array_from_list(children_values, context);
    Ok(array.into())
}

/// `Element.prototype.parentNode` getter
pub(super) fn get_parent_node(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.parentNode called on non-object")
    })?;

    let parent_node = {


        let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {


            JsNativeError::typ()


                .with_message("Element.prototype.parentNode called on non-Element object")


        })?;


        element.get_parent_node()


    };

    Ok(parent_node.map(|parent| parent.into()).unwrap_or(JsValue::null()))
}

/// `Element.prototype.style` getter
pub(super) fn get_style(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.style called on non-object")
    })?;

    // Verify it's an element
    if this_obj.downcast_ref::<ElementData>().is_none() {
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

/// `Element.prototype.classList` getter
pub(super) fn get_class_list(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.classList called on non-object")
    })?;

    // Verify it's an element (using scope to drop the borrow immediately)
    {
        if this_obj.downcast_ref::<ElementData>().is_none() {
            return Err(JsNativeError::typ()
                .with_message("Element.prototype.classList called on non-Element object")
                .into());
        }
    }

    // Create or return a DOMTokenList bound to this element
    let list = crate::dom::domtokenlist::DOMTokenList::create_for_element(this_obj.clone(), context)?;
    Ok(list.into())
}

/// `Element.prototype.setAttribute(name, value)`
pub(super) fn set_attribute_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use crate::debug_utils::log_to_file;

    log_to_file("/tmp/setattr_debug.log", || format!("setAttribute ENTRY: type={}", this.type_of()));

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.setAttribute called on non-object")
    })?;

    // Try to downcast to ElementData first, then try HTMLElementData as fallback
    let element = if let Some(data) = this_obj.downcast_ref::<ElementData>() {
        log_to_file("/tmp/setattr_debug.log", || "setAttribute SUCCESS: ElementData");
        data
    } else if let Some(_html_data) = this_obj.downcast_ref::<crate::dom::html_element::HTMLElementData>() {
        // HTMLElement objects also need setAttribute support
        let name = args.get_or_undefined(0).to_string(context)?;
        let _value = args.get_or_undefined(1).to_string(context)?;
        log_to_file("/tmp/setattr_debug.log", || format!("setAttribute: HTMLElementData name='{}'", name.to_std_string_escaped()));
        return Ok(JsValue::undefined());
    } else if let Some(iframe_data) = this_obj.downcast_ref::<crate::dom::html_iframe_element::HTMLIFrameElementData>() {
        // HTMLIFrameElement objects
        let name = args.get_or_undefined(0).to_string(context)?;
        let value = args.get_or_undefined(1).to_string(context)?;
        iframe_data.set_attribute(&name.to_std_string_escaped(), value.to_std_string_escaped());
        log_to_file("/tmp/setattr_debug.log", || format!("setAttribute SUCCESS: HTMLIFrameElementData name='{}'", name.to_std_string_escaped()));
        return Ok(JsValue::undefined());
    } else {
        // Debug output to understand what type of object this is
        let constructor_name = if let Ok(constructor) = this_obj.get(js_string!("constructor"), context) {
            if let Some(ctor_obj) = constructor.as_object() {
                ctor_obj.get(js_string!("name"), context)
                    .map(|n| n.to_string(context).map(|s| s.to_std_string_escaped()).unwrap_or_default())
                    .unwrap_or_else(|_| "?".to_string())
            } else { "not an object".to_string() }
        } else { "no constructor".to_string() };

        let has_document_data = this_obj.downcast_ref::<crate::dom::document::DocumentData>().is_some();
        let has_nodelist_data = this_obj.downcast_ref::<crate::dom::nodelist::NodeListData>().is_some();
        let has_style_data = this_obj.downcast_ref::<crate::browser::cssom::CSSStyleDeclarationData>().is_some();

        log_to_file("/tmp/setattr_debug.log", || {
            format!("setAttribute FAIL: unknown type, constructor='{}', isDoc={}, isNodeList={}, isStyle={}",
                constructor_name, has_document_data, has_nodelist_data, has_style_data)
        });

        return Err(JsNativeError::typ()
            .with_message("Element.prototype.setAttribute called on non-Element object")
            .into());
    };

    let name = args.get_or_undefined(0).to_string(context)?;
    let value = args.get_or_undefined(1).to_string(context)?;
    element.set_attribute(name.to_std_string_escaped(), value.to_std_string_escaped());
    Ok(JsValue::undefined())
}

/// `Element.prototype.getAttribute(name)`
pub(super) fn get_attribute_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.getAttribute called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.getAttribute called on non-Element object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    if let Some(value) = element.get_attribute(&name.to_std_string_escaped()) {
        Ok(JsString::from(value).into())
    } else {
        Ok(JsValue::null())
    }
}

/// `Element.prototype.hasAttribute(name)`
pub(super) fn has_attribute_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.hasAttribute called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.hasAttribute called on non-Element object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    Ok(element.has_attribute(&name.to_std_string_escaped()).into())
}

/// `Element.prototype.removeAttribute(name)`
pub(super) fn remove_attribute_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.removeAttribute called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.removeAttribute called on non-Element object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    element.remove_attribute(&name.to_std_string_escaped());
    Ok(JsValue::undefined())
}

/// `Element.prototype.firstChild` getter
pub(super) fn get_first_child(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.firstChild called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.firstChild called on non-Element object")
    })?;

    Ok(element.get_first_child().map(|c| c.into()).unwrap_or(JsValue::null()))
}

/// `Element.prototype.lastChild` getter
pub(super) fn get_last_child(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.lastChild called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.lastChild called on non-Element object")
    })?;

    Ok(element.get_last_child().map(|c| c.into()).unwrap_or(JsValue::null()))
}

/// `Element.prototype.nextSibling` getter
pub(super) fn get_next_sibling(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.nextSibling called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.nextSibling called on non-Element object")
    })?;

    Ok(element.get_next_sibling().map(|c| c.into()).unwrap_or(JsValue::null()))
}

/// `Element.prototype.previousSibling` getter
pub(super) fn get_previous_sibling(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.previousSibling called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.previousSibling called on non-Element object")
    })?;

    Ok(element.get_previous_sibling().map(|c| c.into()).unwrap_or(JsValue::null()))
}

/// `Element.prototype.nodeType` getter (returns ELEMENT_NODE = 1)
pub(super) fn get_node_type(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.nodeType called on non-object")
    })?;

    // Verify it's an element
    this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.nodeType called on non-Element object")
    })?;

    // ELEMENT_NODE = 1
    Ok(1.into())
}

/// `Element.prototype.nodeName` getter (returns tagName)
pub(super) fn get_node_name(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.nodeName called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.nodeName called on non-Element object")
    })?;

    Ok(JsString::from(element.get_tag_name()).into())
}

/// `Element.prototype.outerHTML` getter
pub(super) fn get_outer_html(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.outerHTML called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.outerHTML called on non-Element object")
    })?;

    Ok(JsString::from(element.serialize_to_html()).into())
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
    // Setting outerHTML replaces this element in its parent
    // For now, we'll just update the innerHTML and attributes
    // Full implementation would require parent manipulation
    {
        let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Element.prototype.outerHTML setter called on non-Element object")
        })?;

        *element.inner_html.lock().unwrap() = html_string;
        let mut children = element.children.lock().unwrap();
        children.clear();
        children.extend(parsed_elements);
        drop(children);
        element.recompute_text_content();
        element.update_document_html_content();
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.childNodes` getter
pub(super) fn get_child_nodes(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.childNodes called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.childNodes called on non-Element object")
    })?;

    let children = element.get_children();

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
