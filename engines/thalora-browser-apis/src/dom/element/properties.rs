//! Element property getters and setters (tagName, id, className, innerHTML, textContent, etc.)

use boa_engine::{
    builtins::array::Array,
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

    let value = with_element_data(&this_obj, |el| el.serialize_inner_html(), "Element.prototype.innerHTML called on non-Element object")?;
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

/// `Element.prototype.children` getter — returns HTMLCollection (element children only)
pub(super) fn get_children(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.children called on non-object")
    })?;

    let children = with_element_data(&this_obj, |el| el.get_children(), "Element.prototype.children called on non-Element object")?;

    // Per spec, children returns only Element nodes (not text/comment nodes)
    let element_children: Vec<JsObject> = children.into_iter().filter(|c| has_element_data(c)).collect();
    let collection = crate::dom::htmlcollection::HTMLCollection::create_from_elements(element_children, context)?;
    Ok(collection.into())
}

/// `Element.prototype.parentNode` getter
pub(super) fn get_parent_node(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.parentNode called on non-object")
    })?;

    let parent_node = with_element_data(&this_obj, |el| el.get_parent_node(), "Element.prototype.parentNode called on non-Element object")?;

    Ok(parent_node.map(|parent| parent.into()).unwrap_or(JsValue::null()))
}

/// `Element.prototype.style` getter — returns cached CSSStyleDeclaration (same object identity)
pub(super) fn get_style(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.style called on non-object")
    })?;

    if !has_element_data(&this_obj) {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.style called on non-Element object")
            .into());
    }

    // Return cached style object if it exists
    let cached = this_obj.get(js_string!("__style__"), context)?;
    if cached.is_object() {
        return Ok(cached);
    }

    // Create and cache a new CSSStyleDeclaration
    use boa_engine::builtins::BuiltInConstructor;
    let css_style_constructor = context.intrinsics().constructors().css_style_declaration().constructor();
    let style = crate::browser::cssom::CSSStyleDeclaration::constructor(
        &css_style_constructor.into(),
        &[],
        context,
    )?;

    // Cache as non-enumerable internal property
    use boa_engine::property::PropertyDescriptor;
    this_obj.define_property_or_throw(
        js_string!("__style__"),
        PropertyDescriptor::builder()
            .value(style.clone())
            .writable(false)
            .enumerable(false)
            .configurable(false),
        context,
    )?;

    Ok(style)
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

/// `Element.prototype.classList` getter — returns cached DOMTokenList (same object identity)
pub(super) fn get_class_list(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.classList called on non-object")
    })?;

    if !has_element_data(&this_obj) {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.classList called on non-Element object")
            .into());
    }

    // Return cached classList object if it exists
    let cached = this_obj.get(js_string!("__classList__"), context)?;
    if cached.is_object() {
        return Ok(cached);
    }

    // Create and cache a new DOMTokenList
    let list = crate::dom::domtokenlist::DOMTokenList::create_for_element(this_obj.clone(), context)?;
    let list_value: JsValue = list.into();

    // Cache as non-enumerable internal property
    use boa_engine::property::PropertyDescriptor;
    this_obj.define_property_or_throw(
        js_string!("__classList__"),
        PropertyDescriptor::builder()
            .value(list_value.clone())
            .writable(false)
            .enumerable(false)
            .configurable(false),
        context,
    )?;

    Ok(list_value)
}

/// `Element.prototype.nextElementSibling` getter
pub(super) fn get_next_element_sibling(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.nextElementSibling called on non-object")
    })?;

    let sibling = with_element_data(&this_obj, |el| el.get_next_sibling(), "Element.prototype.nextElementSibling called on non-Element object")?;

    // Filter to only element nodes (nodeType == 1) by checking if it has ElementData
    if let Some(sib) = sibling {
        if has_element_data(&sib) {
            return Ok(sib.into());
        }
        // Walk forward to find the next element sibling
        let mut current = sib;
        loop {
            let next = with_element_data(&current, |el| el.get_next_sibling(), "");
            match next {
                Ok(Some(next_sib)) => {
                    if has_element_data(&next_sib) {
                        return Ok(next_sib.into());
                    }
                    current = next_sib;
                }
                _ => break,
            }
        }
    }
    Ok(JsValue::null())
}

/// `Element.prototype.previousElementSibling` getter
pub(super) fn get_previous_element_sibling(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.previousElementSibling called on non-object")
    })?;

    let sibling = with_element_data(&this_obj, |el| el.get_previous_sibling(), "Element.prototype.previousElementSibling called on non-Element object")?;

    if let Some(sib) = sibling {
        if has_element_data(&sib) {
            return Ok(sib.into());
        }
        let mut current = sib;
        loop {
            let prev = with_element_data(&current, |el| el.get_previous_sibling(), "");
            match prev {
                Ok(Some(prev_sib)) => {
                    if has_element_data(&prev_sib) {
                        return Ok(prev_sib.into());
                    }
                    current = prev_sib;
                }
                _ => break,
            }
        }
    }
    Ok(JsValue::null())
}

/// `Element.prototype.firstElementChild` getter
pub(super) fn get_first_element_child(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.firstElementChild called on non-object")
    })?;

    let children = with_element_data(&this_obj, |el| el.get_children(), "Element.prototype.firstElementChild called on non-Element object")?;

    for child in children {
        if has_element_data(&child) {
            return Ok(child.into());
        }
    }
    Ok(JsValue::null())
}

/// `Element.prototype.lastElementChild` getter
pub(super) fn get_last_element_child(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.lastElementChild called on non-object")
    })?;

    let children = with_element_data(&this_obj, |el| el.get_children(), "Element.prototype.lastElementChild called on non-Element object")?;

    for child in children.into_iter().rev() {
        if has_element_data(&child) {
            return Ok(child.into());
        }
    }
    Ok(JsValue::null())
}

/// `Element.prototype.childElementCount` getter
pub(super) fn get_child_element_count(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.childElementCount called on non-object")
    })?;

    let children = with_element_data(&this_obj, |el| el.get_children(), "Element.prototype.childElementCount called on non-Element object")?;

    let count = children.iter().filter(|c| has_element_data(c)).count();
    Ok((count as u32).into())
}

/// `Element.prototype.parentElement` getter
pub(super) fn get_parent_element(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.parentElement called on non-object")
    })?;

    let parent = with_element_data(&this_obj, |el| el.get_parent_node(), "Element.prototype.parentElement called on non-Element object")?;

    // parentElement returns null if the parent is not an Element (e.g., Document)
    if let Some(p) = parent {
        if has_element_data(&p) {
            return Ok(p.into());
        }
    }
    Ok(JsValue::null())
}

/// `Element.prototype.getAttributeNames()`
pub(super) fn get_attribute_names(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.getAttributeNames called on non-object")
    })?;

    let names = with_element_data(&this_obj, |el| {
        el.attributes.lock().unwrap().keys().cloned().collect::<Vec<_>>()
    }, "Element.prototype.getAttributeNames called on non-Element object")?;

    let values: Vec<JsValue> = names.into_iter().map(|n| JsString::from(n).into()).collect();
    Ok(Array::create_array_from_list(values, context).into())
}

/// `Element.prototype.attributes` getter — returns NamedNodeMap of attributes
pub(super) fn get_attributes(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.attributes called on non-object")
    })?;

    // Get the attribute HashMap from ElementData
    let attrs = with_element_data(&this_obj, |el| {
        el.attributes.lock().unwrap().clone()
    }, "Element.prototype.attributes called on non-Element object")?;

    // Build a NamedNodeMap from the attributes
    let named_node_map_data = crate::dom::namednodemap::NamedNodeMapData::from_attributes(attrs, context)?;

    let len = named_node_map_data.length();

    let named_node_map_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        context.intrinsics().constructors().namednodemap().prototype(),
        named_node_map_data,
    );

    let result = named_node_map_obj.upcast();

    // Set indexed properties for el.attributes[0], el.attributes[1] etc.
    // Also needed for Array.from(el.attributes) and [].slice.call(el.attributes)
    for i in 0..len {
        if let Some(attr) = result.downcast_ref::<crate::dom::namednodemap::NamedNodeMapData>()
            .and_then(|d| d.get_item(i))
        {
            result.set(i as u64, attr.clone(), false, context)?;
        }
    }

    Ok(result.into())
}

/// `Element.prototype.namespaceURI` getter
pub(super) fn get_namespace_uri_js(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.namespaceURI called on non-object")
    })?;

    let value = with_element_data(&this_obj, |el| el.get_namespace_uri(), "Element.prototype.namespaceURI called on non-Element object")?;
    // Per spec, HTML elements default to "http://www.w3.org/1999/xhtml"
    let uri = value.unwrap_or_else(|| "http://www.w3.org/1999/xhtml".to_string());
    Ok(JsString::from(uri).into())
}

/// `Element.prototype.isConnected` getter (element-level override)
pub(super) fn get_is_connected_element(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.isConnected called on non-object")
    })?;

    // Elements in the live DOM tree are always connected
    let _connected = with_element_data(&this_obj, |_el| true, "Element.prototype.isConnected called on non-Element object")?;
    Ok(JsValue::from(true))
}

/// `Element.prototype.baseURI` getter (element-level override)
pub(super) fn get_base_uri_element(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.baseURI called on non-object")
    })?;

    if !has_element_data(&this_obj) {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.baseURI called on non-Element object")
            .into());
    }

    // Return empty string — prevents crash without incorrect URL resolution
    Ok(JsString::from("").into())
}

/// `Element.prototype.hasAttributes()`
pub(super) fn has_attributes_js(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.hasAttributes called on non-object")
    })?;

    let result = with_element_data(&this_obj, |el| {
        !el.attributes.lock().unwrap().is_empty()
    }, "Element.prototype.hasAttributes called on non-Element object")?;
    Ok(result.into())
}

/// `Element.prototype.getAttributeNS(namespace, localName)`
pub(super) fn get_attribute_ns(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.getAttributeNS called on non-object")
    })?;

    // args[0] = namespace (ignored — no namespace support yet)
    let local_name = args.get_or_undefined(1).to_string(context)?;
    let local_name_str = local_name.to_std_string_escaped();

    let value = with_element_data(&this_obj, |el| {
        el.get_attribute(&local_name_str)
    }, "Element.prototype.getAttributeNS called on non-Element object")?;

    match value {
        Some(v) => Ok(JsString::from(v).into()),
        None => Ok(JsValue::null()),
    }
}

/// `Element.prototype.setAttributeNS(namespace, qualifiedName, value)`
pub(super) fn set_attribute_ns(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.setAttributeNS called on non-object")
    })?;

    // args[0] = namespace (ignored — no namespace support yet)
    let qualified_name = args.get_or_undefined(1).to_string(context)?;
    let value = args.get_or_undefined(2).to_string(context)?;
    let name_str = qualified_name.to_std_string_escaped();
    let value_str = value.to_std_string_escaped();

    with_element_data(&this_obj, |el| {
        el.set_attribute(name_str, value_str);
    }, "Element.prototype.setAttributeNS called on non-Element object")?;

    Ok(JsValue::undefined())
}

/// `Element.prototype.removeAttributeNS(namespace, localName)`
pub(super) fn remove_attribute_ns(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.removeAttributeNS called on non-object")
    })?;

    // args[0] = namespace (ignored — no namespace support yet)
    let local_name = args.get_or_undefined(1).to_string(context)?;
    let local_name_str = local_name.to_std_string_escaped();

    with_element_data(&this_obj, |el| {
        el.remove_attribute(&local_name_str);
    }, "Element.prototype.removeAttributeNS called on non-Element object")?;

    Ok(JsValue::undefined())
}

/// `Element.prototype.toggleAttribute(name, force?)`
pub(super) fn toggle_attribute(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.toggleAttribute called on non-object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    let name_str = name.to_std_string_escaped();
    let force = if args.len() > 1 && !args[1].is_undefined() {
        Some(args[1].to_boolean())
    } else {
        None
    };

    let result = with_element_data(&this_obj, |el| {
        let has = el.has_attribute(&name_str);
        match force {
            Some(true) => {
                if !has {
                    el.set_attribute(name_str.clone(), String::new());
                }
                true
            }
            Some(false) => {
                if has {
                    el.remove_attribute(&name_str);
                }
                false
            }
            None => {
                if has {
                    el.remove_attribute(&name_str);
                    false
                } else {
                    el.set_attribute(name_str.clone(), String::new());
                    true
                }
            }
        }
    }, "Element.prototype.toggleAttribute called on non-Element object")?;

    Ok(result.into())
}

/// `Element.prototype.insertAdjacentHTML(position, html)`
pub(super) fn insert_adjacent_html(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("insertAdjacentHTML called on non-object")
    })?;

    let position = args.get_or_undefined(0).to_string(context)?;
    let html = args.get_or_undefined(1).to_string(context)?;
    let position_str = position.to_std_string_escaped().to_lowercase();
    let html_str = html.to_std_string_escaped();

    let parsed = parse_html_elements_with_context(&html_str, context)?;

    match position_str.as_str() {
        "beforebegin" => {
            // Insert before this element in its parent
            let parent = with_element_data(&this_obj, |el| el.get_parent_node(), "insertAdjacentHTML")?;
            if let Some(parent_obj) = parent {
                with_element_data(&parent_obj, |parent_el| {
                    let mut children = parent_el.children.lock().unwrap();
                    if let Some(idx) = children.iter().position(|c| JsObject::equals(c, &this_obj)) {
                        for (i, el) in parsed.into_iter().enumerate() {
                            children.insert(idx + i, el);
                        }
                    }
                    parent_el.mark_modified_by_js();
                }, "insertAdjacentHTML parent")?;
            }
        }
        "afterbegin" => {
            with_element_data(&this_obj, |el| {
                let mut children = el.children.lock().unwrap();
                for (i, child) in parsed.into_iter().enumerate() {
                    children.insert(i, child);
                }
                el.mark_modified_by_js();
            }, "insertAdjacentHTML")?;
        }
        "beforeend" => {
            with_element_data(&this_obj, |el| {
                let mut children = el.children.lock().unwrap();
                children.extend(parsed);
                el.mark_modified_by_js();
            }, "insertAdjacentHTML")?;
        }
        "afterend" => {
            let parent = with_element_data(&this_obj, |el| el.get_parent_node(), "insertAdjacentHTML")?;
            if let Some(parent_obj) = parent {
                with_element_data(&parent_obj, |parent_el| {
                    let mut children = parent_el.children.lock().unwrap();
                    if let Some(idx) = children.iter().position(|c| JsObject::equals(c, &this_obj)) {
                        for (i, el) in parsed.into_iter().enumerate() {
                            children.insert(idx + 1 + i, el);
                        }
                    }
                    parent_el.mark_modified_by_js();
                }, "insertAdjacentHTML parent")?;
            }
        }
        _ => {
            return Err(JsNativeError::syntax()
                .with_message(format!("Invalid position: {}", position_str))
                .into());
        }
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.insertAdjacentElement(position, element)`
pub(super) fn insert_adjacent_element(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("insertAdjacentElement called on non-object")
    })?;

    let position = args.get_or_undefined(0).to_string(context)?;
    let position_str = position.to_std_string_escaped().to_lowercase();
    let element = args.get_or_undefined(1);
    let element_obj = element.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("insertAdjacentElement: second argument must be an element")
    })?;

    match position_str.as_str() {
        "beforebegin" => {
            let parent = with_element_data(&this_obj, |el| el.get_parent_node(), "insertAdjacentElement")?;
            if let Some(parent_obj) = parent {
                with_element_data(&parent_obj, |parent_el| {
                    let mut children = parent_el.children.lock().unwrap();
                    if let Some(idx) = children.iter().position(|c| JsObject::equals(c, &this_obj)) {
                        children.insert(idx, element_obj.clone());
                    }
                    parent_el.mark_modified_by_js();
                }, "insertAdjacentElement parent")?;
                Ok(element.clone())
            } else {
                Ok(JsValue::null())
            }
        }
        "afterbegin" => {
            with_element_data(&this_obj, |el| {
                let mut children = el.children.lock().unwrap();
                children.insert(0, element_obj.clone());
                el.mark_modified_by_js();
            }, "insertAdjacentElement")?;
            Ok(element.clone())
        }
        "beforeend" => {
            with_element_data(&this_obj, |el| {
                let mut children = el.children.lock().unwrap();
                children.push(element_obj.clone());
                el.mark_modified_by_js();
            }, "insertAdjacentElement")?;
            Ok(element.clone())
        }
        "afterend" => {
            let parent = with_element_data(&this_obj, |el| el.get_parent_node(), "insertAdjacentElement")?;
            if let Some(parent_obj) = parent {
                with_element_data(&parent_obj, |parent_el| {
                    let mut children = parent_el.children.lock().unwrap();
                    if let Some(idx) = children.iter().position(|c| JsObject::equals(c, &this_obj)) {
                        children.insert(idx + 1, element_obj.clone());
                    }
                    parent_el.mark_modified_by_js();
                }, "insertAdjacentElement parent")?;
                Ok(element.clone())
            } else {
                Ok(JsValue::null())
            }
        }
        _ => {
            Err(JsNativeError::syntax()
                .with_message(format!("Invalid position: {}", position_str))
                .into())
        }
    }
}

/// `Element.prototype.getElementsByClassName(classNames)`
pub(super) fn get_elements_by_class_name(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("getElementsByClassName called on non-object")
    })?;

    let class_names = args.get_or_undefined(0).to_string(context)?;
    let class_names_str = class_names.to_std_string_escaped();
    let target_classes: Vec<&str> = class_names_str.split_whitespace().collect();

    if target_classes.is_empty() {
        return Ok(Array::create_array_from_list(vec![], context).into());
    }

    let mut results = Vec::new();
    fn collect_by_class(obj: &JsObject, target_classes: &[&str], results: &mut Vec<JsValue>) {
        if let Ok(children) = with_element_data(obj, |el| el.get_children(), "") {
            for child in &children {
                if has_element_data(child) {
                    let matches = with_element_data(child, |el| {
                        let class_name = el.get_class_name();
                        let el_classes: Vec<&str> = class_name.split_whitespace().collect();
                        target_classes.iter().all(|tc| el_classes.contains(tc))
                    }, "");
                    if matches.unwrap_or(false) {
                        results.push(child.clone().into());
                    }
                    collect_by_class(child, target_classes, results);
                }
            }
        }
    }
    collect_by_class(&this_obj, &target_classes, &mut results);
    Ok(Array::create_array_from_list(results, context).into())
}

/// `Element.prototype.getElementsByTagName(tagName)`
pub(super) fn get_elements_by_tag_name(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("getElementsByTagName called on non-object")
    })?;

    let tag_name = args.get_or_undefined(0).to_string(context)?;
    let tag_str = tag_name.to_std_string_escaped().to_uppercase();
    let match_all = tag_str == "*";

    let mut results = Vec::new();
    fn collect_by_tag(obj: &JsObject, tag: &str, match_all: bool, results: &mut Vec<JsValue>) {
        if let Ok(children) = with_element_data(obj, |el| el.get_children(), "") {
            for child in &children {
                if has_element_data(child) {
                    let matches = if match_all {
                        true
                    } else {
                        with_element_data(child, |el| el.get_tag_name().to_uppercase() == tag, "").unwrap_or(false)
                    };
                    if matches {
                        results.push(child.clone().into());
                    }
                    collect_by_tag(child, tag, match_all, results);
                }
            }
        }
    }
    collect_by_tag(&this_obj, &tag_str, match_all, &mut results);
    Ok(Array::create_array_from_list(results, context).into())
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

/// `Element.prototype.childNodes` getter — returns NodeList
pub(super) fn get_child_nodes(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.childNodes called on non-object")
    })?;

    let children = with_element_data(&this_obj, |el| el.get_children(), "Element.prototype.childNodes called on non-Element object")?;

    let nodelist = crate::dom::nodelist::NodeList::create_from_nodes(children, false, context)?;
    Ok(nodelist.into())
}
