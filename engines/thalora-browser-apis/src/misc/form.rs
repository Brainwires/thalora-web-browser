//! HTMLFormElement and Form Control implementations for Boa
//!
//! Complete implementation of HTML Form elements following WHATWG HTML spec
//! https://html.spec.whatwg.org/multipage/forms.html#the-form-element

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    property::{Attribute, PropertyDescriptorBuilder},
    realm::Realm,
    string::{JsString, StaticJsStrings},
    value::JsValue,
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ============================================================================
// ValidityState - Form validation state object
// ============================================================================

/// ValidityState represents the validity states of a form control
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct ValidityState {
    #[unsafe_ignore_trace]
    value_missing: bool,
    #[unsafe_ignore_trace]
    type_mismatch: bool,
    #[unsafe_ignore_trace]
    pattern_mismatch: bool,
    #[unsafe_ignore_trace]
    too_long: bool,
    #[unsafe_ignore_trace]
    too_short: bool,
    #[unsafe_ignore_trace]
    range_underflow: bool,
    #[unsafe_ignore_trace]
    range_overflow: bool,
    #[unsafe_ignore_trace]
    step_mismatch: bool,
    #[unsafe_ignore_trace]
    bad_input: bool,
    #[unsafe_ignore_trace]
    custom_error: bool,
}

impl ValidityState {
    pub fn new() -> Self {
        Self {
            value_missing: false,
            type_mismatch: false,
            pattern_mismatch: false,
            too_long: false,
            too_short: false,
            range_underflow: false,
            range_overflow: false,
            step_mismatch: false,
            bad_input: false,
            custom_error: false,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.value_missing
            && !self.type_mismatch
            && !self.pattern_mismatch
            && !self.too_long
            && !self.too_short
            && !self.range_underflow
            && !self.range_overflow
            && !self.step_mismatch
            && !self.bad_input
            && !self.custom_error
    }
}

impl IntrinsicObject for ValidityState {
    fn init(realm: &Realm) {
        let valid_getter = BuiltInBuilder::callable(realm, validity_valid_getter)
            .name(js_string!("get valid"))
            .build();
        let value_missing_getter = BuiltInBuilder::callable(realm, validity_value_missing_getter)
            .name(js_string!("get valueMissing"))
            .build();
        let type_mismatch_getter = BuiltInBuilder::callable(realm, validity_type_mismatch_getter)
            .name(js_string!("get typeMismatch"))
            .build();
        let pattern_mismatch_getter =
            BuiltInBuilder::callable(realm, validity_pattern_mismatch_getter)
                .name(js_string!("get patternMismatch"))
                .build();
        let too_long_getter = BuiltInBuilder::callable(realm, validity_too_long_getter)
            .name(js_string!("get tooLong"))
            .build();
        let too_short_getter = BuiltInBuilder::callable(realm, validity_too_short_getter)
            .name(js_string!("get tooShort"))
            .build();
        let range_underflow_getter =
            BuiltInBuilder::callable(realm, validity_range_underflow_getter)
                .name(js_string!("get rangeUnderflow"))
                .build();
        let range_overflow_getter = BuiltInBuilder::callable(realm, validity_range_overflow_getter)
            .name(js_string!("get rangeOverflow"))
            .build();
        let step_mismatch_getter = BuiltInBuilder::callable(realm, validity_step_mismatch_getter)
            .name(js_string!("get stepMismatch"))
            .build();
        let bad_input_getter = BuiltInBuilder::callable(realm, validity_bad_input_getter)
            .name(js_string!("get badInput"))
            .build();
        let custom_error_getter = BuiltInBuilder::callable(realm, validity_custom_error_getter)
            .name(js_string!("get customError"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("valid"),
                Some(valid_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("valueMissing"),
                Some(value_missing_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("typeMismatch"),
                Some(type_mismatch_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("patternMismatch"),
                Some(pattern_mismatch_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("tooLong"),
                Some(too_long_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("tooShort"),
                Some(too_short_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("rangeUnderflow"),
                Some(range_underflow_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("rangeOverflow"),
                Some(range_overflow_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("stepMismatch"),
                Some(step_mismatch_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("badInput"),
                Some(bad_input_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("customError"),
                Some(custom_error_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for ValidityState {
    const NAME: JsString = js_string!("ValidityState");
}

impl BuiltInConstructor for ValidityState {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 22; // Accessors on prototype
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::validity_state;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("ValidityState constructor cannot be called directly")
            .into())
    }
}

// ValidityState getters
fn validity_valid_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(validity) = obj.downcast_ref::<ValidityState>() {
        return Ok(JsValue::from(validity.is_valid()));
    }
    Ok(JsValue::from(true))
}

fn validity_value_missing_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(validity) = obj.downcast_ref::<ValidityState>() {
        return Ok(JsValue::from(validity.value_missing));
    }
    Ok(JsValue::from(false))
}

fn validity_type_mismatch_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(validity) = obj.downcast_ref::<ValidityState>() {
        return Ok(JsValue::from(validity.type_mismatch));
    }
    Ok(JsValue::from(false))
}

fn validity_pattern_mismatch_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(validity) = obj.downcast_ref::<ValidityState>() {
        return Ok(JsValue::from(validity.pattern_mismatch));
    }
    Ok(JsValue::from(false))
}

fn validity_too_long_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(validity) = obj.downcast_ref::<ValidityState>() {
        return Ok(JsValue::from(validity.too_long));
    }
    Ok(JsValue::from(false))
}

fn validity_too_short_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(validity) = obj.downcast_ref::<ValidityState>() {
        return Ok(JsValue::from(validity.too_short));
    }
    Ok(JsValue::from(false))
}

fn validity_range_underflow_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(validity) = obj.downcast_ref::<ValidityState>() {
        return Ok(JsValue::from(validity.range_underflow));
    }
    Ok(JsValue::from(false))
}

fn validity_range_overflow_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(validity) = obj.downcast_ref::<ValidityState>() {
        return Ok(JsValue::from(validity.range_overflow));
    }
    Ok(JsValue::from(false))
}

fn validity_step_mismatch_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(validity) = obj.downcast_ref::<ValidityState>() {
        return Ok(JsValue::from(validity.step_mismatch));
    }
    Ok(JsValue::from(false))
}

fn validity_bad_input_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(validity) = obj.downcast_ref::<ValidityState>() {
        return Ok(JsValue::from(validity.bad_input));
    }
    Ok(JsValue::from(false))
}

fn validity_custom_error_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(validity) = obj.downcast_ref::<ValidityState>() {
        return Ok(JsValue::from(validity.custom_error));
    }
    Ok(JsValue::from(false))
}

// ============================================================================
// HTMLFormElement - Complete form element implementation
// ============================================================================

/// HTMLFormElement implementation
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct HTMLFormElement {
    /// Form controls collection by name/id
    #[unsafe_ignore_trace]
    pub elements: Arc<Mutex<HashMap<String, JsObject>>>,
    /// Form controls collection by index
    #[unsafe_ignore_trace]
    pub elements_by_index: Arc<Mutex<Vec<JsObject>>>,
    /// Form action URL
    #[unsafe_ignore_trace]
    pub action: Arc<Mutex<String>>,
    /// Form method (GET, POST, etc.)
    #[unsafe_ignore_trace]
    pub method: Arc<Mutex<String>>,
    /// Form name
    #[unsafe_ignore_trace]
    pub name: Arc<Mutex<String>>,
    /// Form encoding type
    #[unsafe_ignore_trace]
    pub enctype: Arc<Mutex<String>>,
    /// Form target
    #[unsafe_ignore_trace]
    pub target: Arc<Mutex<String>>,
    /// No validation flag
    #[unsafe_ignore_trace]
    pub no_validate: Arc<Mutex<bool>>,
    /// Autocomplete setting
    #[unsafe_ignore_trace]
    pub autocomplete: Arc<Mutex<String>>,
}

impl HTMLFormElement {
    /// Create a new HTMLFormElement
    pub fn new() -> Self {
        Self {
            elements: Arc::new(Mutex::new(HashMap::new())),
            elements_by_index: Arc::new(Mutex::new(Vec::new())),
            action: Arc::new(Mutex::new(String::new())),
            method: Arc::new(Mutex::new("get".to_string())),
            name: Arc::new(Mutex::new(String::new())),
            enctype: Arc::new(Mutex::new("application/x-www-form-urlencoded".to_string())),
            target: Arc::new(Mutex::new(String::new())),
            no_validate: Arc::new(Mutex::new(false)),
            autocomplete: Arc::new(Mutex::new("on".to_string())),
        }
    }

    /// Add a form control element
    pub fn add_element(&self, name: String, element: JsObject) {
        self.elements.lock().unwrap().insert(name, element.clone());
        self.elements_by_index.lock().unwrap().push(element);
    }

    /// Get element by name
    pub fn get_element_by_name(&self, name: &str) -> Option<JsObject> {
        self.elements.lock().unwrap().get(name).cloned()
    }

    /// Get element by index
    pub fn get_element_by_index(&self, index: usize) -> Option<JsObject> {
        self.elements_by_index.lock().unwrap().get(index).cloned()
    }

    /// Get elements count
    pub fn elements_length(&self) -> usize {
        self.elements_by_index.lock().unwrap().len()
    }
}

impl IntrinsicObject for HTMLFormElement {
    fn init(realm: &Realm) {
        // Property getters/setters
        let action_getter = BuiltInBuilder::callable(realm, form_action_getter)
            .name(js_string!("get action"))
            .build();
        let action_setter = BuiltInBuilder::callable(realm, form_action_setter)
            .name(js_string!("set action"))
            .build();
        let method_getter = BuiltInBuilder::callable(realm, form_method_getter)
            .name(js_string!("get method"))
            .build();
        let method_setter = BuiltInBuilder::callable(realm, form_method_setter)
            .name(js_string!("set method"))
            .build();
        let name_getter = BuiltInBuilder::callable(realm, form_name_getter)
            .name(js_string!("get name"))
            .build();
        let name_setter = BuiltInBuilder::callable(realm, form_name_setter)
            .name(js_string!("set name"))
            .build();
        let enctype_getter = BuiltInBuilder::callable(realm, form_enctype_getter)
            .name(js_string!("get enctype"))
            .build();
        let enctype_setter = BuiltInBuilder::callable(realm, form_enctype_setter)
            .name(js_string!("set enctype"))
            .build();
        let target_getter = BuiltInBuilder::callable(realm, form_target_getter)
            .name(js_string!("get target"))
            .build();
        let target_setter = BuiltInBuilder::callable(realm, form_target_setter)
            .name(js_string!("set target"))
            .build();
        let novalidate_getter = BuiltInBuilder::callable(realm, form_novalidate_getter)
            .name(js_string!("get noValidate"))
            .build();
        let novalidate_setter = BuiltInBuilder::callable(realm, form_novalidate_setter)
            .name(js_string!("set noValidate"))
            .build();
        let length_getter = BuiltInBuilder::callable(realm, form_length_getter)
            .name(js_string!("get length"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("action"),
                Some(action_getter),
                Some(action_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("method"),
                Some(method_getter),
                Some(method_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("name"),
                Some(name_getter),
                Some(name_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("enctype"),
                Some(enctype_getter),
                Some(enctype_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("target"),
                Some(target_getter),
                Some(target_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("noValidate"),
                Some(novalidate_getter),
                Some(novalidate_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("length"),
                Some(length_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(Self::submit, js_string!("submit"), 0)
            .method(Self::reset, js_string!("reset"), 0)
            .method(Self::check_validity, js_string!("checkValidity"), 0)
            .method(Self::report_validity, js_string!("reportValidity"), 0)
            .method(Self::request_submit, js_string!("requestSubmit"), 0)
            .static_method(Self::named_getter, js_string!("namedItem"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLFormElement {
    const NAME: JsString = StaticJsStrings::FORM_ELEMENT;
}

impl BuiltInConstructor for HTMLFormElement {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_form_element;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling HTMLFormElement constructor without `new` is forbidden")
                .into());
        }

        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::html_form_element,
            context,
        )?;

        let form = HTMLFormElement::new();
        let form_obj =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, form);

        // Add elements collection property
        let form_generic = form_obj.upcast();
        let elements_collection = HTMLFormControlsCollection::new(form_generic.clone());
        let elements_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            elements_collection,
        );

        form_generic.define_property_or_throw(
            js_string!("elements"),
            PropertyDescriptorBuilder::new()
                .value(elements_obj)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
            context,
        )?;

        Ok(form_generic.into())
    }
}

// HTMLFormElement property getters/setters
fn form_action_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(form) = obj.downcast_ref::<HTMLFormElement>() {
        return Ok(JsValue::from(js_string!(
            form.action.lock().unwrap().clone()
        )));
    }
    Ok(JsValue::from(js_string!("")))
}

fn form_action_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(form) = obj.downcast_ref::<HTMLFormElement>() {
        let value = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        *form.action.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn form_method_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(form) = obj.downcast_ref::<HTMLFormElement>() {
        return Ok(JsValue::from(js_string!(
            form.method.lock().unwrap().clone()
        )));
    }
    Ok(JsValue::from(js_string!("get")))
}

fn form_method_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(form) = obj.downcast_ref::<HTMLFormElement>() {
        let value = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped()
            .to_lowercase();
        // Normalize method
        let normalized = match value.as_str() {
            "post" => "post",
            "dialog" => "dialog",
            _ => "get",
        };
        *form.method.lock().unwrap() = normalized.to_string();
    }
    Ok(JsValue::undefined())
}

fn form_name_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(form) = obj.downcast_ref::<HTMLFormElement>() {
        return Ok(JsValue::from(js_string!(form.name.lock().unwrap().clone())));
    }
    Ok(JsValue::from(js_string!("")))
}

fn form_name_setter(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(form) = obj.downcast_ref::<HTMLFormElement>() {
        let value = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        *form.name.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn form_enctype_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(form) = obj.downcast_ref::<HTMLFormElement>() {
        return Ok(JsValue::from(js_string!(
            form.enctype.lock().unwrap().clone()
        )));
    }
    Ok(JsValue::from(js_string!(
        "application/x-www-form-urlencoded"
    )))
}

fn form_enctype_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(form) = obj.downcast_ref::<HTMLFormElement>() {
        let value = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped()
            .to_lowercase();
        // Normalize enctype
        let normalized = match value.as_str() {
            "multipart/form-data" => "multipart/form-data",
            "text/plain" => "text/plain",
            _ => "application/x-www-form-urlencoded",
        };
        *form.enctype.lock().unwrap() = normalized.to_string();
    }
    Ok(JsValue::undefined())
}

fn form_target_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(form) = obj.downcast_ref::<HTMLFormElement>() {
        return Ok(JsValue::from(js_string!(
            form.target.lock().unwrap().clone()
        )));
    }
    Ok(JsValue::from(js_string!("")))
}

fn form_target_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(form) = obj.downcast_ref::<HTMLFormElement>() {
        let value = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        *form.target.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn form_novalidate_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(form) = obj.downcast_ref::<HTMLFormElement>() {
        return Ok(JsValue::from(*form.no_validate.lock().unwrap()));
    }
    Ok(JsValue::from(false))
}

fn form_novalidate_setter(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(form) = obj.downcast_ref::<HTMLFormElement>() {
        let value = args.get_or_undefined(0).to_boolean();
        *form.no_validate.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn form_length_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(form) = obj.downcast_ref::<HTMLFormElement>() {
        return Ok(JsValue::from(form.elements_length() as i32));
    }
    Ok(JsValue::from(0))
}

impl HTMLFormElement {
    /// `HTMLFormElement.prototype.namedItem(name)`
    fn named_getter(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLFormElement.namedItem called on non-object")
        })?;

        if let Some(form) = this_obj.downcast_ref::<HTMLFormElement>() {
            let name = args.get_or_undefined(0).to_string(context)?;
            let name_str = name.to_std_string_escaped();

            if let Some(element) = form.get_element_by_name(&name_str) {
                return Ok(element.into());
            }
        }

        Ok(JsValue::null())
    }

    /// HTMLFormElement.submit() method
    fn submit(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let _this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLFormElement.submit called on non-object")
        })?;

        // In a headless browser, form submission triggers navigation
        // For now, log the intent
        eprintln!("HTMLFormElement.submit() called - would submit form");

        Ok(JsValue::undefined())
    }

    /// HTMLFormElement.reset() method
    fn reset(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let _this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLFormElement.reset called on non-object")
        })?;

        // Reset all form elements to their default values
        eprintln!("HTMLFormElement.reset() called - would reset form");

        Ok(JsValue::undefined())
    }

    /// HTMLFormElement.checkValidity() method
    fn check_validity(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLFormElement.checkValidity called on non-object")
        })?;

        if let Some(form) = this_obj.downcast_ref::<HTMLFormElement>() {
            // Check if noValidate is set
            if *form.no_validate.lock().unwrap() {
                return Ok(JsValue::from(true));
            }

            // Iterate through all form elements and check validity
            let elements = form.elements_by_index.lock().unwrap().clone();
            for element in &elements {
                // Try each form control type
                if let Some(input) = element.downcast_ref::<HTMLInputElement>() {
                    if !input.check_validity_state().is_valid() {
                        return Ok(JsValue::from(false));
                    }
                } else if let Some(select) = element.downcast_ref::<HTMLSelectElement>() {
                    let required = *select.required.lock().unwrap();
                    let selected_index = *select.selected_index.lock().unwrap();
                    if required && selected_index < 0 {
                        return Ok(JsValue::from(false));
                    }
                } else if let Some(textarea) = element.downcast_ref::<HTMLTextAreaElement>() {
                    let required = *textarea.required.lock().unwrap();
                    let value = textarea.value.lock().unwrap();
                    if required && value.is_empty() {
                        return Ok(JsValue::from(false));
                    }
                }
            }

            return Ok(JsValue::from(true));
        }

        Ok(JsValue::from(true))
    }

    /// HTMLFormElement.reportValidity() method
    fn report_validity(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // In a headless browser, reportValidity behaves the same as checkValidity
        // (no UI to show validation messages)
        Self::check_validity(this, args, context)
    }

    /// HTMLFormElement.requestSubmit() method
    fn request_submit(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let _this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLFormElement.requestSubmit called on non-object")
        })?;

        // requestSubmit validates before submitting (unlike submit())
        eprintln!("HTMLFormElement.requestSubmit() called - would validate and submit form");

        Ok(JsValue::undefined())
    }
}

// ============================================================================
// HTMLFormControlsCollection
// ============================================================================

/// HTMLFormControlsCollection implementation
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct HTMLFormControlsCollection {
    /// Reference to the parent form
    pub form: JsObject,
}

impl HTMLFormControlsCollection {
    pub fn new(form: JsObject) -> Self {
        Self { form }
    }
}

impl IntrinsicObject for HTMLFormControlsCollection {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(Self::item, js_string!("item"), 1)
            .static_method(Self::named_item, js_string!("namedItem"), 1)
            .property(js_string!("length"), 0, Attribute::CONFIGURABLE)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLFormControlsCollection {
    const NAME: JsString = StaticJsStrings::FORM_CONTROLS_COLLECTION;
}

impl BuiltInConstructor for HTMLFormControlsCollection {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_form_controls_collection;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("HTMLFormControlsCollection constructor cannot be called")
            .into())
    }
}

impl HTMLFormControlsCollection {
    fn item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("HTMLFormControlsCollection.item called on non-object")
        })?;

        if let Some(collection) = this_obj.downcast_ref::<HTMLFormControlsCollection>() {
            let index = args.get_or_undefined(0).to_number(context)? as usize;

            if let Some(form_data) = collection.form.downcast_ref::<HTMLFormElement>() {
                if let Some(element) = form_data.get_element_by_index(index) {
                    return Ok(element.into());
                }
            }
        }

        Ok(JsValue::null())
    }

    fn named_item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("HTMLFormControlsCollection.namedItem called on non-object")
        })?;

        if let Some(collection) = this_obj.downcast_ref::<HTMLFormControlsCollection>() {
            let name = args.get_or_undefined(0).to_string(context)?;
            let name_str = name.to_std_string_escaped();

            if let Some(form_data) = collection.form.downcast_ref::<HTMLFormElement>() {
                if let Some(element) = form_data.get_element_by_name(&name_str) {
                    return Ok(element.into());
                }
            }
        }

        Ok(JsValue::null())
    }
}

// ============================================================================
// HTMLInputElement - Complete input element implementation
// ============================================================================

/// HTMLInputElement implementation with full property support
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct HTMLInputElement {
    // Identity
    #[unsafe_ignore_trace]
    pub name: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub id: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub input_type: Arc<Mutex<String>>,

    // Value state
    #[unsafe_ignore_trace]
    pub value: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub default_value: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub checked: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    pub default_checked: Arc<Mutex<bool>>,

    // UI state
    #[unsafe_ignore_trace]
    pub disabled: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    pub readonly: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    pub required: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    pub autofocus: Arc<Mutex<bool>>,

    // Text properties
    #[unsafe_ignore_trace]
    pub placeholder: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub autocomplete: Arc<Mutex<String>>,

    // Validation
    #[unsafe_ignore_trace]
    pub pattern: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub min_length: Arc<Mutex<i32>>,
    #[unsafe_ignore_trace]
    pub max_length: Arc<Mutex<i32>>,
    #[unsafe_ignore_trace]
    pub min: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub max: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub step: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub custom_validity_message: Arc<Mutex<String>>,

    // File input
    #[unsafe_ignore_trace]
    pub accept: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub multiple: Arc<Mutex<bool>>,

    // Misc
    #[unsafe_ignore_trace]
    pub size: Arc<Mutex<u32>>,
    #[unsafe_ignore_trace]
    pub form_action: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub form_method: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub form_enctype: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub form_target: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub form_no_validate: Arc<Mutex<bool>>,
}

impl HTMLInputElement {
    pub fn new() -> Self {
        Self {
            name: Arc::new(Mutex::new(String::new())),
            id: Arc::new(Mutex::new(String::new())),
            input_type: Arc::new(Mutex::new("text".to_string())),
            value: Arc::new(Mutex::new(String::new())),
            default_value: Arc::new(Mutex::new(String::new())),
            checked: Arc::new(Mutex::new(false)),
            default_checked: Arc::new(Mutex::new(false)),
            disabled: Arc::new(Mutex::new(false)),
            readonly: Arc::new(Mutex::new(false)),
            required: Arc::new(Mutex::new(false)),
            autofocus: Arc::new(Mutex::new(false)),
            placeholder: Arc::new(Mutex::new(String::new())),
            autocomplete: Arc::new(Mutex::new(String::new())),
            pattern: Arc::new(Mutex::new(String::new())),
            min_length: Arc::new(Mutex::new(-1)),
            max_length: Arc::new(Mutex::new(-1)),
            min: Arc::new(Mutex::new(String::new())),
            max: Arc::new(Mutex::new(String::new())),
            step: Arc::new(Mutex::new(String::new())),
            custom_validity_message: Arc::new(Mutex::new(String::new())),
            accept: Arc::new(Mutex::new(String::new())),
            multiple: Arc::new(Mutex::new(false)),
            size: Arc::new(Mutex::new(20)),
            form_action: Arc::new(Mutex::new(String::new())),
            form_method: Arc::new(Mutex::new(String::new())),
            form_enctype: Arc::new(Mutex::new(String::new())),
            form_target: Arc::new(Mutex::new(String::new())),
            form_no_validate: Arc::new(Mutex::new(false)),
        }
    }

    /// Check if the input is valid based on constraints
    pub fn check_validity_state(&self) -> ValidityState {
        let mut validity = ValidityState::new();

        let value = self.value.lock().unwrap();
        let required = *self.required.lock().unwrap();
        let input_type = self.input_type.lock().unwrap();
        let pattern = self.pattern.lock().unwrap();
        let min_length = *self.min_length.lock().unwrap();
        let max_length = *self.max_length.lock().unwrap();

        // Check required
        if required && value.is_empty() {
            validity.value_missing = true;
        }

        // Check type
        match input_type.as_str() {
            "email" => {
                if !value.is_empty() && !value.contains('@') {
                    validity.type_mismatch = true;
                }
            }
            "url" => {
                if !value.is_empty()
                    && !value.starts_with("http://")
                    && !value.starts_with("https://")
                {
                    validity.type_mismatch = true;
                }
            }
            "number" => {
                if !value.is_empty() && value.parse::<f64>().is_err() {
                    validity.bad_input = true;
                }
            }
            _ => {}
        }

        // Check pattern
        if !pattern.is_empty() && !value.is_empty() {
            if let Ok(re) = regex::Regex::new(&pattern) {
                if !re.is_match(&value) {
                    validity.pattern_mismatch = true;
                }
            }
        }

        // Check length constraints
        if min_length >= 0 && (value.len() as i32) < min_length {
            validity.too_short = true;
        }
        if max_length >= 0 && (value.len() as i32) > max_length {
            validity.too_long = true;
        }

        // Check custom error
        if !self.custom_validity_message.lock().unwrap().is_empty() {
            validity.custom_error = true;
        }

        validity
    }
}

impl IntrinsicObject for HTMLInputElement {
    fn init(realm: &Realm) {
        // Create getters/setters for all properties
        let name_getter = BuiltInBuilder::callable(realm, input_name_getter)
            .name(js_string!("get name"))
            .build();
        let name_setter = BuiltInBuilder::callable(realm, input_name_setter)
            .name(js_string!("set name"))
            .build();
        let value_getter = BuiltInBuilder::callable(realm, input_value_getter)
            .name(js_string!("get value"))
            .build();
        let value_setter = BuiltInBuilder::callable(realm, input_value_setter)
            .name(js_string!("set value"))
            .build();
        let type_getter = BuiltInBuilder::callable(realm, input_type_getter)
            .name(js_string!("get type"))
            .build();
        let type_setter = BuiltInBuilder::callable(realm, input_type_setter)
            .name(js_string!("set type"))
            .build();
        let checked_getter = BuiltInBuilder::callable(realm, input_checked_getter)
            .name(js_string!("get checked"))
            .build();
        let checked_setter = BuiltInBuilder::callable(realm, input_checked_setter)
            .name(js_string!("set checked"))
            .build();
        let disabled_getter = BuiltInBuilder::callable(realm, input_disabled_getter)
            .name(js_string!("get disabled"))
            .build();
        let disabled_setter = BuiltInBuilder::callable(realm, input_disabled_setter)
            .name(js_string!("set disabled"))
            .build();
        let required_getter = BuiltInBuilder::callable(realm, input_required_getter)
            .name(js_string!("get required"))
            .build();
        let required_setter = BuiltInBuilder::callable(realm, input_required_setter)
            .name(js_string!("set required"))
            .build();
        let placeholder_getter = BuiltInBuilder::callable(realm, input_placeholder_getter)
            .name(js_string!("get placeholder"))
            .build();
        let placeholder_setter = BuiltInBuilder::callable(realm, input_placeholder_setter)
            .name(js_string!("set placeholder"))
            .build();
        let readonly_getter = BuiltInBuilder::callable(realm, input_readonly_getter)
            .name(js_string!("get readOnly"))
            .build();
        let readonly_setter = BuiltInBuilder::callable(realm, input_readonly_setter)
            .name(js_string!("set readOnly"))
            .build();
        let pattern_getter = BuiltInBuilder::callable(realm, input_pattern_getter)
            .name(js_string!("get pattern"))
            .build();
        let pattern_setter = BuiltInBuilder::callable(realm, input_pattern_setter)
            .name(js_string!("set pattern"))
            .build();
        let minlength_getter = BuiltInBuilder::callable(realm, input_minlength_getter)
            .name(js_string!("get minLength"))
            .build();
        let minlength_setter = BuiltInBuilder::callable(realm, input_minlength_setter)
            .name(js_string!("set minLength"))
            .build();
        let maxlength_getter = BuiltInBuilder::callable(realm, input_maxlength_getter)
            .name(js_string!("get maxLength"))
            .build();
        let maxlength_setter = BuiltInBuilder::callable(realm, input_maxlength_setter)
            .name(js_string!("set maxLength"))
            .build();
        let validation_message_getter =
            BuiltInBuilder::callable(realm, input_validation_message_getter)
                .name(js_string!("get validationMessage"))
                .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("name"),
                Some(name_getter),
                Some(name_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("value"),
                Some(value_getter),
                Some(value_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("type"),
                Some(type_getter),
                Some(type_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("checked"),
                Some(checked_getter),
                Some(checked_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("disabled"),
                Some(disabled_getter),
                Some(disabled_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("required"),
                Some(required_getter),
                Some(required_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("placeholder"),
                Some(placeholder_getter),
                Some(placeholder_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("readOnly"),
                Some(readonly_getter),
                Some(readonly_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("pattern"),
                Some(pattern_getter),
                Some(pattern_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("minLength"),
                Some(minlength_getter),
                Some(minlength_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("maxLength"),
                Some(maxlength_getter),
                Some(maxlength_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("validationMessage"),
                Some(validation_message_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(Self::check_validity, js_string!("checkValidity"), 0)
            .method(Self::report_validity, js_string!("reportValidity"), 0)
            .method(
                Self::set_custom_validity,
                js_string!("setCustomValidity"),
                1,
            )
            .method(Self::focus, js_string!("focus"), 0)
            .method(Self::blur, js_string!("blur"), 0)
            .method(Self::click, js_string!("click"), 0)
            .method(Self::select, js_string!("select"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLInputElement {
    const NAME: JsString = StaticJsStrings::INPUT_ELEMENT;
}

impl BuiltInConstructor for HTMLInputElement {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_input_element;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling HTMLInputElement constructor without `new` is forbidden")
                .into());
        }

        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::html_input_element,
            context,
        )?;

        let input = HTMLInputElement::new();
        let input_obj =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, input);

        // Create validity object
        let validity_data = ValidityState::new();
        let validity_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            validity_data,
        );

        let input_generic = input_obj.upcast();
        input_generic.define_property_or_throw(
            js_string!("validity"),
            PropertyDescriptorBuilder::new()
                .value(validity_obj)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
            context,
        )?;

        Ok(input_generic.into())
    }
}

// HTMLInputElement property getters/setters
fn input_name_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        return Ok(JsValue::from(js_string!(
            input.name.lock().unwrap().clone()
        )));
    }
    Ok(JsValue::from(js_string!("")))
}

fn input_name_setter(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        let value = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        *input.name.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn input_value_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        return Ok(JsValue::from(js_string!(
            input.value.lock().unwrap().clone()
        )));
    }
    Ok(JsValue::from(js_string!("")))
}

fn input_value_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        let value = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        *input.value.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn input_type_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        return Ok(JsValue::from(js_string!(
            input.input_type.lock().unwrap().clone()
        )));
    }
    Ok(JsValue::from(js_string!("text")))
}

fn input_type_setter(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        let value = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped()
            .to_lowercase();
        *input.input_type.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn input_checked_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        return Ok(JsValue::from(*input.checked.lock().unwrap()));
    }
    Ok(JsValue::from(false))
}

fn input_checked_setter(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        let value = args.get_or_undefined(0).to_boolean();
        *input.checked.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn input_disabled_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        return Ok(JsValue::from(*input.disabled.lock().unwrap()));
    }
    Ok(JsValue::from(false))
}

fn input_disabled_setter(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        let value = args.get_or_undefined(0).to_boolean();
        *input.disabled.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn input_required_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        return Ok(JsValue::from(*input.required.lock().unwrap()));
    }
    Ok(JsValue::from(false))
}

fn input_required_setter(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        let value = args.get_or_undefined(0).to_boolean();
        *input.required.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn input_placeholder_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        return Ok(JsValue::from(js_string!(
            input.placeholder.lock().unwrap().clone()
        )));
    }
    Ok(JsValue::from(js_string!("")))
}

fn input_placeholder_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        let value = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        *input.placeholder.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn input_readonly_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        return Ok(JsValue::from(*input.readonly.lock().unwrap()));
    }
    Ok(JsValue::from(false))
}

fn input_readonly_setter(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        let value = args.get_or_undefined(0).to_boolean();
        *input.readonly.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn input_pattern_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        return Ok(JsValue::from(js_string!(
            input.pattern.lock().unwrap().clone()
        )));
    }
    Ok(JsValue::from(js_string!("")))
}

fn input_pattern_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        let value = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        *input.pattern.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn input_minlength_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        return Ok(JsValue::from(*input.min_length.lock().unwrap()));
    }
    Ok(JsValue::from(-1))
}

fn input_minlength_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        let value = args.get_or_undefined(0).to_number(context)? as i32;
        *input.min_length.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn input_maxlength_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        return Ok(JsValue::from(*input.max_length.lock().unwrap()));
    }
    Ok(JsValue::from(-1))
}

fn input_maxlength_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        let value = args.get_or_undefined(0).to_number(context)? as i32;
        *input.max_length.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn input_validation_message_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
        let custom_msg = input.custom_validity_message.lock().unwrap().clone();
        if !custom_msg.is_empty() {
            return Ok(JsValue::from(js_string!(custom_msg)));
        }
        // If there's no custom message, check built-in validity and return appropriate message
        let validity = input.check_validity_state();
        if !validity.is_valid() {
            if validity.value_missing {
                return Ok(JsValue::from(js_string!("Please fill out this field.")));
            }
            if validity.type_mismatch {
                let input_type = input.input_type.lock().unwrap().clone();
                match input_type.as_str() {
                    "email" => return Ok(JsValue::from(js_string!("Please include an '@' in the email address."))),
                    "url" => return Ok(JsValue::from(js_string!("Please enter a URL."))),
                    _ => return Ok(JsValue::from(js_string!("Invalid value."))),
                }
            }
            if validity.pattern_mismatch {
                return Ok(JsValue::from(js_string!("Please match the requested format.")));
            }
            if validity.too_short {
                return Ok(JsValue::from(js_string!("Value is too short.")));
            }
            if validity.too_long {
                return Ok(JsValue::from(js_string!("Value is too long.")));
            }
            if validity.bad_input {
                return Ok(JsValue::from(js_string!("Please enter a valid value.")));
            }
            return Ok(JsValue::from(js_string!("Invalid value.")));
        }
    }
    Ok(JsValue::from(js_string!("")))
}

impl HTMLInputElement {
    /// checkValidity() method
    fn check_validity(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLInputElement.checkValidity called on non-object")
        })?;

        if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
            let validity = input.check_validity_state();
            return Ok(JsValue::from(validity.is_valid()));
        }

        Ok(JsValue::from(true))
    }

    /// reportValidity() method
    fn report_validity(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("HTMLInputElement.reportValidity called on non-object")
        })?;

        if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
            let validity = input.check_validity_state();
            // In a real browser, this would show validation UI
            return Ok(JsValue::from(validity.is_valid()));
        }

        Ok(JsValue::from(true))
    }

    /// setCustomValidity(message) method
    fn set_custom_validity(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("HTMLInputElement.setCustomValidity called on non-object")
        })?;

        if let Some(input) = obj.downcast_ref::<HTMLInputElement>() {
            let message = args
                .get_or_undefined(0)
                .to_string(context)?
                .to_std_string_escaped();
            *input.custom_validity_message.lock().unwrap() = message;
        }

        Ok(JsValue::undefined())
    }

    /// focus() method
    fn focus(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let _obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLInputElement.focus called on non-object")
        })?;
        // In a headless browser, focus is simulated
        Ok(JsValue::undefined())
    }

    /// blur() method
    fn blur(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let _obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLInputElement.blur called on non-object")
        })?;
        // In a headless browser, blur is simulated
        Ok(JsValue::undefined())
    }

    /// click() method
    fn click(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let _obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLInputElement.click called on non-object")
        })?;
        // Simulate click - would fire click event
        Ok(JsValue::undefined())
    }

    /// select() method - selects all text in input
    fn select(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let _obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLInputElement.select called on non-object")
        })?;
        // Would select text content
        Ok(JsValue::undefined())
    }
}

// ============================================================================
// HTMLSelectElement
// ============================================================================

/// HTMLSelectElement implementation
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct HTMLSelectElement {
    #[unsafe_ignore_trace]
    pub name: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub id: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub value: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub selected_index: Arc<Mutex<i32>>,
    #[unsafe_ignore_trace]
    pub multiple: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    pub disabled: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    pub required: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    pub size: Arc<Mutex<u32>>,
    #[unsafe_ignore_trace]
    pub options: Arc<Mutex<Vec<JsObject>>>,
    #[unsafe_ignore_trace]
    pub custom_validity_message: Arc<Mutex<String>>,
}

impl HTMLSelectElement {
    pub fn new() -> Self {
        Self {
            name: Arc::new(Mutex::new(String::new())),
            id: Arc::new(Mutex::new(String::new())),
            value: Arc::new(Mutex::new(String::new())),
            selected_index: Arc::new(Mutex::new(-1)),
            multiple: Arc::new(Mutex::new(false)),
            disabled: Arc::new(Mutex::new(false)),
            required: Arc::new(Mutex::new(false)),
            size: Arc::new(Mutex::new(1)),
            options: Arc::new(Mutex::new(Vec::new())),
            custom_validity_message: Arc::new(Mutex::new(String::new())),
        }
    }
}

impl IntrinsicObject for HTMLSelectElement {
    fn init(realm: &Realm) {
        let name_getter = BuiltInBuilder::callable(realm, select_name_getter)
            .name(js_string!("get name"))
            .build();
        let name_setter = BuiltInBuilder::callable(realm, select_name_setter)
            .name(js_string!("set name"))
            .build();
        let value_getter = BuiltInBuilder::callable(realm, select_value_getter)
            .name(js_string!("get value"))
            .build();
        let value_setter = BuiltInBuilder::callable(realm, select_value_setter)
            .name(js_string!("set value"))
            .build();
        let selected_index_getter = BuiltInBuilder::callable(realm, select_selected_index_getter)
            .name(js_string!("get selectedIndex"))
            .build();
        let selected_index_setter = BuiltInBuilder::callable(realm, select_selected_index_setter)
            .name(js_string!("set selectedIndex"))
            .build();
        let multiple_getter = BuiltInBuilder::callable(realm, select_multiple_getter)
            .name(js_string!("get multiple"))
            .build();
        let multiple_setter = BuiltInBuilder::callable(realm, select_multiple_setter)
            .name(js_string!("set multiple"))
            .build();
        let disabled_getter = BuiltInBuilder::callable(realm, select_disabled_getter)
            .name(js_string!("get disabled"))
            .build();
        let disabled_setter = BuiltInBuilder::callable(realm, select_disabled_setter)
            .name(js_string!("set disabled"))
            .build();
        let required_getter = BuiltInBuilder::callable(realm, select_required_getter)
            .name(js_string!("get required"))
            .build();
        let required_setter = BuiltInBuilder::callable(realm, select_required_setter)
            .name(js_string!("set required"))
            .build();
        let length_getter = BuiltInBuilder::callable(realm, select_length_getter)
            .name(js_string!("get length"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("name"),
                Some(name_getter),
                Some(name_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("value"),
                Some(value_getter),
                Some(value_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("selectedIndex"),
                Some(selected_index_getter),
                Some(selected_index_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("multiple"),
                Some(multiple_getter),
                Some(multiple_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("disabled"),
                Some(disabled_getter),
                Some(disabled_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("required"),
                Some(required_getter),
                Some(required_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("length"),
                Some(length_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(Self::check_validity, js_string!("checkValidity"), 0)
            .method(Self::report_validity, js_string!("reportValidity"), 0)
            .method(
                Self::set_custom_validity,
                js_string!("setCustomValidity"),
                1,
            )
            .method(Self::add, js_string!("add"), 1)
            .method(Self::remove, js_string!("remove"), 0)
            .method(Self::item, js_string!("item"), 1)
            .method(Self::named_item, js_string!("namedItem"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLSelectElement {
    const NAME: JsString = js_string!("HTMLSelectElement");
}

impl BuiltInConstructor for HTMLSelectElement {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 21; // Accessors and methods on prototype
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_select_element;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling HTMLSelectElement constructor without `new` is forbidden")
                .into());
        }

        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::html_select_element,
            context,
        )?;

        let select = HTMLSelectElement::new();
        let select_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            select,
        );

        Ok(select_obj.into())
    }
}

// HTMLSelectElement property getters/setters
fn select_name_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
        return Ok(JsValue::from(js_string!(
            select.name.lock().unwrap().clone()
        )));
    }
    Ok(JsValue::from(js_string!("")))
}

fn select_name_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
        let value = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        *select.name.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn select_value_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
        return Ok(JsValue::from(js_string!(
            select.value.lock().unwrap().clone()
        )));
    }
    Ok(JsValue::from(js_string!("")))
}

fn select_value_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
        let value = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        *select.value.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn select_selected_index_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
        return Ok(JsValue::from(*select.selected_index.lock().unwrap()));
    }
    Ok(JsValue::from(-1))
}

fn select_selected_index_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
        let value = args.get_or_undefined(0).to_number(context)? as i32;
        *select.selected_index.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn select_multiple_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
        return Ok(JsValue::from(*select.multiple.lock().unwrap()));
    }
    Ok(JsValue::from(false))
}

fn select_multiple_setter(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
        let value = args.get_or_undefined(0).to_boolean();
        *select.multiple.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn select_disabled_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
        return Ok(JsValue::from(*select.disabled.lock().unwrap()));
    }
    Ok(JsValue::from(false))
}

fn select_disabled_setter(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
        let value = args.get_or_undefined(0).to_boolean();
        *select.disabled.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn select_required_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
        return Ok(JsValue::from(*select.required.lock().unwrap()));
    }
    Ok(JsValue::from(false))
}

fn select_required_setter(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
        let value = args.get_or_undefined(0).to_boolean();
        *select.required.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn select_length_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
        return Ok(JsValue::from(select.options.lock().unwrap().len() as i32));
    }
    Ok(JsValue::from(0))
}

impl HTMLSelectElement {
    fn check_validity(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("HTMLSelectElement.checkValidity called on non-object")
        })?;

        if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
            let required = *select.required.lock().unwrap();
            let selected_index = *select.selected_index.lock().unwrap();

            if required && selected_index < 0 {
                return Ok(JsValue::from(false));
            }
        }

        Ok(JsValue::from(true))
    }

    fn report_validity(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Self::check_validity(this, args, context)
    }

    fn set_custom_validity(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("HTMLSelectElement.setCustomValidity called on non-object")
        })?;

        if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
            let message = args
                .get_or_undefined(0)
                .to_string(context)?
                .to_std_string_escaped();
            *select.custom_validity_message.lock().unwrap() = message;
        }

        Ok(JsValue::undefined())
    }

    fn add(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLSelectElement.add called on non-object")
        })?;

        if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
            let element = args.get_or_undefined(0);
            if let Some(option_obj) = element.as_object() {
                select.options.lock().unwrap().push(option_obj.clone());
            }
        }

        Ok(JsValue::undefined())
    }

    fn remove(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLSelectElement.remove called on non-object")
        })?;

        if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
            let index = args.get_or_undefined(0).to_number(context)? as usize;
            let mut options = select.options.lock().unwrap();
            if index < options.len() {
                options.remove(index);
            }
        }

        Ok(JsValue::undefined())
    }

    fn item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLSelectElement.item called on non-object")
        })?;

        if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
            let index = args.get_or_undefined(0).to_number(context)? as usize;
            let options = select.options.lock().unwrap();
            if let Some(option) = options.get(index) {
                return Ok(option.clone().into());
            }
        }

        Ok(JsValue::null())
    }

    fn named_item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLSelectElement.namedItem called on non-object")
        })?;

        if let Some(select) = obj.downcast_ref::<HTMLSelectElement>() {
            let name = args
                .get_or_undefined(0)
                .to_string(context)?
                .to_std_string_escaped();
            let options = select.options.lock().unwrap();
            for option in options.iter() {
                if let Ok(option_name) = option.get(js_string!("name"), context) {
                    if let Some(name_str) = option_name.as_string() {
                        if name_str.to_std_string_escaped() == name {
                            return Ok(option.clone().into());
                        }
                    }
                }
            }
        }

        Ok(JsValue::null())
    }
}

// ============================================================================
// HTMLTextAreaElement
// ============================================================================

/// HTMLTextAreaElement implementation
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct HTMLTextAreaElement {
    #[unsafe_ignore_trace]
    pub name: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub id: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub value: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub default_value: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub placeholder: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub rows: Arc<Mutex<u32>>,
    #[unsafe_ignore_trace]
    pub cols: Arc<Mutex<u32>>,
    #[unsafe_ignore_trace]
    pub min_length: Arc<Mutex<i32>>,
    #[unsafe_ignore_trace]
    pub max_length: Arc<Mutex<i32>>,
    #[unsafe_ignore_trace]
    pub disabled: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    pub readonly: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    pub required: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    pub wrap: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub custom_validity_message: Arc<Mutex<String>>,
}

impl HTMLTextAreaElement {
    pub fn new() -> Self {
        Self {
            name: Arc::new(Mutex::new(String::new())),
            id: Arc::new(Mutex::new(String::new())),
            value: Arc::new(Mutex::new(String::new())),
            default_value: Arc::new(Mutex::new(String::new())),
            placeholder: Arc::new(Mutex::new(String::new())),
            rows: Arc::new(Mutex::new(2)),
            cols: Arc::new(Mutex::new(20)),
            min_length: Arc::new(Mutex::new(-1)),
            max_length: Arc::new(Mutex::new(-1)),
            disabled: Arc::new(Mutex::new(false)),
            readonly: Arc::new(Mutex::new(false)),
            required: Arc::new(Mutex::new(false)),
            wrap: Arc::new(Mutex::new("soft".to_string())),
            custom_validity_message: Arc::new(Mutex::new(String::new())),
        }
    }
}

impl IntrinsicObject for HTMLTextAreaElement {
    fn init(realm: &Realm) {
        let name_getter = BuiltInBuilder::callable(realm, textarea_name_getter)
            .name(js_string!("get name"))
            .build();
        let name_setter = BuiltInBuilder::callable(realm, textarea_name_setter)
            .name(js_string!("set name"))
            .build();
        let value_getter = BuiltInBuilder::callable(realm, textarea_value_getter)
            .name(js_string!("get value"))
            .build();
        let value_setter = BuiltInBuilder::callable(realm, textarea_value_setter)
            .name(js_string!("set value"))
            .build();
        let placeholder_getter = BuiltInBuilder::callable(realm, textarea_placeholder_getter)
            .name(js_string!("get placeholder"))
            .build();
        let placeholder_setter = BuiltInBuilder::callable(realm, textarea_placeholder_setter)
            .name(js_string!("set placeholder"))
            .build();
        let rows_getter = BuiltInBuilder::callable(realm, textarea_rows_getter)
            .name(js_string!("get rows"))
            .build();
        let rows_setter = BuiltInBuilder::callable(realm, textarea_rows_setter)
            .name(js_string!("set rows"))
            .build();
        let cols_getter = BuiltInBuilder::callable(realm, textarea_cols_getter)
            .name(js_string!("get cols"))
            .build();
        let cols_setter = BuiltInBuilder::callable(realm, textarea_cols_setter)
            .name(js_string!("set cols"))
            .build();
        let disabled_getter = BuiltInBuilder::callable(realm, textarea_disabled_getter)
            .name(js_string!("get disabled"))
            .build();
        let disabled_setter = BuiltInBuilder::callable(realm, textarea_disabled_setter)
            .name(js_string!("set disabled"))
            .build();
        let required_getter = BuiltInBuilder::callable(realm, textarea_required_getter)
            .name(js_string!("get required"))
            .build();
        let required_setter = BuiltInBuilder::callable(realm, textarea_required_setter)
            .name(js_string!("set required"))
            .build();
        let readonly_getter = BuiltInBuilder::callable(realm, textarea_readonly_getter)
            .name(js_string!("get readOnly"))
            .build();
        let readonly_setter = BuiltInBuilder::callable(realm, textarea_readonly_setter)
            .name(js_string!("set readOnly"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("name"),
                Some(name_getter),
                Some(name_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("value"),
                Some(value_getter),
                Some(value_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("placeholder"),
                Some(placeholder_getter),
                Some(placeholder_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("rows"),
                Some(rows_getter),
                Some(rows_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("cols"),
                Some(cols_getter),
                Some(cols_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("disabled"),
                Some(disabled_getter),
                Some(disabled_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("required"),
                Some(required_getter),
                Some(required_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("readOnly"),
                Some(readonly_getter),
                Some(readonly_setter),
                Attribute::CONFIGURABLE,
            )
            .method(Self::check_validity, js_string!("checkValidity"), 0)
            .method(Self::report_validity, js_string!("reportValidity"), 0)
            .method(
                Self::set_custom_validity,
                js_string!("setCustomValidity"),
                1,
            )
            .method(Self::select, js_string!("select"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLTextAreaElement {
    const NAME: JsString = js_string!("HTMLTextAreaElement");
}

impl BuiltInConstructor for HTMLTextAreaElement {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 20; // Accessors and methods on prototype
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_textarea_element;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling HTMLTextAreaElement constructor without `new` is forbidden")
                .into());
        }

        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::html_textarea_element,
            context,
        )?;

        let textarea = HTMLTextAreaElement::new();
        let textarea_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            textarea,
        );

        Ok(textarea_obj.into())
    }
}

// HTMLTextAreaElement property getters/setters
fn textarea_name_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
        return Ok(JsValue::from(js_string!(
            textarea.name.lock().unwrap().clone()
        )));
    }
    Ok(JsValue::from(js_string!("")))
}

fn textarea_name_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
        let value = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        *textarea.name.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn textarea_value_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
        return Ok(JsValue::from(js_string!(
            textarea.value.lock().unwrap().clone()
        )));
    }
    Ok(JsValue::from(js_string!("")))
}

fn textarea_value_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
        let value = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        *textarea.value.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn textarea_placeholder_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
        return Ok(JsValue::from(js_string!(
            textarea.placeholder.lock().unwrap().clone()
        )));
    }
    Ok(JsValue::from(js_string!("")))
}

fn textarea_placeholder_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
        let value = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        *textarea.placeholder.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn textarea_rows_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
        return Ok(JsValue::from(*textarea.rows.lock().unwrap()));
    }
    Ok(JsValue::from(2))
}

fn textarea_rows_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
        let value = args.get_or_undefined(0).to_number(context)? as u32;
        *textarea.rows.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn textarea_cols_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
        return Ok(JsValue::from(*textarea.cols.lock().unwrap()));
    }
    Ok(JsValue::from(20))
}

fn textarea_cols_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
        let value = args.get_or_undefined(0).to_number(context)? as u32;
        *textarea.cols.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn textarea_disabled_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
        return Ok(JsValue::from(*textarea.disabled.lock().unwrap()));
    }
    Ok(JsValue::from(false))
}

fn textarea_disabled_setter(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
        let value = args.get_or_undefined(0).to_boolean();
        *textarea.disabled.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn textarea_required_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
        return Ok(JsValue::from(*textarea.required.lock().unwrap()));
    }
    Ok(JsValue::from(false))
}

fn textarea_required_setter(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
        let value = args.get_or_undefined(0).to_boolean();
        *textarea.required.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn textarea_readonly_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
        return Ok(JsValue::from(*textarea.readonly.lock().unwrap()));
    }
    Ok(JsValue::from(false))
}

fn textarea_readonly_setter(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
        let value = args.get_or_undefined(0).to_boolean();
        *textarea.readonly.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

impl HTMLTextAreaElement {
    fn check_validity(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("HTMLTextAreaElement.checkValidity called on non-object")
        })?;

        if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
            let value = textarea.value.lock().unwrap();
            let required = *textarea.required.lock().unwrap();
            let min_length = *textarea.min_length.lock().unwrap();
            let max_length = *textarea.max_length.lock().unwrap();

            if required && value.is_empty() {
                return Ok(JsValue::from(false));
            }

            if min_length >= 0 && (value.len() as i32) < min_length {
                return Ok(JsValue::from(false));
            }

            if max_length >= 0 && (value.len() as i32) > max_length {
                return Ok(JsValue::from(false));
            }

            if !textarea.custom_validity_message.lock().unwrap().is_empty() {
                return Ok(JsValue::from(false));
            }
        }

        Ok(JsValue::from(true))
    }

    fn report_validity(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Self::check_validity(this, args, context)
    }

    fn set_custom_validity(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("HTMLTextAreaElement.setCustomValidity called on non-object")
        })?;

        if let Some(textarea) = obj.downcast_ref::<HTMLTextAreaElement>() {
            let message = args
                .get_or_undefined(0)
                .to_string(context)?
                .to_std_string_escaped();
            *textarea.custom_validity_message.lock().unwrap() = message;
        }

        Ok(JsValue::undefined())
    }

    fn select(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let _obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLTextAreaElement.select called on non-object")
        })?;
        // Would select all text in the textarea
        Ok(JsValue::undefined())
    }
}

// ============================================================================
// HTMLOptionElement
// ============================================================================

/// HTMLOptionElement implementation
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct HTMLOptionElement {
    #[unsafe_ignore_trace]
    pub value: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub text: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub label: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub selected: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    pub default_selected: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    pub disabled: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    pub index: Arc<Mutex<i32>>,
}

impl HTMLOptionElement {
    pub fn new() -> Self {
        Self {
            value: Arc::new(Mutex::new(String::new())),
            text: Arc::new(Mutex::new(String::new())),
            label: Arc::new(Mutex::new(String::new())),
            selected: Arc::new(Mutex::new(false)),
            default_selected: Arc::new(Mutex::new(false)),
            disabled: Arc::new(Mutex::new(false)),
            index: Arc::new(Mutex::new(-1)),
        }
    }
}

impl IntrinsicObject for HTMLOptionElement {
    fn init(realm: &Realm) {
        let value_getter = BuiltInBuilder::callable(realm, option_value_getter)
            .name(js_string!("get value"))
            .build();
        let value_setter = BuiltInBuilder::callable(realm, option_value_setter)
            .name(js_string!("set value"))
            .build();
        let text_getter = BuiltInBuilder::callable(realm, option_text_getter)
            .name(js_string!("get text"))
            .build();
        let text_setter = BuiltInBuilder::callable(realm, option_text_setter)
            .name(js_string!("set text"))
            .build();
        let selected_getter = BuiltInBuilder::callable(realm, option_selected_getter)
            .name(js_string!("get selected"))
            .build();
        let selected_setter = BuiltInBuilder::callable(realm, option_selected_setter)
            .name(js_string!("set selected"))
            .build();
        let disabled_getter = BuiltInBuilder::callable(realm, option_disabled_getter)
            .name(js_string!("get disabled"))
            .build();
        let disabled_setter = BuiltInBuilder::callable(realm, option_disabled_setter)
            .name(js_string!("set disabled"))
            .build();
        let index_getter = BuiltInBuilder::callable(realm, option_index_getter)
            .name(js_string!("get index"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("value"),
                Some(value_getter),
                Some(value_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("text"),
                Some(text_getter),
                Some(text_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("selected"),
                Some(selected_getter),
                Some(selected_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("disabled"),
                Some(disabled_getter),
                Some(disabled_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("index"),
                Some(index_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLOptionElement {
    const NAME: JsString = js_string!("HTMLOptionElement");
}

impl BuiltInConstructor for HTMLOptionElement {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 10; // Accessors on prototype
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_option_element;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling HTMLOptionElement constructor without `new` is forbidden")
                .into());
        }

        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::html_option_element,
            context,
        )?;

        let option = HTMLOptionElement::new();

        // Optional constructor arguments: text, value, defaultSelected, selected
        if let Some(text_arg) = args.get(0) {
            if !text_arg.is_undefined() {
                let text = text_arg.to_string(context)?.to_std_string_escaped();
                *option.text.lock().unwrap() = text;
            }
        }

        if let Some(value_arg) = args.get(1) {
            if !value_arg.is_undefined() {
                let value = value_arg.to_string(context)?.to_std_string_escaped();
                *option.value.lock().unwrap() = value;
            }
        }

        if let Some(default_selected_arg) = args.get(2) {
            *option.default_selected.lock().unwrap() = default_selected_arg.to_boolean();
        }

        if let Some(selected_arg) = args.get(3) {
            *option.selected.lock().unwrap() = selected_arg.to_boolean();
        }

        let option_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            option,
        );

        Ok(option_obj.into())
    }
}

// HTMLOptionElement property getters/setters
fn option_value_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(option) = obj.downcast_ref::<HTMLOptionElement>() {
        let value = option.value.lock().unwrap();
        if value.is_empty() {
            // If value is empty, return text
            return Ok(JsValue::from(js_string!(
                option.text.lock().unwrap().clone()
            )));
        }
        return Ok(JsValue::from(js_string!(value.clone())));
    }
    Ok(JsValue::from(js_string!("")))
}

fn option_value_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(option) = obj.downcast_ref::<HTMLOptionElement>() {
        let value = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        *option.value.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn option_text_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(option) = obj.downcast_ref::<HTMLOptionElement>() {
        return Ok(JsValue::from(js_string!(
            option.text.lock().unwrap().clone()
        )));
    }
    Ok(JsValue::from(js_string!("")))
}

fn option_text_setter(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(option) = obj.downcast_ref::<HTMLOptionElement>() {
        let value = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        *option.text.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn option_selected_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(option) = obj.downcast_ref::<HTMLOptionElement>() {
        return Ok(JsValue::from(*option.selected.lock().unwrap()));
    }
    Ok(JsValue::from(false))
}

fn option_selected_setter(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(option) = obj.downcast_ref::<HTMLOptionElement>() {
        let value = args.get_or_undefined(0).to_boolean();
        *option.selected.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn option_disabled_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(option) = obj.downcast_ref::<HTMLOptionElement>() {
        return Ok(JsValue::from(*option.disabled.lock().unwrap()));
    }
    Ok(JsValue::from(false))
}

fn option_disabled_setter(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(option) = obj.downcast_ref::<HTMLOptionElement>() {
        let value = args.get_or_undefined(0).to_boolean();
        *option.disabled.lock().unwrap() = value;
    }
    Ok(JsValue::undefined())
}

fn option_index_getter(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("invalid this"))?;
    if let Some(option) = obj.downcast_ref::<HTMLOptionElement>() {
        return Ok(JsValue::from(*option.index.lock().unwrap()));
    }
    Ok(JsValue::from(-1))
}
