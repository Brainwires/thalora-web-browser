//! HTMLTemplateElement Web API implementation
//!
//! The HTMLTemplateElement interface represents a <template> element.
//! The <template> element is used to hold client-side content that is not
//! rendered when the page loads but may be instantiated during runtime using JavaScript.
//!
//! https://html.spec.whatwg.org/multipage/scripting.html#the-template-element

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject, ObjectInitializer, FunctionObjectBuilder},
    property::Attribute,
    realm::Realm,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, NativeFunction,
};
use boa_gc::{Finalize, Trace};

/// JavaScript `HTMLTemplateElement` implementation.
#[derive(Debug, Copy, Clone)]
pub struct HTMLTemplateElement;

impl HTMLTemplateElement {
    /// Initialize the HTMLTemplateElement constructor in the global scope
    pub fn init(context: &mut Context) {
        // Create the HTMLTemplateElement constructor function
        let constructor = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_fn_ptr(Self::constructor),
        )
        .name(js_string!("HTMLTemplateElement"))
        .length(0)
        .constructor(true)
        .build();

        // Add prototype with content property getter
        let prototype = ObjectInitializer::new(context).build();

        // Add content property as a getter
        let content_getter = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_fn_ptr(Self::content_getter),
        )
        .length(0)
        .build();

        prototype
            .define_property_or_throw(
                js_string!("content"),
                boa_engine::property::PropertyDescriptor::builder()
                    .get(content_getter)
                    .enumerable(true)
                    .configurable(true),
                context,
            )
            .expect("Failed to define content property");

        // Set prototype on constructor
        constructor
            .set(js_string!("prototype"), prototype.clone(), false, context)
            .expect("Failed to set prototype");

        // Set constructor on prototype
        prototype
            .set(
                js_string!("constructor"),
                constructor.clone(),
                false,
                context,
            )
            .expect("Failed to set constructor");

        // Register globally
        context
            .register_global_property(
                js_string!("HTMLTemplateElement"),
                constructor,
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .expect("Failed to register HTMLTemplateElement");
    }

    /// Constructor function for HTMLTemplateElement
    fn constructor(
        _this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Create the template element object
        let template_obj = ObjectInitializer::new(context).build();

        // Create empty document fragment for content
        let content_fragment = Self::create_document_fragment(context)?;

        // Store content on the object
        template_obj.set(js_string!("__content__"), content_fragment, false, context)?;

        // Set tagName property
        template_obj.set(
            js_string!("tagName"),
            js_string!("TEMPLATE"),
            false,
            context,
        )?;

        // Set nodeName property
        template_obj.set(
            js_string!("nodeName"),
            js_string!("TEMPLATE"),
            false,
            context,
        )?;

        // Add content property getter
        let content_getter = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_fn_ptr(Self::content_getter),
        )
        .length(0)
        .build();

        template_obj
            .define_property_or_throw(
                js_string!("content"),
                boa_engine::property::PropertyDescriptor::builder()
                    .get(content_getter)
                    .enumerable(true)
                    .configurable(true),
                context,
            )
            .expect("Failed to define content property");

        Ok(template_obj.into())
    }

    /// Create a DocumentFragment for template content
    fn create_document_fragment(context: &mut Context) -> JsResult<JsValue> {
        let fragment = ObjectInitializer::new(context)
            .property(
                js_string!("nodeName"),
                js_string!("#document-fragment"),
                Attribute::default(),
            )
            .property(
                js_string!("nodeType"),
                11, // DOCUMENT_FRAGMENT_NODE
                Attribute::default(),
            )
            .build();

        // Add childNodes array
        let child_nodes = boa_engine::object::builtins::JsArray::new(context);
        fragment.set(js_string!("childNodes"), child_nodes, false, context)?;

        // Add children array
        let children = boa_engine::object::builtins::JsArray::new(context);
        fragment.set(js_string!("children"), children, false, context)?;

        // Add basic methods
        fragment.set(
            js_string!("appendChild"),
            FunctionObjectBuilder::new(
                context.realm(),
                NativeFunction::from_fn_ptr(Self::fragment_append_child),
            )
            .length(1)
            .build(),
            false,
            context,
        )?;

        fragment.set(
            js_string!("cloneNode"),
            FunctionObjectBuilder::new(
                context.realm(),
                NativeFunction::from_fn_ptr(Self::fragment_clone_node),
            )
            .length(1)
            .build(),
            false,
            context,
        )?;

        fragment.set(
            js_string!("querySelector"),
            FunctionObjectBuilder::new(
                context.realm(),
                NativeFunction::from_fn_ptr(Self::fragment_query_selector),
            )
            .length(1)
            .build(),
            false,
            context,
        )?;

        fragment.set(
            js_string!("querySelectorAll"),
            FunctionObjectBuilder::new(
                context.realm(),
                NativeFunction::from_fn_ptr(Self::fragment_query_selector_all),
            )
            .length(1)
            .build(),
            false,
            context,
        )?;

        Ok(fragment.into())
    }

    /// Getter for the content property
    fn content_getter(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLTemplateElement.content called on non-object")
        })?;

        // Return the stored content fragment
        let content = obj.get(js_string!("__content__"), context)?;
        if content.is_undefined() {
            // Create a new empty fragment if none exists
            Self::create_document_fragment(context)
        } else {
            Ok(content)
        }
    }

    /// appendChild for document fragment
    fn fragment_append_child(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("appendChild called on non-object")
        })?;

        let child = args.get_or_undefined(0);

        // Get childNodes array
        let child_nodes = obj.get(js_string!("childNodes"), context)?;
        if let Some(arr) = child_nodes.as_object() {
            let length = arr.get(js_string!("length"), context)?.to_u32(context)?;
            arr.set(length, child.clone(), false, context)?;
        }

        Ok(child.clone())
    }

    /// cloneNode for document fragment
    fn fragment_clone_node(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let _deep = args
            .get_or_undefined(0)
            .to_boolean();

        // Create a new fragment
        let new_fragment = Self::create_document_fragment(context)?;

        // In a real implementation, we would clone all children if deep is true
        Ok(new_fragment)
    }

    /// querySelector for document fragment
    fn fragment_query_selector(
        _this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // In a real implementation, this would search the fragment's children
        Ok(JsValue::null())
    }

    /// querySelectorAll for document fragment
    fn fragment_query_selector_all(
        _this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Return empty NodeList
        let array = boa_engine::object::builtins::JsArray::new(context);
        Ok(array.into())
    }
}
