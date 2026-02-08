//! DOM tree builder — builds a persistent tree of ElementData JsObjects from HTML
//!
//! Uses `scraper` to parse the document HTML, then creates a live DOM tree where:
//! - Each element is a proper JsObject with ElementData
//! - Parent-child relationships are wired via children/parent_node
//! - Elements with `id` attributes are registered in the document's element map
//! - `modified_by_js` is left `false` so serialize_to_html uses cached innerHTML
//!
//! This tree enables JS frameworks (Vue, React, etc.) to find elements via
//! querySelector, modify them (appendChild, etc.), and have those changes
//! reflected when documentElement.outerHTML is serialized.

use boa_engine::{
    object::JsObject,
    Context, JsResult,
};
use scraper::node::Node;

use crate::dom::element::ElementData;

/// Build a complete DOM tree from HTML, returning the root `<html>` element.
///
/// The returned JsObject is an ElementData with tag name "HTML", whose children
/// vector contains the full tree. Each node's innerHTML is set to the original
/// HTML content so that unmodified subtrees serialize correctly.
pub fn build_dom_tree(html: &str, context: &mut Context) -> JsResult<JsObject> {
    let doc = scraper::Html::parse_document(html);
    let root = doc.root_element(); // the <html> element
    build_element_from_ref(&root, context)
}

/// Recursively build a JsObject element tree from a scraper ElementRef.
fn build_element_from_ref(
    element_ref: &scraper::ElementRef,
    context: &mut Context,
) -> JsResult<JsObject> {
    let tag_name = element_ref.value().name().to_uppercase();

    // Create ElementData with tag name and attributes
    let element_data = ElementData::with_tag_name(tag_name.clone());

    // Set all attributes
    for (attr_name, attr_value) in element_ref.value().attrs() {
        element_data.set_attribute(attr_name.to_string(), attr_value.to_string());
    }

    // Set innerHTML from the original parsed content (used for serialization fallback)
    let inner_html = element_ref.inner_html();
    *element_data.inner_html.lock().unwrap() = inner_html;

    // Set text content (all descendant text concatenated)
    let text_content: String = element_ref.text().collect();
    element_data.set_text_content(text_content);

    // Create JsObject with HTMLElement prototype
    let prototype = context.intrinsics().constructors().html_element().prototype();
    let element_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        prototype,
        element_data,
    );
    let element_obj = element_obj.upcast();

    // Iterate direct children and build child elements
    // We push directly to the children vec (not via append_child JS method)
    // so modified_by_js stays false
    let mut child_elements: Vec<JsObject> = Vec::new();

    for child_node in element_ref.children() {
        match child_node.value() {
            Node::Element(_) => {
                if let Some(child_element_ref) = scraper::ElementRef::wrap(child_node) {
                    let child_obj = build_element_from_ref(&child_element_ref, context)?;

                    // Set parent reference
                    if let Some(child_data) = child_obj.downcast_ref::<ElementData>() {
                        child_data.set_parent_node(Some(element_obj.clone()));
                    }

                    child_elements.push(child_obj);
                }
            }
            Node::Text(text) => {
                let text_str = text.text.trim();
                if !text_str.is_empty() {
                    // Create a text node element
                    let text_data = ElementData::with_tag_name("#text".to_string());
                    text_data.set_text_content(text_str.to_string());

                    let text_prototype = context.intrinsics().constructors().element().prototype();
                    let text_obj = JsObject::from_proto_and_data_with_shared_shape(
                        context.root_shape(),
                        text_prototype,
                        text_data,
                    );
                    let text_obj = text_obj.upcast();

                    // Set parent reference
                    if let Some(text_el_data) = text_obj.downcast_ref::<ElementData>() {
                        text_el_data.set_parent_node(Some(element_obj.clone()));
                    }

                    child_elements.push(text_obj);
                }
            }
            _ => {
                // Comments, processing instructions, etc. — skip
            }
        }
    }

    // Wire children directly into the children vector (bypasses modified_by_js)
    if let Some(el_data) = element_obj.downcast_ref::<ElementData>() {
        let mut children = el_data.children.lock().unwrap();
        *children = child_elements;
    }

    // Set up sibling relationships
    if let Some(el_data) = element_obj.downcast_ref::<ElementData>() {
        let children = el_data.children.lock().unwrap();
        for i in 0..children.len() {
            if i > 0 {
                if let Some(prev_data) = children[i - 1].downcast_ref::<ElementData>() {
                    prev_data.set_next_sibling(Some(children[i].clone()));
                }
                if let Some(curr_data) = children[i].downcast_ref::<ElementData>() {
                    curr_data.set_previous_sibling(Some(children[i - 1].clone()));
                }
            }
        }
    }

    Ok(element_obj)
}

/// Collect all elements with `id` attributes from the tree and register them
/// in the document's element map. Also registers special elements like "body", "head".
pub fn register_tree_elements(
    root: &JsObject,
    document_elements: &std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, JsObject>>>,
) {
    if let Some(el_data) = root.downcast_ref::<ElementData>() {
        let tag = el_data.get_tag_name().to_lowercase();
        let id = el_data.get_id();

        let mut elements = document_elements.lock().unwrap();

        // Register by tag name for special elements
        match tag.as_str() {
            "html" => { elements.insert("html".to_string(), root.clone()); }
            "head" => { elements.insert("head".to_string(), root.clone()); }
            "body" => { elements.insert("body".to_string(), root.clone()); }
            _ => {}
        }

        // Register by ID
        if !id.is_empty() {
            elements.insert(id, root.clone());
        }

        // Recurse into children
        let children = el_data.children.lock().unwrap();
        drop(elements); // Release lock before recursing
        for child in children.iter() {
            register_tree_elements(child, document_elements);
        }
    }
}
