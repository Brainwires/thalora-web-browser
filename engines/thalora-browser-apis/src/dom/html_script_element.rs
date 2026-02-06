//! HTMLScriptElement implementation for Boa
//!
//! Implements the HTMLScriptElement interface as defined in:
//! https://html.spec.whatwg.org/multipage/scripting.html#the-script-element

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, js_string,
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::dom::element::ElementData;

/// JavaScript `HTMLScriptElement` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct HTMLScriptElement;

impl IntrinsicObject for HTMLScriptElement {
    fn init(realm: &Realm) {
        let src_getter = BuiltInBuilder::callable(realm, get_src)
            .name(js_string!("get src"))
            .build();
        let src_setter = BuiltInBuilder::callable(realm, set_src)
            .name(js_string!("set src"))
            .build();

        let type_getter = BuiltInBuilder::callable(realm, get_type)
            .name(js_string!("get type"))
            .build();
        let type_setter = BuiltInBuilder::callable(realm, set_type)
            .name(js_string!("set type"))
            .build();

        let async_getter = BuiltInBuilder::callable(realm, get_async)
            .name(js_string!("get async"))
            .build();
        let async_setter = BuiltInBuilder::callable(realm, set_async)
            .name(js_string!("set async"))
            .build();

        let defer_getter = BuiltInBuilder::callable(realm, get_defer)
            .name(js_string!("get defer"))
            .build();
        let defer_setter = BuiltInBuilder::callable(realm, set_defer)
            .name(js_string!("set defer"))
            .build();

        let text_getter = BuiltInBuilder::callable(realm, get_text)
            .name(js_string!("get text"))
            .build();
        let text_setter = BuiltInBuilder::callable(realm, set_text)
            .name(js_string!("set text"))
            .build();

        let id_getter = BuiltInBuilder::callable(realm, get_id)
            .name(js_string!("get id"))
            .build();
        let id_setter = BuiltInBuilder::callable(realm, set_id)
            .name(js_string!("set id"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(realm.intrinsics().constructors().html_element().prototype()))
            .accessor(
                js_string!("src"),
                Some(src_getter),
                Some(src_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("type"),
                Some(type_getter),
                Some(type_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("async"),
                Some(async_getter),
                Some(async_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("defer"),
                Some(defer_getter),
                Some(defer_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("text"),
                Some(text_getter),
                Some(text_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("id"),
                Some(id_getter),
                Some(id_setter),
                Attribute::CONFIGURABLE,
            )
            // getAttribute/setAttribute/hasAttribute/removeAttribute are inherited
            // from Element.prototype via the dispatch helper
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLScriptElement {
    const NAME: JsString = StaticJsStrings::HTML_SCRIPT_ELEMENT;
}

impl BuiltInConstructor for HTMLScriptElement {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_script_element;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("HTMLScriptElement constructor requires 'new'")
                .into());
        }

        let proto = get_prototype_from_constructor(new_target, StandardConstructors::html_script_element, context)?;
        let script_data = HTMLScriptElementData::new();
        let script_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            script_data,
        );

        let script_generic = script_obj.upcast();

        // Set nodeType as own property (tagName/nodeName come from Element.prototype via dispatch)
        script_generic.set(js_string!("nodeType"), 1, false, context)?;

        Ok(script_generic.into())
    }
}

/// Internal data for HTMLScriptElement instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct HTMLScriptElementData {
    /// Base element data — provides tagName, id, className, attributes, children, etc.
    #[unsafe_ignore_trace]
    pub(crate) element: ElementData,
    #[unsafe_ignore_trace]
    src: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    type_: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    async_: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    defer: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    text: Arc<Mutex<String>>,
}

impl HTMLScriptElementData {
    pub fn new() -> Self {
        Self {
            element: ElementData::with_tag_name("SCRIPT".to_string()),
            src: Arc::new(Mutex::new(String::new())),
            type_: Arc::new(Mutex::new(String::new())),
            async_: Arc::new(Mutex::new(false)),
            defer: Arc::new(Mutex::new(false)),
            text: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Create a new HTMLScriptElementData with pre-populated attributes
    pub fn with_attributes(attrs: HashMap<String, String>) -> Self {
        let mut data = Self::new();

        // Set known fields from attributes
        for (key, value) in &attrs {
            match key.as_str() {
                "src" => *data.src.lock().unwrap() = value.clone(),
                "type" => *data.type_.lock().unwrap() = value.clone(),
                "async" => *data.async_.lock().unwrap() = true,
                "defer" => *data.defer.lock().unwrap() = true,
                "id" => {
                    data.element.set_id(value.clone());
                }
                _ => {}
            }
            // Store all attributes in ElementData
            data.element.set_attribute(key.clone(), value.clone());
        }

        data
    }

    /// Access the embedded ElementData
    pub fn element_data(&self) -> &ElementData {
        &self.element
    }

    pub fn get_src(&self) -> String {
        self.src.lock().unwrap().clone()
    }

    pub fn set_src(&self, src: String) {
        *self.src.lock().unwrap() = src.clone();
        self.element.set_attribute("src".to_string(), src);
    }

    pub fn get_type(&self) -> String {
        self.type_.lock().unwrap().clone()
    }

    pub fn set_type(&self, type_: String) {
        *self.type_.lock().unwrap() = type_.clone();
        self.element.set_attribute("type".to_string(), type_);
    }

    pub fn get_async(&self) -> bool {
        *self.async_.lock().unwrap()
    }

    pub fn set_async(&self, async_: bool) {
        *self.async_.lock().unwrap() = async_;
        if async_ {
            self.element.set_attribute("async".to_string(), "".to_string());
        } else {
            self.element.remove_attribute("async");
        }
    }

    pub fn get_defer(&self) -> bool {
        *self.defer.lock().unwrap()
    }

    pub fn set_defer(&self, defer: bool) {
        *self.defer.lock().unwrap() = defer;
        if defer {
            self.element.set_attribute("defer".to_string(), "".to_string());
        } else {
            self.element.remove_attribute("defer");
        }
    }

    pub fn get_text(&self) -> String {
        self.text.lock().unwrap().clone()
    }

    pub fn set_text(&self, text: String) {
        *self.text.lock().unwrap() = text;
    }

    pub fn get_id(&self) -> String {
        self.element.get_id()
    }

    pub fn set_id(&self, id: String) {
        self.element.set_id(id.clone());
        self.element.set_attribute("id".to_string(), id);
    }

    pub fn get_attribute(&self, name: &str) -> Option<String> {
        // First check known script-specific fields
        match name {
            "src" => Some(self.get_src()),
            "type" => Some(self.get_type()),
            "async" => if self.get_async() { Some("".to_string()) } else { None },
            "defer" => if self.get_defer() { Some("".to_string()) } else { None },
            // For everything else, delegate to ElementData
            _ => self.element.get_attribute(name)
        }
    }

    pub fn set_attribute(&self, name: &str, value: String) {
        // Update known script-specific fields if applicable
        match name {
            "src" => self.set_src(value.clone()),
            "type" => self.set_type(value.clone()),
            "async" => self.set_async(true),
            "defer" => self.set_defer(true),
            "id" => self.set_id(value.clone()),
            _ => {
                self.element.set_attribute(name.to_string(), value);
            }
        }
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        match name {
            "src" => !self.get_src().is_empty(),
            "type" => !self.get_type().is_empty(),
            "async" => self.get_async(),
            "defer" => self.get_defer(),
            // For everything else, delegate to ElementData
            _ => self.element.has_attribute(name)
        }
    }

    pub fn remove_attribute(&self, name: &str) {
        match name {
            "src" => *self.src.lock().unwrap() = String::new(),
            "type" => *self.type_.lock().unwrap() = String::new(),
            "async" => *self.async_.lock().unwrap() = false,
            "defer" => *self.defer.lock().unwrap() = false,
            _ => {}
        }
        self.element.remove_attribute(name);
    }
}

fn get_src(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.src called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        Ok(js_string!(data.get_src()).into())
    } else {
        Ok(js_string!("").into())
    }
}

fn set_src(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.src called on non-object")
    })?;

    let src = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        data.set_src(src);
    }

    Ok(JsValue::undefined())
}

fn get_type(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.type called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        Ok(js_string!(data.get_type()).into())
    } else {
        Ok(js_string!("").into())
    }
}

fn set_type(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.type called on non-object")
    })?;

    let type_ = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        data.set_type(type_);
    }

    Ok(JsValue::undefined())
}

fn get_async(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.async called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        Ok(JsValue::from(data.get_async()))
    } else {
        Ok(JsValue::from(false))
    }
}

fn set_async(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.async called on non-object")
    })?;

    let async_ = args.get_or_undefined(0).to_boolean();

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        data.set_async(async_);
    }

    Ok(JsValue::undefined())
}

fn get_defer(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.defer called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        Ok(JsValue::from(data.get_defer()))
    } else {
        Ok(JsValue::from(false))
    }
}

fn set_defer(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.defer called on non-object")
    })?;

    let defer = args.get_or_undefined(0).to_boolean();

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        data.set_defer(defer);
    }

    Ok(JsValue::undefined())
}

fn get_text(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.text called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        Ok(js_string!(data.get_text()).into())
    } else {
        Ok(js_string!("").into())
    }
}

fn set_text(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.text called on non-object")
    })?;

    let text = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        data.set_text(text);
    }

    Ok(JsValue::undefined())
}

fn get_id(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.id called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        Ok(js_string!(data.get_id()).into())
    } else {
        Ok(js_string!("").into())
    }
}

fn set_id(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.id called on non-object")
    })?;

    let id = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        data.set_id(id);
    }

    Ok(JsValue::undefined())
}

fn get_attribute(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.getAttribute called on non-object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        match data.get_attribute(&name) {
            Some(value) => Ok(js_string!(value).into()),
            None => Ok(JsValue::null())
        }
    } else {
        Ok(JsValue::null())
    }
}

fn set_attribute(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.setAttribute called on non-object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    let value = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        data.set_attribute(&name, value);
    }

    Ok(JsValue::undefined())
}

fn has_attribute(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.hasAttribute called on non-object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        Ok(JsValue::from(data.has_attribute(&name)))
    } else {
        Ok(JsValue::from(false))
    }
}

fn remove_attribute(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.removeAttribute called on non-object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        data.remove_attribute(&name);
    }

    Ok(JsValue::undefined())
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;

    fn create_test_context() -> Context {
        let mut context = Context::default();
        crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
        context
    }

    #[test]
    fn test_html_script_element_exists() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes("typeof HTMLScriptElement === 'function'")).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_html_script_element_constructor() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes(r#"
            const script = new HTMLScriptElement();
            script.tagName === 'SCRIPT';
        "#)).unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
