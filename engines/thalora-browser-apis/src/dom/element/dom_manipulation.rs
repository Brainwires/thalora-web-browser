//! DOM manipulation methods (appendChild, removeChild, append, prepend, etc.)
//! and context-aware HTML parsing for iframe support

use boa_engine::{
    builtins::BuiltInBuilder,
    object::JsObject,
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
    JsString,
};

use super::types::ElementData;
use super::scripts::{is_script_element, execute_script_element};
use super::helpers::{with_element_data, has_element_data};

/// `Element.prototype.appendChild(child)`
pub(super) fn append_child(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.appendChild called on non-object")
    })?;

    let child_value = args.get_or_undefined(0);
    if let Some(child_obj) = child_value.as_object() {
        // Set parent relationship
        let _ = with_element_data(&child_obj, |el| {
            el.set_parent_node(Some(this_obj.clone()));
        }, "");

        // with_element_data handles ElementData, HTMLIFrameElementData, HTMLScriptElementData
        with_element_data(&this_obj, |el| {
            el.append_child(child_obj.clone());
        }, "Node.appendChild called on non-Node object")?;

        // Check if the appended child is a script element and execute it
        if is_script_element(&child_obj, context)? {
            execute_script_element(&child_obj, context)?;
        }

        Ok(child_value.clone())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.appendChild requires an Element argument")
            .into())
    }
}

/// `Element.prototype.removeChild(child)`
pub(super) fn remove_child(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.removeChild called on non-object")
    })?;

    let child_value = args.get_or_undefined(0);
    if let Some(child_obj) = child_value.as_object() {
        // Remove parent relationship
        let _ = with_element_data(&child_obj, |el| {
            el.set_parent_node(None);
        }, "");

        with_element_data(&this_obj, |el| {
            el.remove_child(&child_obj);
        }, "Element.prototype.removeChild called on non-Element object")?;

        Ok(child_value.clone())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.removeChild requires an Element argument")
            .into())
    }
}

/// `Element.prototype.append(...nodes)` - ParentNode mixin
/// Appends nodes or strings as the last children of the element
/// https://dom.spec.whatwg.org/#dom-parentnode-append
pub(super) fn append_method(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.append called on non-object")
    })?;

    // Process each argument and append
    for arg in args {
        if let Some(child_obj) = arg.as_object() {
            // It's a Node - append it
            let _ = with_element_data(&child_obj, |el| {
                el.set_parent_node(Some(this_obj.clone()));
            }, "");

            with_element_data(&this_obj, |el| {
                el.append_child(child_obj.clone());
            }, "Element.prototype.append called on non-Element object")?;

            // Check if the appended child is a script element and execute it
            if is_script_element(&child_obj, context)? {
                execute_script_element(&child_obj, context)?;
            }
        } else {
            // It's a string - create a Text node and append it
            let text_content = arg.to_string(context)?.to_std_string_escaped();
            // Create a simple text node representation
            let text_obj = JsObject::with_null_proto();
            text_obj.set(js_string!("nodeType"), 3, false, context)?; // TEXT_NODE
            text_obj.set(js_string!("textContent"), js_string!(text_content), false, context)?;

            with_element_data(&this_obj, |el| {
                el.append_child(text_obj);
            }, "Element.prototype.append called on non-Element object")?;
        }
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.prepend(...nodes)` - ParentNode mixin
/// Inserts nodes or strings before the first child of the element
/// https://dom.spec.whatwg.org/#dom-parentnode-prepend
pub(super) fn prepend_method(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.prepend called on non-object")
    })?;

    // Process each argument in reverse order and insert at the beginning
    for arg in args.iter().rev() {
        if let Some(child_obj) = arg.as_object() {
            // It's a Node - prepend it
            let _ = with_element_data(&child_obj, |el| {
                el.set_parent_node(Some(this_obj.clone()));
            }, "");

            with_element_data(&this_obj, |el| {
                el.prepend_child(child_obj.clone());
            }, "Element.prototype.prepend called on non-Element object")?;
        } else {
            // It's a string - create a Text node and prepend it
            let text_content = arg.to_string(context)?.to_std_string_escaped();
            let text_obj = JsObject::with_null_proto();
            text_obj.set(js_string!("nodeType"), 3, false, context)?;
            text_obj.set(js_string!("textContent"), js_string!(text_content), false, context)?;

            with_element_data(&this_obj, |el| {
                el.prepend_child(text_obj);
            }, "Element.prototype.prepend called on non-Element object")?;
        }
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.after(...nodes)` - ChildNode mixin
/// Inserts nodes or strings after this element
/// https://dom.spec.whatwg.org/#dom-childnode-after
pub(super) fn after_method(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.after called on non-object")
    })?;

    // Get parent node via with_element_data
    let parent_obj = with_element_data(&this_obj, |el| {
        el.get_parent_node()
    }, "Element.prototype.after called on non-Element object")?;

    // Process each argument and insert after this element
    if let Some(parent_obj) = parent_obj {
        if has_element_data(&parent_obj) {
            for arg in args.iter() {
                if let Some(child_obj) = arg.as_object() {
                    let _ = with_element_data(&child_obj, |el| {
                        el.set_parent_node(Some(parent_obj.clone()));
                    }, "");

                    with_element_data(&parent_obj, |parent_el| {
                        parent_el.insert_after(child_obj.clone(), &this_obj);
                    }, "Element.prototype.after: parent is not an Element")?;
                } else {
                    let text_content = arg.to_string(context)?.to_std_string_escaped();
                    let text_obj = JsObject::with_null_proto();
                    text_obj.set(js_string!("nodeType"), 3, false, context)?;
                    text_obj.set(js_string!("textContent"), js_string!(text_content), false, context)?;

                    with_element_data(&parent_obj, |parent_el| {
                        parent_el.insert_after(text_obj, &this_obj);
                    }, "Element.prototype.after: parent is not an Element")?;
                }
            }
        }
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.before(...nodes)` - ChildNode mixin
/// Inserts nodes or strings before this element
/// https://dom.spec.whatwg.org/#dom-childnode-before
pub(super) fn before_method(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.before called on non-object")
    })?;

    // Get parent node via with_element_data
    let parent_obj = with_element_data(&this_obj, |el| {
        el.get_parent_node()
    }, "Element.prototype.before called on non-Element object")?;

    // Process each argument in reverse and insert before this element
    if let Some(parent_obj) = parent_obj {
        if has_element_data(&parent_obj) {
            for arg in args.iter().rev() {
                if let Some(child_obj) = arg.as_object() {
                    let _ = with_element_data(&child_obj, |el| {
                        el.set_parent_node(Some(parent_obj.clone()));
                    }, "");

                    with_element_data(&parent_obj, |parent_el| {
                        parent_el.insert_before_elem(child_obj.clone(), &this_obj);
                    }, "Element.prototype.before: parent is not an Element")?;
                } else {
                    let text_content = arg.to_string(context)?.to_std_string_escaped();
                    let text_obj = JsObject::with_null_proto();
                    text_obj.set(js_string!("nodeType"), 3, false, context)?;
                    text_obj.set(js_string!("textContent"), js_string!(text_content), false, context)?;

                    with_element_data(&parent_obj, |parent_el| {
                        parent_el.insert_before_elem(text_obj, &this_obj);
                    }, "Element.prototype.before: parent is not an Element")?;
                }
            }
        }
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.remove()` - ChildNode mixin
/// Removes this element from its parent
/// https://dom.spec.whatwg.org/#dom-childnode-remove
pub(super) fn remove_method(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.remove called on non-object")
    })?;

    // Get parent node via with_element_data
    let parent_obj = with_element_data(&this_obj, |el| {
        el.get_parent_node()
    }, "Element.prototype.remove called on non-Element object")?;

    // Get parent node and remove this element from it
    if let Some(parent_obj) = parent_obj {
        if has_element_data(&parent_obj) {
            with_element_data(&parent_obj, |parent_el| {
                parent_el.remove_child(&this_obj);
            }, "Element.prototype.remove: parent is not an Element")?;

            with_element_data(&this_obj, |el| {
                el.set_parent_node(None);
            }, "Element.prototype.remove called on non-Element object")?;
        }
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.replaceWith(...nodes)` - ChildNode mixin
/// Replaces this element with nodes or strings
/// https://dom.spec.whatwg.org/#dom-childnode-replacewith
pub(super) fn replace_with_method(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.replaceWith called on non-object")
    })?;

    // Get parent node via with_element_data
    let parent_obj = with_element_data(&this_obj, |el| {
        el.get_parent_node()
    }, "Element.prototype.replaceWith called on non-Element object")?;

    // Get parent node
    if let Some(parent_obj) = parent_obj {
        if has_element_data(&parent_obj) {
            // Insert all new nodes before this element
            for arg in args.iter() {
                if let Some(child_obj) = arg.as_object() {
                    let _ = with_element_data(&child_obj, |el| {
                        el.set_parent_node(Some(parent_obj.clone()));
                    }, "");

                    with_element_data(&parent_obj, |parent_el| {
                        parent_el.insert_before_elem(child_obj.clone(), &this_obj);
                    }, "Element.prototype.replaceWith: parent is not an Element")?;
                } else {
                    let text_content = arg.to_string(context)?.to_std_string_escaped();
                    let text_obj = JsObject::with_null_proto();
                    text_obj.set(js_string!("nodeType"), 3, false, context)?;
                    text_obj.set(js_string!("textContent"), js_string!(text_content), false, context)?;

                    with_element_data(&parent_obj, |parent_el| {
                        parent_el.insert_before_elem(text_obj, &this_obj);
                    }, "Element.prototype.replaceWith: parent is not an Element")?;
                }
            }
            // Remove this element
            with_element_data(&parent_obj, |parent_el| {
                parent_el.remove_child(&this_obj);
            }, "Element.prototype.replaceWith: parent is not an Element")?;

            with_element_data(&this_obj, |el| {
                el.set_parent_node(None);
            }, "Element.prototype.replaceWith called on non-Element object")?;
        }
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.replaceChildren(...nodes)` - ParentNode mixin
/// Replaces all children of this element with new nodes or strings
/// https://dom.spec.whatwg.org/#dom-parentnode-replacechildren
pub(super) fn replace_children_method(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.replaceChildren called on non-object")
    })?;

    // Clear all existing children
    with_element_data(&this_obj, |el| {
        el.clear_children();
    }, "Element.prototype.replaceChildren called on non-Element object")?;

    // Add all new nodes
    for arg in args {
        if let Some(child_obj) = arg.as_object() {
            let _ = with_element_data(&child_obj, |el| {
                el.set_parent_node(Some(this_obj.clone()));
            }, "");

            with_element_data(&this_obj, |el| {
                el.append_child(child_obj.clone());
            }, "Element.prototype.replaceChildren called on non-Element object")?;
        } else {
            let text_content = arg.to_string(context)?.to_std_string_escaped();
            let text_obj = JsObject::with_null_proto();
            text_obj.set(js_string!("nodeType"), 3, false, context)?;
            text_obj.set(js_string!("textContent"), js_string!(text_content), false, context)?;

            with_element_data(&this_obj, |el| {
                el.append_child(text_obj);
            }, "Element.prototype.replaceChildren called on non-Element object")?;
        }
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.setHTML(input, options)` - Chrome 124
/// Uses context-aware parsing to properly handle iframe elements
pub(super) fn set_html(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.setHTML called on non-object")
    })?;

    let input = args.get_or_undefined(0).to_string(context)?;
    let html_string = input.to_std_string_escaped();
    let _options = args.get(1).cloned().unwrap_or(JsValue::undefined());

    // Parse HTML with context-aware handling for iframes
    let parsed_elements = parse_html_elements_with_context(&html_string, context)?;

    // Update the ElementData
    with_element_data(&this_obj, |element| {
        *element.inner_html.lock().unwrap() = html_string;
        let mut children = element.children.lock().unwrap();
        children.clear();
        children.extend(parsed_elements);
        drop(children);
        element.recompute_text_content();
        element.update_document_html_content();
    }, "Element.prototype.setHTML called on non-Element object")?;

    Ok(JsValue::undefined())
}

/// `Element.prototype.setHTMLUnsafe(input)` - Chrome 124
/// Uses context-aware parsing to properly handle iframe elements
pub(super) fn set_html_unsafe(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.setHTMLUnsafe called on non-object")
    })?;

    let input = args.get_or_undefined(0).to_string(context)?;
    let html_string = input.to_std_string_escaped();

    // Parse HTML with context-aware handling for iframes
    let parsed_elements = parse_html_elements_with_context(&html_string, context)?;

    // Update the ElementData
    with_element_data(&this_obj, |element| {
        *element.inner_html.lock().unwrap() = html_string;
        let mut children = element.children.lock().unwrap();
        children.clear();
        children.extend(parsed_elements);
        drop(children);
        element.recompute_text_content();
        element.update_document_html_content();
    }, "Element.prototype.setHTMLUnsafe called on non-Element object")?;

    Ok(JsValue::undefined())
}

/// `Element.prototype.insertBefore(newNode, referenceNode)`
pub(super) fn insert_before_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.insertBefore called on non-object")
    })?;

    let new_node = args.get_or_undefined(0);
    let reference_node = args.get_or_undefined(1);

    let new_obj = new_node.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("insertBefore: newNode must be a Node")
    })?;

    // Set parent on new node
    let _ = with_element_data(&new_obj, |el| {
        el.set_parent_node(Some(this_obj.clone()));
    }, "");

    let ref_obj = if reference_node.is_null() || reference_node.is_undefined() {
        None
    } else {
        Some(reference_node.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("insertBefore: referenceNode must be a Node or null")
        })?)
    };

    with_element_data(&this_obj, |el| {
        el.insert_before(new_obj.clone(), ref_obj.as_ref());
    }, "Element.prototype.insertBefore called on non-Element object")?;

    // Check if the inserted node is a script element and execute it
    if is_script_element(&new_obj, context)? {
        execute_script_element(&new_obj, context)?;
    }

    Ok(new_node.clone())
}

/// `Element.prototype.replaceChild(newChild, oldChild)`
pub(super) fn replace_child_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.replaceChild called on non-object")
    })?;

    let new_child = args.get_or_undefined(0);
    let old_child = args.get_or_undefined(1);

    let new_obj = new_child.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("replaceChild: newChild must be a Node")
    })?;

    let old_obj = old_child.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("replaceChild: oldChild must be a Node")
    })?;

    // Set parent on new node
    let _ = with_element_data(&new_obj, |el| {
        el.set_parent_node(Some(this_obj.clone()));
    }, "");

    let replaced = with_element_data(&this_obj, |el| {
        el.replace_child(new_obj.clone(), &old_obj)
    }, "Element.prototype.replaceChild called on non-Element object")?;

    if let Some(replaced) = replaced {
        Ok(replaced.into())
    } else {
        Err(JsNativeError::error()
            .with_message("replaceChild: oldChild is not a child of this node")
            .into())
    }
}

/// `Element.prototype.cloneNode(deep)`
pub(super) fn clone_node(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.cloneNode called on non-object")
    })?;

    let deep = args.get_or_undefined(0).to_boolean();

    let cloned = with_element_data(&this_obj, |el| {
        el.clone_element(deep, context)
    }, "Element.prototype.cloneNode called on non-Element object")?;

    let cloned = cloned?;
    Ok(cloned.into())
}

/// `Element.prototype.contains(node)`
pub(super) fn contains_js(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.contains called on non-object")
    })?;

    let other = args.get_or_undefined(0);

    if other.is_null() || other.is_undefined() {
        return Ok(false.into());
    }

    if let Some(other_obj) = other.as_object() {
        // Check if it's the same node (compare by reference)
        if std::ptr::eq(this_obj.as_ref(), other_obj.as_ref()) {
            return Ok(true.into());
        }
        let result = with_element_data(&this_obj, |el| {
            el.contains_node(&other_obj)
        }, "Element.prototype.contains called on non-Element object")?;
        Ok(result.into())
    } else {
        Ok(false.into())
    }
}

// ============================================================================
// Context-aware HTML parsing for iframe support
// ============================================================================

/// Parse HTML string and create elements, properly handling iframes with context initialization
///
/// When parsing `<iframe>` tags via innerHTML, this function creates proper HTMLIFrameElement
/// instances with their contentWindow and contentDocument initialized, enabling:
/// - postMessage communication between windows
/// - window.parent/top/frameElement navigation
/// - Proper browsing context for Turnstile and similar scripts
pub fn parse_html_elements_with_context(
    html: &str,
    context: &mut Context,
) -> JsResult<Vec<JsObject>> {
    let mut elements = Vec::new();
    let mut current_pos = 0;
    let html_bytes = html.as_bytes();

    while current_pos < html_bytes.len() {
        if html_bytes[current_pos] == b'<' {
            // Find end of tag
            if let Some(tag_end) = html[current_pos..].find('>') {
                let tag_content = &html[current_pos + 1..current_pos + tag_end];

                if !tag_content.starts_with('/') { // Not a closing tag
                    // Parse opening tag
                    let parts: Vec<&str> = tag_content.split_whitespace().collect();
                    if let Some(tag_name) = parts.first() {
                        let tag_upper = tag_name.to_uppercase();

                        // Handle IFRAME specially - create with full context
                        if tag_upper == "IFRAME" {
                            let iframe = create_parsed_iframe(&parts[1..], context)?;
                            elements.push(iframe);
                        } else {
                            // Create normal element
                            let element_data = ElementData::with_tag_name(tag_upper);

                            // Parse attributes
                            for attr_part in parts.iter().skip(1) {
                                if let Some(eq_pos) = attr_part.find('=') {
                                    let attr_name = &attr_part[..eq_pos];
                                    let attr_value = &attr_part[eq_pos + 1..].trim_matches('"');
                                    element_data.set_attribute(attr_name.to_string(), attr_value.to_string());
                                }
                            }

                            // Create JsObject for the element with HTMLElement prototype chain
                            let prototype = context.intrinsics().constructors().html_element().prototype();
                            let element = JsObject::from_proto_and_data_with_shared_shape(
                                context.root_shape(),
                                prototype,
                                element_data,
                            );
                            elements.push(element.upcast());
                        }
                    }
                }

                current_pos += tag_end + 1;
            } else {
                current_pos += 1;
            }
        } else {
            // Text content - find next tag or end
            let text_start = current_pos;
            let text_end = html[current_pos..].find('<').map(|pos| current_pos + pos).unwrap_or(html.len());

            let text_content = html[text_start..text_end].trim();
            if !text_content.is_empty() {
                // Create text node as element with special tag and proper prototype
                let text_element = ElementData::with_tag_name("#text".to_string());
                text_element.set_text_content(text_content.to_string());

                let prototype = context.intrinsics().constructors().element().prototype();
                let text_obj = JsObject::from_proto_and_data_with_shared_shape(
                    context.root_shape(),
                    prototype,
                    text_element,
                );
                elements.push(text_obj.upcast());
            }

            current_pos = text_end;
        }
    }

    Ok(elements)
}

/// Create an iframe element from parsed HTML attributes with proper context initialization
///
/// This creates a full HTMLIFrameElement with:
/// - All parsed attributes (src, id, name, width, height, etc.)
/// - Initialized contentWindow with parent/top/frameElement set up
/// - Initialized contentDocument
/// - Registration in the window hierarchy
fn create_parsed_iframe(
    attrs: &[&str],
    context: &mut Context,
) -> JsResult<JsObject> {
    use boa_engine::builtins::BuiltInConstructor;
    use crate::dom::html_iframe_element::{HTMLIFrameElement, HTMLIFrameElementData, initialize_iframe_context};

    eprintln!("🔲 IFRAME: Creating iframe via innerHTML parsing...");

    // Create HTMLIFrameElement via constructor
    let iframe_constructor = context.intrinsics().constructors().html_iframe_element().constructor();
    let iframe = HTMLIFrameElement::constructor(
        &iframe_constructor.clone().into(),
        &[],
        context,
    )?;

    let iframe_obj = iframe.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Failed to create HTMLIFrameElement")
    })?.clone();

    // Parse attributes into separate collections to avoid borrow conflicts
    // We need to separate:
    // 1. Attributes that go to HTMLIFrameElementData (src, name, width, height, sandbox, allow)
    // 2. Attributes that get set on the JS object (id, generic attributes)
    let mut src_val = None;
    let mut name_val = None;
    let mut width_val = None;
    let mut height_val = None;
    let mut sandbox_val = None;
    let mut allow_val = None;
    let mut js_attrs: Vec<(String, String)> = Vec::new();

    for attr_part in attrs {
        let attr_part = attr_part.trim_end_matches('/'); // Handle self-closing tags
        if let Some(eq_pos) = attr_part.find('=') {
            let attr_name = &attr_part[..eq_pos];
            let attr_value = attr_part[eq_pos + 1..].trim_matches('"').trim_matches('\'');

            match attr_name.to_lowercase().as_str() {
                "src" => src_val = Some(attr_value.to_string()),
                "name" => name_val = Some(attr_value.to_string()),
                "width" => width_val = Some(attr_value.to_string()),
                "height" => height_val = Some(attr_value.to_string()),
                "sandbox" => sandbox_val = Some(attr_value.to_string()),
                "allow" => allow_val = Some(attr_value.to_string()),
                "id" => js_attrs.push(("id".to_string(), attr_value.to_string())),
                _ => js_attrs.push((attr_name.to_string(), attr_value.to_string())),
            }
        }
    }

    // Set HTMLIFrameElementData fields (no context needed, borrow-safe)
    if let Some(data) = iframe_obj.downcast_ref::<HTMLIFrameElementData>() {
        if let Some(ref v) = src_val {
            *data.get_src_mutex().lock().unwrap() = v.clone();
        }
        if let Some(v) = name_val {
            *data.get_name_mutex().lock().unwrap() = v;
        }
        if let Some(v) = width_val {
            *data.get_width_mutex().lock().unwrap() = v;
        }
        if let Some(v) = height_val {
            *data.get_height_mutex().lock().unwrap() = v;
        }
        if let Some(v) = sandbox_val {
            *data.get_sandbox_mutex().lock().unwrap() = v;
        }
        if let Some(v) = allow_val {
            *data.get_allow_mutex().lock().unwrap() = v;
        }
    }

    // Set JS object attributes (requires context, done after borrow is released)
    for (attr_name, attr_value) in js_attrs {
        iframe_obj.set(
            JsString::from(attr_name.as_str()),
            js_string!(attr_value.as_str()),
            false,
            context,
        )?;
    }

    // Initialize iframe context (creates contentWindow/contentDocument)
    initialize_iframe_context(&iframe_obj, context)?;

    eprintln!("🔲 IFRAME: Created iframe via innerHTML parsing - context initialized");

    // If the iframe has a src, trigger content loading
    if let Some(ref src) = src_val {
        if !src.is_empty() && src != "about:blank" && !src.starts_with("about:") {
            eprintln!("🔲 IFRAME: innerHTML iframe has src='{}', triggering load", src);
            if let Err(e) = crate::dom::html_iframe_element::load_iframe_content(&iframe_obj, src, context) {
                eprintln!("🔲 IFRAME: Load failed: {:?}", e);
                // Don't fail iframe creation if load fails
            }
        }
    }

    Ok(iframe_obj)
}
