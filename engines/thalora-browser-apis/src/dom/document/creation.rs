//! Element/node creation methods for Document
//!
//! createElement, createElementNS, createTextNode, createDocumentFragment,
//! createRange, createComment, createAttribute, hasFocus, execCommand,
//! createTreeWalker, createNodeIterator, svg_get_bbox

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor},
    object::JsObject,
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
    JsString, property::PropertyDescriptorBuilder
};

use super::types::DocumentData;
use super::canvas::canvas_get_context;
use super::canvas::canvas_to_data_url;

/// `Document.prototype.createElement(tagName)`
pub(super) fn create_element(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.createElement called on non-object")
    })?;

    let _document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.createElement called on non-Document object")
    })?;

    let tag_name = args.get_or_undefined(0).to_string(context)?;
    let tag_name_upper = tag_name.to_std_string_escaped().to_uppercase();

    // Create a proper Element object using Element constructor pattern
    let element_constructor = context.intrinsics().constructors().element().constructor();
    let element = crate::dom::element::Element::constructor(
        &element_constructor.clone().into(),
        &[],
        context,
    )?;

    // Get the Element object from the JsValue
    let element_obj = element.as_object().unwrap();

    // Add tagName property (this should be done by ElementData, but make it explicit)
    element_obj.define_property_or_throw(
        js_string!("tagName"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsString::from(tag_name_upper.as_str()))
            .build(),
        context,
    )?;

    // Set the tag name in the element data
    if let Some(element_data) = element_obj.downcast_ref::<crate::dom::element::ElementData>() {
        element_data.set_tag_name(tag_name_upper.clone());
    }

    // Add style property as a proper CSSStyleDeclaration
    let style_constructor = context.intrinsics().constructors().css_style_declaration().constructor();
    let style_val = crate::browser::cssom::CSSStyleDeclaration::constructor(
        &style_constructor.clone().into(),
        &[],
        context,
    )?;
    element_obj.define_property_or_throw(
        js_string!("style"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(style_val)
            .build(),
        context,
    )?;

    // Add Form-specific functionality for <form> elements
    if tag_name_upper == "FORM" {
        // Create elements collection that Google's code expects
        let elements_collection = JsObject::default(context.intrinsics());

        // Add common form controls as properties of elements collection
        // Google often checks for elements like 'q' (search query)
        let q_element = JsObject::default(context.intrinsics());
        q_element.define_property_or_throw(
            js_string!("value"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(js_string!(""))
                .build(),
            context,
        )?;

        elements_collection.define_property_or_throw(
            js_string!("q"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(q_element)
                .build(),
            context,
        )?;

        // Add elements collection to form
        element_obj.define_property_or_throw(
            js_string!("elements"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(elements_collection)
                .build(),
            context,
        )?;

        // Add getAttribute method that Google's code uses
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

        element_obj.define_property_or_throw(
            js_string!("getAttribute"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(get_attribute_func)
                .build(),
            context,
        )?;
    }

    // Add Button-specific functionality for <button> elements
    if tag_name_upper == "BUTTON" {
        // Add button-specific properties
        element_obj.define_property_or_throw(
            js_string!("type"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(js_string!("button"))
                .build(),
            context,
        )?;

        element_obj.define_property_or_throw(
            js_string!("value"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(js_string!(""))
                .build(),
            context,
        )?;
    }

    // Add Canvas-specific functionality for <canvas> elements
    if tag_name_upper == "CANVAS" {
        // Add width and height properties with default values
        element_obj.define_property_or_throw(
            js_string!("width"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(300) // Default canvas width
                .build(),
            context,
        )?;

        element_obj.define_property_or_throw(
            js_string!("height"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(150) // Default canvas height
                .build(),
            context,
        )?;

        // Add getContext method
        let get_context_func = BuiltInBuilder::callable(context.realm(), canvas_get_context)
            .name(js_string!("getContext"))
            .build();

        element_obj.define_property_or_throw(
            js_string!("getContext"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(get_context_func)
                .build(),
            context,
        )?;

        // Add toDataURL method
        let to_data_url_func = BuiltInBuilder::callable(context.realm(), canvas_to_data_url)
            .name(js_string!("toDataURL"))
            .build();

        element_obj.define_property_or_throw(
            js_string!("toDataURL"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(to_data_url_func)
                .build(),
            context,
        )?;
    }

    // Add IFrame-specific functionality for <iframe> elements
    if tag_name_upper == "IFRAME" {
        // Create an HTMLIFrameElement instead of generic Element
        let iframe_constructor = context.intrinsics().constructors().html_iframe_element().constructor();
        let iframe = crate::dom::html_iframe_element::HTMLIFrameElement::constructor(
            &iframe_constructor.clone().into(),
            &[],
            context,
        )?;

        let iframe_obj = iframe.as_object().unwrap();

        // Initialize the iframe's isolated browsing context (contentDocument/contentWindow)
        crate::dom::html_iframe_element::initialize_iframe_context(&iframe_obj, context)?;

        return Ok(iframe);
    }

    // Add Script-specific functionality for <script> elements
    if tag_name_upper == "SCRIPT" {
        // Create an HTMLScriptElement instead of generic Element
        let script_constructor = context.intrinsics().constructors().html_script_element().constructor();
        let script = crate::dom::html_script_element::HTMLScriptElement::constructor(
            &script_constructor.clone().into(),
            &[],
            context,
        )?;

        return Ok(script);
    }

    Ok(element)
}

/// `Document.prototype.createElementNS(namespaceURI, qualifiedName)`
/// Creates an element with the specified namespace URI and qualified name.
/// Used for creating SVG, MathML, and other namespaced elements.
pub(super) fn create_element_ns(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.createElementNS called on non-object")
    })?;

    let _document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.createElementNS called on non-Document object")
    })?;

    // Get namespace URI (can be null)
    let namespace_uri = args.get_or_undefined(0);
    let namespace_str = if namespace_uri.is_null() || namespace_uri.is_undefined() {
        None
    } else {
        Some(namespace_uri.to_string(context)?.to_std_string_escaped())
    };

    // Get qualified name (required)
    let qualified_name = args.get_or_undefined(1).to_string(context)?;
    let qualified_name_str = qualified_name.to_std_string_escaped();

    // Extract local name (after the colon if there's a prefix)
    let local_name = if let Some(colon_pos) = qualified_name_str.find(':') {
        &qualified_name_str[colon_pos + 1..]
    } else {
        &qualified_name_str
    };

    // Create a proper Element object using Element constructor
    let element_constructor = context.intrinsics().constructors().element().constructor();
    let element = crate::dom::element::Element::constructor(
        &element_constructor.clone().into(),
        &[],
        context,
    )?;

    let element_obj = element.as_object().unwrap();

    // For SVG namespace, use lowercase tag name; otherwise uppercase
    let is_svg = namespace_str.as_deref() == Some("http://www.w3.org/2000/svg");
    let tag_name = if is_svg {
        local_name.to_string()
    } else {
        local_name.to_uppercase()
    };

    // Set tagName property
    element_obj.define_property_or_throw(
        js_string!("tagName"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsString::from(tag_name.as_str()))
            .build(),
        context,
    )?;

    // Set namespace URI if provided
    if let Some(ref ns) = namespace_str {
        element_obj.define_property_or_throw(
            js_string!("namespaceURI"),
            PropertyDescriptorBuilder::new()
                .configurable(false)
                .enumerable(true)
                .writable(false)
                .value(JsString::from(ns.as_str()))
                .build(),
            context,
        )?;
    }

    // Set the element data
    if let Some(element_data) = element_obj.downcast_ref::<crate::dom::element::ElementData>() {
        element_data.set_tag_name(tag_name.clone());
        element_data.set_namespace_uri(namespace_str.clone());
    }

    // Add style property as a proper CSSStyleDeclaration
    let style_constructor_ns = context.intrinsics().constructors().css_style_declaration().constructor();
    let style_val_ns = crate::browser::cssom::CSSStyleDeclaration::constructor(
        &style_constructor_ns.clone().into(),
        &[],
        context,
    )?;
    element_obj.define_property_or_throw(
        js_string!("style"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(style_val_ns)
            .build(),
        context,
    )?;

    // For SVG elements, add SVG-specific properties
    if is_svg {
        // Add SVGAnimatedLength-like properties for common SVG attributes
        // These return objects with baseVal and animVal properties
        let svg_animated_props = ["width", "height", "x", "y", "cx", "cy", "r", "rx", "ry"];
        for prop in svg_animated_props {
            let animated_length = JsObject::default(context.intrinsics());
            animated_length.define_property_or_throw(
                js_string!("baseVal"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(JsValue::from(0))
                    .build(),
                context,
            )?;
            animated_length.define_property_or_throw(
                js_string!("animVal"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(JsValue::from(0))
                    .build(),
                context,
            )?;

            element_obj.define_property_or_throw(
                js_string!(prop),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(animated_length)
                    .build(),
                context,
            )?;
        }

        // Add getBBox method for SVG elements
        let get_bbox_func = BuiltInBuilder::callable(context.realm(), svg_get_bbox)
            .name(js_string!("getBBox"))
            .build();

        element_obj.define_property_or_throw(
            js_string!("getBBox"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(get_bbox_func)
                .build(),
            context,
        )?;
    }

    Ok(element)
}

/// SVG getBBox() implementation - returns bounding box for SVG elements
pub(super) fn svg_get_bbox(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Return a DOMRect-like object with x, y, width, height
    let bbox = JsObject::default(context.intrinsics());
    bbox.define_property_or_throw(
        js_string!("x"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(JsValue::from(0))
            .build(),
        context,
    )?;
    bbox.define_property_or_throw(
        js_string!("y"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(JsValue::from(0))
            .build(),
        context,
    )?;
    bbox.define_property_or_throw(
        js_string!("width"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(JsValue::from(0))
            .build(),
        context,
    )?;
    bbox.define_property_or_throw(
        js_string!("height"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(JsValue::from(0))
            .build(),
        context,
    )?;
    Ok(bbox.into())
}

/// `Document.prototype.createTextNode(data)`
pub(super) fn create_text_node(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let data = args.get_or_undefined(0).to_string(context)?;

    // Create a Text node using the Text constructor
    let text_constructor = context.intrinsics().constructors().text().constructor();
    let text = crate::dom::text::Text::constructor(
        &text_constructor.clone().into(),
        &[data.into()],
        context,
    )?;

    Ok(text)
}

/// `Document.prototype.createDocumentFragment()`
pub(super) fn create_document_fragment(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Create a DocumentFragment using the DocumentFragment constructor
    let fragment_constructor = context.intrinsics().constructors().document_fragment().constructor();
    let fragment = super::document_fragment::DocumentFragment::constructor(
        &fragment_constructor.clone().into(),
        &[],
        context,
    )?;

    Ok(fragment)
}

/// `Document.prototype.createRange()`
pub(super) fn create_range(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Create a Range using the Range constructor
    let range_constructor = context.intrinsics().constructors().range().constructor();
    let range = crate::dom::range::Range::constructor(
        &range_constructor.clone().into(),
        &[],
        context,
    )?;

    Ok(range)
}

/// `Document.prototype.createComment(data)`
pub(super) fn create_comment(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let data = args.get_or_undefined(0).to_string(context)?;

    // Create a Comment node using the Comment constructor
    let comment_constructor = context.intrinsics().constructors().comment().constructor();
    let comment = crate::dom::comment::Comment::constructor(
        &comment_constructor.clone().into(),
        &[data.into()],
        context,
    )?;

    Ok(comment)
}

/// `Document.prototype.createAttribute(name)`
pub(super) fn create_attribute(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let name = args.get_or_undefined(0).to_string(context)?;
    let name_str = name.to_std_string_escaped();

    // Create an Attr node using the Attr constructor
    let attr_constructor = context.intrinsics().constructors().attr().constructor();
    let attr = crate::dom::attr::Attr::constructor(
        &attr_constructor.clone().into(),
        &[],
        context,
    )?;

    // Set the attribute name
    if let Some(attr_obj) = attr.as_object() {
        if let Some(attr_data) = attr_obj.downcast_ref::<crate::dom::attr::AttrData>() {
            attr_data.set_name(name_str);
        }
    }

    Ok(attr)
}

/// `Document.prototype.hasFocus()`
pub(super) fn has_focus(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // In a headless browser context, the document always has focus
    Ok(JsValue::from(true))
}

/// `Document.prototype.execCommand(commandId, showUI, value)`
pub(super) fn exec_command(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let command_id = args.get_or_undefined(0).to_string(context)?;
    let _command_str = command_id.to_std_string_escaped();

    // execCommand is deprecated but still used by some sites
    // Return false to indicate the command was not executed
    // In a headless browser, we don't have editing capabilities
    Ok(JsValue::from(false))
}

/// `Document.prototype.createTreeWalker(root, whatToShow, filter)`
pub(super) fn create_tree_walker(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use crate::dom::treewalker::{TreeWalker, node_filter};

    // Get the root node (required)
    let root = args.get_or_undefined(0);
    let root_obj = root.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("createTreeWalker: root must be a Node")
    })?.clone();

    // Get whatToShow (optional, defaults to SHOW_ALL)
    let what_to_show = if args.len() > 1 && !args.get_or_undefined(1).is_undefined() {
        args.get_or_undefined(1).to_u32(context)?
    } else {
        node_filter::SHOW_ALL
    };

    // Get filter (optional)
    let filter = if args.len() > 2 && !args.get_or_undefined(2).is_null_or_undefined() {
        args.get_or_undefined(2).as_object().map(|o| o.clone())
    } else {
        None
    };

    let tree_walker = TreeWalker::create(root_obj, what_to_show, filter, context)?;
    Ok(tree_walker.into())
}

/// `Document.prototype.createNodeIterator(root, whatToShow, filter)`
pub(super) fn create_node_iterator(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use crate::dom::nodeiterator::NodeIterator;
    use crate::dom::treewalker::node_filter;

    // Get the root node (required)
    let root = args.get_or_undefined(0);
    let root_obj = root.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("createNodeIterator: root must be a Node")
    })?.clone();

    // Get whatToShow (optional, defaults to SHOW_ALL)
    let what_to_show = if args.len() > 1 && !args.get_or_undefined(1).is_undefined() {
        args.get_or_undefined(1).to_u32(context)?
    } else {
        node_filter::SHOW_ALL
    };

    // Get filter (optional)
    let filter = if args.len() > 2 && !args.get_or_undefined(2).is_null_or_undefined() {
        args.get_or_undefined(2).as_object().map(|o| o.clone())
    } else {
        None
    };

    let node_iterator = NodeIterator::create(root_obj, what_to_show, filter, context)?;
    Ok(node_iterator.into())
}
