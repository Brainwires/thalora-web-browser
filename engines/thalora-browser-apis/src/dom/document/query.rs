//! Query/selection methods for Document
//!
//! getElementById, querySelector, querySelectorAll, getElementsByClassName,
//! getElementsByTagName, getElementsByName, elementFromPoint, elementsFromPoint,
//! scrollToDocument, dispatchTrustedMouseEventDocument
//! Helpers: create_real_element_from_html, create_all_real_elements_from_html

use boa_engine::{
    builtins::BuiltInBuilder,
    object::JsObject,
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
    JsString, property::PropertyDescriptorBuilder
};
use std::collections::HashMap;

use super::types::DocumentData;
use super::events::dispatch_event;
use super::collections::get_scripts;

/// `Document.prototype.getElementById(id)`
///
/// Checks the document element registry first (includes elements registered from
/// the live DOM tree), then falls back to searching the live tree by ID.
pub(super) fn get_element_by_id(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.getElementById called on non-object")
    })?;

    let id = args.get_or_undefined(0).to_string(context)?;
    let id_str = id.to_std_string_escaped();

    // Ensure DOM tree is built
    {
        let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Document.prototype.getElementById called on non-Document object")
        })?;

        if document.get_element("html").is_none() {
            let html_content = document.get_html_content();
            if !html_content.is_empty() {
                let elements_ref = document.elements.clone();
                drop(document);

                let root = super::dom_tree::build_dom_tree(&html_content, context)?;
                super::dom_tree::register_tree_elements(&root, &elements_ref);
            }
        }
    }

    // Re-borrow after potential tree build
    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.getElementById called on non-Document object")
    })?;

    // Search the live tree first — this returns the SAME JsObject reference that
    // querySelector('#id') would return, ensuring identity (===) equality.
    // The element registry can hold stale references if the tree was rebuilt.
    if let Some(root) = document.get_element("html") {
        let selector = format!("#{}", id_str);
        let found = crate::dom::element::with_element_data(&root, |ed| {
            ed.query_selector(&selector)
        }, "not element");

        if let Ok(Some(found)) = found {
            return Ok(found.into());
        }
    }

    // Fallback to registry for special elements (html, head, body)
    if let Some(element) = document.get_element(&id_str) {
        return Ok(element.into());
    }

    Ok(JsValue::null())
}

/// `Document.prototype.querySelector(selector)`
///
/// Searches the live DOM tree first (returns persistent references that JS can mutate),
/// then falls back to HTML parsing for complex selectors.
pub(super) fn query_selector(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.querySelector called on non-object")
    })?;

    let selector = args.get_or_undefined(0).to_string(context)?;
    let selector_str = selector.to_std_string_escaped();
    eprintln!("DEBUG: query_selector selector: {}", selector_str);

    // Ensure DOM tree is built (triggers lazy build if needed)
    {
        let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Document.prototype.querySelector called on non-Document object")
        })?;

        if document.get_element("html").is_none() {
            let html_content = document.get_html_content();
            if !html_content.is_empty() {
                let elements_ref = document.elements.clone();
                drop(document);

                let root = super::dom_tree::build_dom_tree(&html_content, context)?;
                super::dom_tree::register_tree_elements(&root, &elements_ref);
            }
        }
    }

    // Re-borrow document after potential tree build
    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.querySelector called on non-Document object")
    })?;

    // Try live DOM tree first (returns persistent references)
    if let Some(root) = document.get_element("html") {
        // For simple selectors (#id, .class, tag), use ElementData::query_selector
        // which walks the live tree and returns references (not copies)
        let found = crate::dom::element::with_element_data(&root, |ed| {
            ed.query_selector(&selector_str)
        }, "not element");

        if let Ok(Some(found)) = found {
            eprintln!("DEBUG: query_selector found in live tree: {}", selector_str);
            return Ok(found.into());
        }
    }

    // Fallback: parse HTML for complex selectors not handled by matches_selector
    let html_content = document.get_html_content();
    eprintln!("DEBUG: query_selector fallback to HTML parsing for: {}", selector_str);

    if let Some(element) = create_real_element_from_html(context, &selector_str, &html_content)? {
        return Ok(element.into());
    }

    eprintln!("DEBUG: query_selector returning null - no element found");
    Ok(JsValue::null())
}

/// Real DOM element creation using scraper library and actual HTML content
pub(super) fn create_real_element_from_html(context: &mut Context, selector: &str, html_content: &str) -> JsResult<Option<JsObject>> {
    // Use the scraper crate to parse real HTML and find elements
    let document = scraper::Html::parse_document(html_content);

    if let Ok(css_selector) = scraper::Selector::parse(selector) {
        if let Some(element_ref) = document.select(&css_selector).next() {
            // Get real properties from the actual HTML element
            let tag_name = element_ref.value().name().to_uppercase();
            eprintln!("DEBUG: querySelector creating element with tagName: {}", tag_name);

            // Create a proper Element using ElementData with correct prototype
            // This ensures getters (tagName, className, etc.) work correctly
            let element_data = crate::dom::element::ElementData::with_tag_name(tag_name.clone());

            // Set attributes from the parsed HTML
            for (attr_name, attr_value) in element_ref.value().attrs() {
                element_data.set_attribute(attr_name.to_string(), attr_value.to_string());
            }

            // Set text content
            let text_content: String = element_ref.text().collect();
            element_data.set_text_content(text_content);

            // Set innerHTML
            let inner_html = element_ref.inner_html();
            element_data.set_inner_html(inner_html);

            // Create JsObject with proper prototype chain
            // Use HTMLElement prototype for HTML elements (ensures instanceof HTMLElement works)
            // This ensures methods like dispatchEvent, getBoundingClientRect work correctly
            let prototype = context.intrinsics().constructors().html_element().prototype();
            let typed_element_obj = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                prototype,
                element_data,
            );

            // Convert to generic JsObject to use .set() method for form handling
            let element_obj = typed_element_obj.upcast();

            eprintln!("DEBUG: Element created with proper ElementData, tagName: {}", tag_name);

            // Add value property for input elements (needs special handling)
            if tag_name == "INPUT" {
                if let Some(value) = element_ref.value().attr("value") {
                    element_obj.set(js_string!("value"), js_string!(value), false, context)?;
                } else {
                    element_obj.set(js_string!("value"), js_string!(""), false, context)?;
                }

                // Add name property for input elements (needed for form.elements access)
                if let Some(name) = element_ref.value().attr("name") {
                    element_obj.set(js_string!("name"), js_string!(name), false, context)?;
                }
            }

            // Add form-specific functionality for FORM elements from HTML
            if tag_name == "FORM" {
                // Create elements collection
                let elements_collection = context.intrinsics().constructors().object().constructor();

                // Find all input elements within this form using the HTML content
                let form_selector = scraper::Selector::parse("input").unwrap();

                // Parse the inner HTML of this form to find inputs
                let form_inner_html = element_ref.inner_html();
                let form_doc = scraper::Html::parse_fragment(&form_inner_html);

                for input_element in form_doc.select(&form_selector) {
                    if let Some(input_name) = input_element.value().attr("name") {
                        // Create input element object
                        let input_obj = context.intrinsics().constructors().object().constructor();

                        // Add value property
                        if let Some(input_value) = input_element.value().attr("value") {
                            input_obj.set(js_string!("value"), js_string!(input_value), false, context)?;
                        } else {
                            input_obj.set(js_string!("value"), js_string!(""), false, context)?;
                        }

                        // Add name property
                        input_obj.set(js_string!("name"), js_string!(input_name), false, context)?;

                        // Add input type
                        if let Some(input_type) = input_element.value().attr("type") {
                            input_obj.set(js_string!("type"), js_string!(input_type), false, context)?;
                        } else {
                            input_obj.set(js_string!("type"), js_string!("text"), false, context)?;
                        }

                        // Add this input to the elements collection by name
                        elements_collection.set(js_string!(input_name), input_obj, false, context)?;
                    }
                }

                // Add elements collection to the form
                element_obj.set(js_string!("elements"), elements_collection, false, context)?;

                // Add getAttribute method that Google's code needs
                let get_attribute_func = BuiltInBuilder::callable(context.realm(), |this, args, ctx| {
                    let attr_name = args.get_or_undefined(0).to_string(ctx)?;
                    let attr_name_str = attr_name.to_std_string_escaped();

                    // Return common attributes that Google checks
                    match attr_name_str.as_str() {
                        "data-submitfalse" => Ok(JsValue::null()), // Google checks this
                        _ => Ok(JsValue::null())
                    }
                })
                .name(js_string!("getAttribute"))
                .build();

                element_obj.set(js_string!("getAttribute"), get_attribute_func, false, context)?;
            }

            return Ok(Some(element_obj));
        }
    }

    Ok(None)
}

/// Real DOM elements creation using scraper library to find all matching elements
pub(super) fn create_all_real_elements_from_html(context: &mut Context, selector: &str, html_content: &str) -> JsResult<Vec<JsValue>> {
    let mut elements = Vec::new();

    // Use the scraper crate to parse real HTML and find all elements
    let document = scraper::Html::parse_document(html_content);

    if let Ok(css_selector) = scraper::Selector::parse(selector) {
        for element_ref in document.select(&css_selector) {
            // Get real properties from the actual HTML element
            let tag_name = element_ref.value().name().to_uppercase();

            // Create a proper Element using ElementData with correct prototype
            let element_data = crate::dom::element::ElementData::with_tag_name(tag_name.clone());

            // Set attributes from the parsed HTML
            for (attr_name, attr_value) in element_ref.value().attrs() {
                element_data.set_attribute(attr_name.to_string(), attr_value.to_string());
            }

            // Set text content
            let text_content: String = element_ref.text().collect();
            element_data.set_text_content(text_content);

            // Set innerHTML
            let inner_html = element_ref.inner_html();
            element_data.set_inner_html(inner_html);

            // Create JsObject with HTMLElement prototype chain (ensures instanceof HTMLElement works)
            let prototype = context.intrinsics().constructors().html_element().prototype();
            let element_obj = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                prototype,
                element_data,
            );

            elements.push(element_obj.upcast().into());
        }
    }

    Ok(elements)
}

/// `Document.prototype.querySelectorAll(selector)`
pub(super) fn query_selector_all(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.querySelectorAll called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.querySelectorAll called on non-Document object")
    })?;

    let selector = args.get_or_undefined(0).to_string(context)?;
    let selector_str = selector.to_std_string_escaped();

    // Get the HTML content from the document
    let html_content = document.get_html_content();

    // Use real DOM implementation with scraper library to find all matching elements
    let elements = create_all_real_elements_from_html(context, &selector_str, &html_content)?;

    // Convert JsValue elements to JsObject for NodeList
    let nodes: Vec<JsObject> = elements.into_iter()
        .filter_map(|v| v.as_object())
        .collect();
    let nodelist = crate::dom::nodelist::NodeList::create_from_nodes(nodes, false, context)?;
    Ok(nodelist.into())
}

// ============================================================================
// New DOM Query Methods (Phase 6.1)
// ============================================================================

/// `Document.prototype.getElementsByClassName(classNames)`
pub(super) fn get_elements_by_class_name(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.getElementsByClassName called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.getElementsByClassName called on non-Document object")
    })?;

    let class_names = args.get_or_undefined(0).to_string(context)?;
    let class_names_str = class_names.to_std_string_escaped();

    // Parse class names (space-separated)
    let classes: Vec<&str> = class_names_str.split_whitespace().collect();

    // Get HTML content and parse (use parse_document, not parse_fragment,
    // because fragment parsing uses <body> context which drops <head> elements per HTML5 spec)
    let html_content = document.html_content.lock().unwrap().clone();
    let fragment = scraper::Html::parse_document(&html_content);

    // Build CSS selector for matching all classes
    let selector_str = classes.iter()
        .map(|c| format!(".{}", c))
        .collect::<Vec<_>>()
        .join("");

    let result = JsObject::default(context.intrinsics());
    let mut index = 0u32;

    if let Ok(selector) = scraper::Selector::parse(&selector_str) {
        for element in fragment.select(&selector) {
            // Create proper ElementData with correct prototype
            let tag_name = element.value().name().to_uppercase();
            let element_data = crate::dom::element::ElementData::with_tag_name(tag_name.clone());

            // Set attributes from the parsed HTML (including class)
            for (attr_name, attr_value) in element.value().attrs() {
                element_data.set_attribute(attr_name.to_string(), attr_value.to_string());
            }

            // Set innerHTML
            element_data.set_inner_html(element.inner_html());

            // Create JsObject with HTMLElement prototype chain (ensures instanceof HTMLElement works)
            let prototype = context.intrinsics().constructors().html_element().prototype();
            let element_obj = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                prototype,
                element_data,
            );

            result.define_property_or_throw(
                index,
                PropertyDescriptorBuilder::new()
                    .value(element_obj.upcast())
                    .writable(false)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                context,
            )?;
            index += 1;
        }
    }

    // Set length property
    result.define_property_or_throw(
        js_string!("length"),
        PropertyDescriptorBuilder::new()
            .value(JsValue::from(index))
            .writable(false)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;

    Ok(result.into())
}

/// `Document.prototype.getElementsByTagName(tagName)`
pub(super) fn get_elements_by_tag_name(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.getElementsByTagName called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.getElementsByTagName called on non-Document object")
    })?;

    let tag_name = args.get_or_undefined(0).to_string(context)?;
    let tag_name_str = tag_name.to_std_string_escaped().to_lowercase();

    // Special handling for "script" tag - use proper HTMLScriptElement with full attributes
    if tag_name_str == "script" {
        return get_scripts(this, args, context);
    }

    // Get HTML content and parse (use parse_document, not parse_fragment,
    // because fragment parsing uses <body> context which drops <head> elements per HTML5 spec)
    let html_content = document.html_content.lock().unwrap().clone();
    let fragment = scraper::Html::parse_document(&html_content);

    let result = JsObject::default(context.intrinsics());
    let mut index = 0u32;

    // Handle "*" to get all elements
    let selector_str = if tag_name_str == "*" {
        "*".to_string()
    } else {
        tag_name_str.clone()
    };

    if let Ok(selector) = scraper::Selector::parse(&selector_str) {
        for element in fragment.select(&selector) {
            // Create proper ElementData with correct prototype
            let tag_name = element.value().name().to_uppercase();
            let element_data = crate::dom::element::ElementData::with_tag_name(tag_name.clone());

            // Set attributes from the parsed HTML
            for (attr_name, attr_value) in element.value().attrs() {
                element_data.set_attribute(attr_name.to_string(), attr_value.to_string());
            }

            // Set innerHTML
            element_data.set_inner_html(element.inner_html());

            // Create JsObject with HTMLElement prototype chain (ensures instanceof HTMLElement works)
            let prototype = context.intrinsics().constructors().html_element().prototype();
            let element_obj = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                prototype,
                element_data,
            );

            result.define_property_or_throw(
                index,
                PropertyDescriptorBuilder::new()
                    .value(element_obj.upcast())
                    .writable(false)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                context,
            )?;
            index += 1;
        }
    }

    result.define_property_or_throw(
        js_string!("length"),
        PropertyDescriptorBuilder::new()
            .value(JsValue::from(index))
            .writable(false)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;

    Ok(result.into())
}

/// `Document.prototype.getElementsByName(name)`
pub(super) fn get_elements_by_name(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.getElementsByName called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.getElementsByName called on non-Document object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    let name_str = name.to_std_string_escaped();

    let html_content = document.html_content.lock().unwrap().clone();
    let fragment = scraper::Html::parse_document(&html_content);

    let result = JsObject::default(context.intrinsics());
    let mut index = 0u32;

    let selector_str = format!("[name=\"{}\"]", name_str);

    if let Ok(selector) = scraper::Selector::parse(&selector_str) {
        for element in fragment.select(&selector) {
            // Create proper ElementData with correct prototype
            let tag_name = element.value().name().to_uppercase();
            let element_data = crate::dom::element::ElementData::with_tag_name(tag_name.clone());

            // Set attributes from the parsed HTML
            for (attr_name, attr_value) in element.value().attrs() {
                element_data.set_attribute(attr_name.to_string(), attr_value.to_string());
            }

            // Create JsObject with HTMLElement prototype chain (ensures instanceof HTMLElement works)
            let prototype = context.intrinsics().constructors().html_element().prototype();
            let element_obj = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                prototype,
                element_data,
            );

            result.define_property_or_throw(
                index,
                PropertyDescriptorBuilder::new()
                    .value(element_obj.upcast())
                    .writable(false)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                context,
            )?;
            index += 1;
        }
    }

    result.define_property_or_throw(
        js_string!("length"),
        PropertyDescriptorBuilder::new()
            .value(JsValue::from(index))
            .writable(false)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;

    Ok(result.into())
}

/// `Document.prototype.elementFromPoint(x, y)`
/// Returns the topmost Element at the specified coordinates (relative to viewport).
/// Used by Cloudflare Turnstile for bot detection during mouse interactions.
/// https://drafts.csswg.org/cssom-view/#dom-document-elementfrompoint
pub(super) fn element_from_point(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.elementFromPoint called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.elementFromPoint called on non-Document object")
    })?;

    // Get x and y coordinates
    let x = args.get_or_undefined(0).to_number(context)?;
    let y = args.get_or_undefined(1).to_number(context)?;

    // If coordinates are negative, return null per spec
    if x < 0.0 || y < 0.0 {
        return Ok(JsValue::null());
    }

    // Get viewport dimensions
    let viewport_width = crate::layout_registry::get_viewport_width();
    let viewport_height = crate::layout_registry::get_viewport_height();

    // If outside viewport, return null
    if x > viewport_width || y > viewport_height {
        return Ok(JsValue::null());
    }

    // Get HTML content to find elements
    let html_content = document.get_html_content();

    // In a real browser, this would do hit-testing based on rendered layout.
    // For our headless implementation, we return the body element as a reasonable
    // fallback for bot detection purposes. This allows Turnstile's mouse event
    // validation to find a target element.
    if !html_content.is_empty() {
        // Parse HTML and return body element
        let parsed_doc = scraper::Html::parse_document(&html_content);
        if let Ok(body_selector) = scraper::Selector::parse("body") {
            if let Some(body_ref) = parsed_doc.select(&body_selector).next() {
                let element_constructor = context.intrinsics().constructors().element().constructor();
                let element_obj = element_constructor.construct(&[], Some(&element_constructor), context)?;

                element_obj.set(js_string!("tagName"), js_string!("BODY"), false, context)?;
                element_obj.set(js_string!("nodeType"), 1, false, context)?;

                // Copy attributes
                for (attr_name, attr_value) in body_ref.value().attrs() {
                    element_obj.set(js_string!(attr_name), js_string!(attr_value), false, context)?;
                }

                return Ok(element_obj.into());
            }
        }
    }

    // If no body found, return null
    Ok(JsValue::null())
}

/// `Document.prototype.elementsFromPoint(x, y)`
/// Returns an array of all Elements at the specified coordinates.
/// https://drafts.csswg.org/cssom-view/#dom-document-elementsfrompoint
pub(super) fn elements_from_point(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Get the topmost element
    let top_element = element_from_point(this, args, context)?;

    // Create an array with the element (simplified - real impl would return all stacked elements)
    if !top_element.is_null() {
        let array = boa_engine::builtins::array::Array::array_create(1, None, context)?;
        array.set(0, top_element, true, context)?;
        Ok(array.into())
    } else {
        let array = boa_engine::builtins::array::Array::array_create(0, None, context)?;
        Ok(array.into())
    }
}

/// `Document.prototype.scrollTo(x, y)` or `Document.prototype.scrollTo(options)`
/// In browsers, this scrolls the viewport (delegates to window.scrollTo)
pub(super) fn scroll_to_document(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
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

    // Delegate to window.scrollTo via global object
    if let Ok(window_val) = context.global_object().get(js_string!("window"), context) {
        if let Some(window_obj) = window_val.as_object() {
            let _ = window_obj.set(js_string!("scrollX"), x, false, context);
            let _ = window_obj.set(js_string!("scrollY"), y, false, context);
        }
    }

    Ok(JsValue::undefined())
}

/// `Document.prototype.__dispatchTrustedMouseEvent(eventType, clientX, clientY, options?)`
/// Dispatches a trusted mouse event. Used for Cloudflare Turnstile and similar bot detection.
pub(super) fn dispatch_trusted_mouse_event_document(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use crate::events::ui_events::MouseEventData;

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("__dispatchTrustedMouseEvent called on non-object")
    })?;

    // Verify this is a Document
    if this_obj.downcast_ref::<DocumentData>().is_none() {
        return Err(JsNativeError::typ()
            .with_message("__dispatchTrustedMouseEvent called on non-Document object")
            .into());
    }

    // Get event type
    let event_type = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    // Get coordinates
    let client_x = args.get_or_undefined(1).to_number(context).unwrap_or(0.0);
    let client_y = args.get_or_undefined(2).to_number(context).unwrap_or(0.0);

    // Get optional parameters
    let options = args.get_or_undefined(3);
    let (button, buttons, ctrl_key, shift_key, alt_key, meta_key) = if options.is_object() {
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
        (button, buttons, ctrl_key, shift_key, alt_key, meta_key)
    } else {
        let buttons = if event_type.contains("down") || event_type == "click" { 1 } else { 0 };
        (0, buttons, false, false, false, false)
    };

    // Determine event properties
    let (bubbles, cancelable) = match event_type.as_str() {
        "click" | "dblclick" | "mousedown" | "mouseup" | "mousemove"
        | "mouseover" | "mouseout" | "mouseenter" | "mouseleave" => (true, true),
        _ => (true, false),
    };

    // Create trusted mouse event data
    let mut mouse_event = MouseEventData::new_trusted_with_coords(
        event_type.clone(),
        bubbles,
        cancelable,
        client_x,
        client_y,
        client_x,  // screen_x (same as clientX for simplicity)
        client_y,  // screen_y
        client_x,  // page_x
        client_y,  // page_y
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

    // Dispatch to the document using dispatchEvent
    dispatch_event(this, &[event_obj.upcast().into()], context)?;

    Ok(true.into())
}
