//! Document collection getters and helpers
//!
//! documentElement, forms, images, links, scripts + helpers
//! add_html_collection_methods, add_get_attribute_method

use boa_engine::{
    builtins::BuiltInBuilder,
    object::JsObject,
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
    JsString, property::PropertyDescriptorBuilder
};
use std::collections::HashMap;

use super::types::DocumentData;

/// `Document.prototype.documentElement` getter
///
/// Returns the root `<html>` element. On first access, lazily builds a live DOM tree
/// from the document's HTML content using scraper. This tree is persistent — querySelector
/// returns references into it, and JS mutations (appendChild, etc.) modify it in place.
pub(super) fn get_document_element(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.documentElement called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.documentElement called on non-Document object")
    })?;

    // Return existing html element if already built
    if let Some(html) = document.get_element("html") {
        return Ok(html.into());
    }

    // Build DOM tree from stored HTML content (lazy initialization)
    let html_content = document.get_html_content();
    if !html_content.is_empty() {
        let elements_ref = document.elements.clone();
        // Release the borrow on document before calling build_dom_tree (needs context)
        drop(document);

        let root = super::dom_tree::build_dom_tree(&html_content, context)?;

        // Register all elements with IDs and special elements (html, head, body)
        super::dom_tree::register_tree_elements(&root, &elements_ref);

        return Ok(root.into());
    }

    // Fallback: create empty HTML element (no HTML content available)
    let element_constructor = context.intrinsics().constructors().element().constructor();
    let html_element = element_constructor.construct(&[], None, context)?;
    if let Some(elem_data) = html_element.downcast_ref::<crate::dom::element::ElementData>() {
        elem_data.set_tag_name("HTML".to_string());
    }

    // Re-borrow document to store the element
    let document2 = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.documentElement called on non-Document object")
    })?;
    document2.add_element("html".to_string(), html_element.clone());
    Ok(html_element.into())
}

/// `Document.prototype.forms` getter - returns HTMLCollection of all form elements
pub(super) fn get_forms(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.forms called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.forms called on non-Document object")
    })?;

    // Parse HTML content to find all form elements
    let html_content = document.get_html_content();
    let doc = scraper::Html::parse_document(&html_content);

    let mut forms = Vec::new();
    if let Ok(selector) = scraper::Selector::parse("form") {
        for form_element in doc.select(&selector) {
            // Create a form element object
            let element_constructor = context.intrinsics().constructors().element().constructor();
            if let Ok(form_obj) = element_constructor.construct(&[], None, context) {
                if let Some(elem_data) = form_obj.downcast_ref::<crate::dom::element::ElementData>() {
                    elem_data.set_tag_name("FORM".to_string());
                    // Set form attributes
                    if let Some(id) = form_element.value().attr("id") {
                        elem_data.set_id(id.to_string());
                    }
                    if let Some(name) = form_element.value().attr("name") {
                        elem_data.set_attribute("name".to_string(), name.to_string());
                    }
                    if let Some(action) = form_element.value().attr("action") {
                        elem_data.set_attribute("action".to_string(), action.to_string());
                    }
                    if let Some(method) = form_element.value().attr("method") {
                        elem_data.set_attribute("method".to_string(), method.to_string());
                    }
                }
                forms.push(JsValue::from(form_obj));
            }
        }
    }

    // Create HTMLCollection-like array
    let array = boa_engine::builtins::array::Array::create_array_from_list(forms, context);
    add_html_collection_methods(&array, context)?;
    Ok(array.into())
}

/// `Document.prototype.images` getter - returns HTMLCollection of all img elements
pub(super) fn get_images(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.images called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.images called on non-Document object")
    })?;

    let html_content = document.get_html_content();
    let doc = scraper::Html::parse_document(&html_content);

    let mut images = Vec::new();
    if let Ok(selector) = scraper::Selector::parse("img") {
        for img_element in doc.select(&selector) {
            let element_constructor = context.intrinsics().constructors().element().constructor();
            if let Ok(img_obj) = element_constructor.construct(&[], None, context) {
                if let Some(elem_data) = img_obj.downcast_ref::<crate::dom::element::ElementData>() {
                    elem_data.set_tag_name("IMG".to_string());
                    if let Some(src) = img_element.value().attr("src") {
                        elem_data.set_attribute("src".to_string(), src.to_string());
                    }
                    if let Some(alt) = img_element.value().attr("alt") {
                        elem_data.set_attribute("alt".to_string(), alt.to_string());
                    }
                    if let Some(id) = img_element.value().attr("id") {
                        elem_data.set_id(id.to_string());
                    }
                }
                images.push(JsValue::from(img_obj));
            }
        }
    }

    let array = boa_engine::builtins::array::Array::create_array_from_list(images, context);
    add_html_collection_methods(&array, context)?;
    Ok(array.into())
}

/// `Document.prototype.links` getter - returns HTMLCollection of all a and area elements with href
pub(super) fn get_links(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.links called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.links called on non-Document object")
    })?;

    let html_content = document.get_html_content();
    let doc = scraper::Html::parse_document(&html_content);

    let mut links = Vec::new();
    if let Ok(selector) = scraper::Selector::parse("a[href], area[href]") {
        for link_element in doc.select(&selector) {
            let element_constructor = context.intrinsics().constructors().element().constructor();
            if let Ok(link_obj) = element_constructor.construct(&[], None, context) {
                if let Some(elem_data) = link_obj.downcast_ref::<crate::dom::element::ElementData>() {
                    elem_data.set_tag_name(link_element.value().name().to_uppercase());
                    if let Some(href) = link_element.value().attr("href") {
                        elem_data.set_attribute("href".to_string(), href.to_string());
                    }
                    if let Some(id) = link_element.value().attr("id") {
                        elem_data.set_id(id.to_string());
                    }
                }
                links.push(JsValue::from(link_obj));
            }
        }
    }

    let array = boa_engine::builtins::array::Array::create_array_from_list(links, context);
    add_html_collection_methods(&array, context)?;
    Ok(array.into())
}

/// `Document.prototype.scripts` getter - returns HTMLCollection of all script elements
/// This includes both scripts from the static HTML and dynamically loaded scripts
pub(super) fn get_scripts(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.scripts called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.scripts called on non-Document object")
    })?;

    let html_content = document.get_html_content();
    let doc = scraper::Html::parse_document(&html_content);

    let mut scripts = Vec::new();

    // Track script sources to avoid duplicates (static HTML vs registered)
    let mut seen_srcs: std::collections::HashSet<String> = std::collections::HashSet::new();

    // First, get scripts from static HTML parsing
    if let Ok(selector) = scraper::Selector::parse("script") {
        for script_element in doc.select(&selector) {
            // Create proper HTMLScriptElement instead of generic Element
            let script_constructor = context.intrinsics().constructors().html_script_element().constructor();
            if let Ok(script_obj) = script_constructor.construct(&[], None, context) {
                // Collect all attributes from the HTML element
                let mut attrs: HashMap<String, String> = HashMap::new();
                for (key, value) in script_element.value().attrs() {
                    attrs.insert(key.to_string(), value.to_string());
                }

                // Track the src to avoid duplicates
                if let Some(src) = script_element.value().attr("src") {
                    seen_srcs.insert(src.to_string());
                }

                // Set attributes on the HTMLScriptElement
                if let Some(script_data) = script_obj.downcast_ref::<crate::dom::html_script_element::HTMLScriptElementData>() {
                    if let Some(src) = attrs.get("src") {
                        script_data.set_src(src.clone());
                    }
                    if let Some(type_) = attrs.get("type") {
                        script_data.set_type(type_.clone());
                    }
                    if attrs.contains_key("async") {
                        script_data.set_async(true);
                    }
                    if attrs.contains_key("defer") {
                        script_data.set_defer(true);
                    }
                    if let Some(id) = attrs.get("id") {
                        script_data.set_id(id.clone());
                    }
                    // Set all custom attributes (including data-* attributes)
                    for (key, value) in &attrs {
                        script_data.set_attribute(key, value.clone());
                    }
                }

                // Also set the text content (inline script)
                let text_content: String = script_element.text().collect();
                if !text_content.is_empty() {
                    if let Some(script_data) = script_obj.downcast_ref::<crate::dom::html_script_element::HTMLScriptElementData>() {
                        script_data.set_text(text_content);
                    }
                }

                scripts.push(JsValue::from(script_obj));
            }
        }
    }

    // Then, add scripts from the loaded_scripts registry that aren't already in the HTML
    let loaded_scripts = document.get_loaded_scripts();
    for entry in loaded_scripts {
        // Skip if we already have this script from HTML parsing
        if let Some(ref src) = entry.src {
            if seen_srcs.contains(src) {
                continue;
            }
        }

        // Create HTMLScriptElement for the registered script
        let script_constructor = context.intrinsics().constructors().html_script_element().constructor();
        if let Ok(script_obj) = script_constructor.construct(&[], None, context) {
            if let Some(script_data) = script_obj.downcast_ref::<crate::dom::html_script_element::HTMLScriptElementData>() {
                if let Some(ref src) = entry.src {
                    script_data.set_src(src.clone());
                }
                if let Some(ref type_) = entry.script_type {
                    script_data.set_type(type_.clone());
                }
                script_data.set_async(entry.async_);
                script_data.set_defer(entry.defer);
                script_data.set_text(entry.text.clone());

                // Set all custom attributes
                for (key, value) in &entry.attributes {
                    script_data.set_attribute(key, value.clone());
                }
            }
            scripts.push(JsValue::from(script_obj));
        }
    }

    let array = boa_engine::builtins::array::Array::create_array_from_list(scripts, context);
    add_html_collection_methods(&array, context)?;
    Ok(array.into())
}

/// Helper function to add HTMLCollection methods to an array
pub(super) fn add_html_collection_methods(array: &JsObject, context: &mut Context) -> JsResult<()> {
    // Add item() method
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

    // Add namedItem() method
    let named_item_fn = BuiltInBuilder::callable(context.realm(), |this, args, ctx| {
        let name = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();
        if let Some(arr) = this.as_object() {
            if let Ok(length) = arr.get(js_string!("length"), ctx) {
                let len = length.to_u32(ctx)?;
                for i in 0..len {
                    if let Ok(item) = arr.get(i, ctx) {
                        if let Some(item_obj) = item.as_object() {
                            // Check id attribute
                            if let Ok(id) = item_obj.get(js_string!("id"), ctx) {
                                if id.to_string(ctx)?.to_std_string_escaped() == name {
                                    return Ok(item);
                                }
                            }
                            // Check name attribute
                            if let Ok(elem_name) = item_obj.get(js_string!("name"), ctx) {
                                if elem_name.to_string(ctx)?.to_std_string_escaped() == name {
                                    return Ok(item);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(JsValue::null())
    })
    .name(js_string!("namedItem"))
    .build();
    array.set(js_string!("namedItem"), named_item_fn, false, context)?;

    Ok(())
}

/// Helper function to add getAttribute method to an element with captured attributes
pub(super) fn add_get_attribute_method(element: &JsObject, attrs: HashMap<String, String>, context: &mut Context) -> JsResult<()> {
    // Store attributes on the element object itself as a hidden property
    // This allows getAttribute to access them
    let attrs_obj = JsObject::default(context.intrinsics());
    for (key, value) in &attrs {
        attrs_obj.set(js_string!(key.clone()), js_string!(value.clone()), false, context)?;
    }
    element.set(js_string!("__attributes__"), attrs_obj, false, context)?;

    // Create getAttribute method that reads from __attributes__
    let get_attr_fn = BuiltInBuilder::callable(context.realm(), |this, args, ctx| {
        let name = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();

        if let Some(this_obj) = this.as_object() {
            if let Ok(attrs_val) = this_obj.get(js_string!("__attributes__"), ctx) {
                if let Some(attrs_obj) = attrs_val.as_object() {
                    if let Ok(value) = attrs_obj.get(js_string!(name.clone()), ctx) {
                        if !value.is_undefined() {
                            return Ok(value);
                        }
                    }
                }
            }
        }
        Ok(JsValue::null())
    })
    .name(js_string!("getAttribute"))
    .build();

    element.set(js_string!("getAttribute"), get_attr_fn, false, context)?;

    Ok(())
}
