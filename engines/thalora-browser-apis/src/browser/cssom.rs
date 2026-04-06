//! CSS Object Model (CSSOM) APIs for Boa
//!
//! Implements CSSStyleDeclaration and getComputedStyle

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, NativeFunction,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{FunctionObjectBuilder, JsObject, internal_methods::get_prototype_from_constructor},
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;

/// JavaScript `CSSStyleDeclaration` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct CSSStyleDeclaration;

impl IntrinsicObject for CSSStyleDeclaration {
    fn init(realm: &Realm) {
        let length_getter = BuiltInBuilder::callable(realm, get_length)
            .name(js_string!("get length"))
            .build();

        let css_text_getter = BuiltInBuilder::callable(realm, get_css_text)
            .name(js_string!("get cssText"))
            .build();
        let css_text_setter = BuiltInBuilder::callable(realm, set_css_text)
            .name(js_string!("set cssText"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("length"),
                Some(length_getter),
                None,
                boa_engine::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("cssText"),
                Some(css_text_getter),
                Some(css_text_setter),
                boa_engine::property::Attribute::CONFIGURABLE,
            )
            .method(get_property_value, js_string!("getPropertyValue"), 1)
            .method(set_property, js_string!("setProperty"), 2)
            .method(remove_property, js_string!("removeProperty"), 1)
            .method(get_property_priority, js_string!("getPropertyPriority"), 1)
            .method(item, js_string!("item"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for CSSStyleDeclaration {
    const NAME: JsString = StaticJsStrings::CSS_STYLE_DECLARATION;
}

impl BuiltInConstructor for CSSStyleDeclaration {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::css_style_declaration;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("CSSStyleDeclaration constructor requires 'new'")
                .into());
        }

        let proto = get_prototype_from_constructor(
            new_target,
            StandardConstructors::css_style_declaration,
            context,
        )?;
        let style_data = CSSStyleDeclarationData::new();
        let style_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            style_data,
        );

        Ok(style_obj.upcast().into())
    }
}

/// Internal data for CSSStyleDeclaration instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct CSSStyleDeclarationData {
    #[unsafe_ignore_trace]
    properties: HashMap<String, (String, bool)>, // (value, important)
}

impl Default for CSSStyleDeclarationData {
    fn default() -> Self {
        Self::new()
    }
}

impl CSSStyleDeclarationData {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }
}

fn get_length(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("CSSStyleDeclaration.prototype.length called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<CSSStyleDeclarationData>() {
        return Ok(JsValue::from(data.properties.len() as u32));
    }

    Ok(JsValue::from(0))
}

fn get_css_text(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("CSSStyleDeclaration.prototype.cssText called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<CSSStyleDeclarationData>() {
        let css_text: Vec<String> = data
            .properties
            .iter()
            .map(|(name, (value, important))| {
                if *important {
                    format!("{}: {} !important", name, value)
                } else {
                    format!("{}: {}", name, value)
                }
            })
            .collect();
        return Ok(js_string!(css_text.join("; ")).into());
    }

    Ok(js_string!("").into())
}

fn set_css_text(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("CSSStyleDeclaration.prototype.cssText called on non-object")
    })?;

    let css_text = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    if let Some(mut data) = this_obj.downcast_mut::<CSSStyleDeclarationData>() {
        data.properties.clear();
        // Parse simple CSS text (property: value; ...)
        for declaration in css_text.split(';') {
            let parts: Vec<&str> = declaration.splitn(2, ':').collect();
            if parts.len() == 2 {
                let name = parts[0].trim().to_string();
                let mut value = parts[1].trim().to_string();
                let important = value.ends_with("!important");
                if important {
                    value = value.replace("!important", "").trim().to_string();
                }
                data.properties.insert(name, (value, important));
            }
        }
    }

    Ok(JsValue::undefined())
}

fn get_property_value(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("CSSStyleDeclaration.prototype.getPropertyValue called on non-object")
    })?;

    let property = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<CSSStyleDeclarationData>()
        && let Some((value, _)) = data.properties.get(&property)
    {
        return Ok(js_string!(value.clone()).into());
    }

    Ok(js_string!("").into())
}

fn set_property(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("CSSStyleDeclaration.prototype.setProperty called on non-object")
    })?;

    let property = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    let value = args
        .get_or_undefined(1)
        .to_string(context)?
        .to_std_string_escaped();
    let priority = args
        .get(2)
        .and_then(|p| p.to_string(context).ok())
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_default();

    let important = priority.eq_ignore_ascii_case("important");

    if let Some(mut data) = this_obj.downcast_mut::<CSSStyleDeclarationData>() {
        if value.is_empty() {
            data.properties.remove(&property);
        } else {
            data.properties.insert(property, (value, important));
        }
    }

    Ok(JsValue::undefined())
}

fn remove_property(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("CSSStyleDeclaration.prototype.removeProperty called on non-object")
    })?;

    let property = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    if let Some(mut data) = this_obj.downcast_mut::<CSSStyleDeclarationData>()
        && let Some((old_value, _)) = data.properties.remove(&property)
    {
        return Ok(js_string!(old_value).into());
    }

    Ok(js_string!("").into())
}

fn get_property_priority(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("CSSStyleDeclaration.prototype.getPropertyPriority called on non-object")
    })?;

    let property = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<CSSStyleDeclarationData>()
        && let Some((_, important)) = data.properties.get(&property)
        && *important
    {
        return Ok(js_string!("important").into());
    }

    Ok(js_string!("").into())
}

fn item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("CSSStyleDeclaration.prototype.item called on non-object")
    })?;

    let index = args.get_or_undefined(0).to_u32(context)? as usize;

    if let Some(data) = this_obj.downcast_ref::<CSSStyleDeclarationData>()
        && let Some(name) = data.properties.keys().nth(index)
    {
        return Ok(js_string!(name.clone()).into());
    }

    Ok(js_string!("").into())
}

// =============================================================================
// CSSStyleSheet
// =============================================================================

/// Internal data for CSSStyleSheet instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct CSSStyleSheetData {
    #[unsafe_ignore_trace]
    rules: Vec<CSSRuleEntry>,
    #[unsafe_ignore_trace]
    disabled: bool,
    #[unsafe_ignore_trace]
    media: String,
}

/// A single CSS rule entry
#[derive(Debug, Clone)]
struct CSSRuleEntry {
    css_text: String,
}

impl CSSStyleSheetData {
    fn new() -> Self {
        Self {
            rules: Vec::new(),
            disabled: false,
            media: String::new(),
        }
    }
}

/// Create the CSSStyleSheet constructor and register it globally
pub fn register_css_style_sheet(context: &mut Context) -> JsResult<()> {
    let constructor = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(css_style_sheet_constructor),
    )
    .name(js_string!("CSSStyleSheet"))
    .length(0)
    .build();

    // Add prototype methods
    let prototype = JsObject::with_object_proto(context.intrinsics());

    let insert_rule_fn = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(css_style_sheet_insert_rule),
    )
    .name(js_string!("insertRule"))
    .length(1)
    .build();
    prototype.set(js_string!("insertRule"), insert_rule_fn, false, context)?;

    let delete_rule_fn = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(css_style_sheet_delete_rule),
    )
    .name(js_string!("deleteRule"))
    .length(1)
    .build();
    prototype.set(js_string!("deleteRule"), delete_rule_fn, false, context)?;

    let replace_fn = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(css_style_sheet_replace),
    )
    .name(js_string!("replace"))
    .length(1)
    .build();
    prototype.set(js_string!("replace"), replace_fn, false, context)?;

    let replace_sync_fn = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(css_style_sheet_replace_sync),
    )
    .name(js_string!("replaceSync"))
    .length(1)
    .build();
    prototype.set(js_string!("replaceSync"), replace_sync_fn, false, context)?;

    constructor.set(js_string!("prototype"), prototype, false, context)?;

    let global = context.global_object();
    global.set(js_string!("CSSStyleSheet"), constructor, false, context)?;

    Ok(())
}

fn css_style_sheet_constructor(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let mut sheet_data = CSSStyleSheetData::new();

    // Parse optional init dictionary: { media, disabled }
    if let Some(options) = args.first().and_then(|a| a.as_object()) {
        if let Ok(media_val) = options.get(js_string!("media"), context)
            && !media_val.is_undefined()
        {
            sheet_data.media = media_val.to_string(context)?.to_std_string_escaped();
        }
        if let Ok(disabled_val) = options.get(js_string!("disabled"), context) {
            sheet_data.disabled = disabled_val.to_boolean();
        }
    }

    let typed_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        context.intrinsics().constructors().object().prototype(),
        sheet_data,
    );
    let obj = typed_obj.upcast();

    // Set cssRules as a getter-like array (initially empty)
    let css_rules = boa_engine::builtins::array::Array::array_create(0, None, context)?;
    obj.set(js_string!("cssRules"), css_rules, false, context)?;

    // Set type property
    obj.set(js_string!("type"), js_string!("text/css"), false, context)?;

    // Set disabled property
    obj.set(js_string!("disabled"), false, false, context)?;

    // Copy prototype methods to instance
    let global = context.global_object();
    if let Ok(ctor) = global.get(js_string!("CSSStyleSheet"), context)
        && let Some(ctor_obj) = ctor.as_object()
        && let Ok(proto) = ctor_obj.get(js_string!("prototype"), context)
        && let Some(proto_obj) = proto.as_object()
    {
        for method_name in ["insertRule", "deleteRule", "replace", "replaceSync"] {
            if let Ok(method) = proto_obj.get(js_string!(method_name), context) {
                obj.set(js_string!(method_name), method, false, context)?;
            }
        }
    }

    Ok(obj.into())
}

/// `CSSStyleSheet.prototype.insertRule(rule, index)`
fn css_style_sheet_insert_rule(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("insertRule called on non-object"))?;

    let rule_text = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    let index = if args.len() > 1 {
        args[1].to_u32(context)? as usize
    } else {
        // Default: insert at end
        if let Some(data) = this_obj.downcast_ref::<CSSStyleSheetData>() {
            data.rules.len()
        } else {
            0
        }
    };

    if let Some(mut data) = this_obj.downcast_mut::<CSSStyleSheetData>() {
        if index > data.rules.len() {
            return Err(JsNativeError::range()
                .with_message("insertRule index out of bounds")
                .into());
        }
        data.rules.insert(
            index,
            CSSRuleEntry {
                css_text: rule_text.clone(),
            },
        );
    }

    // Update cssRules array
    rebuild_css_rules_array(&this_obj, context)?;

    Ok(JsValue::from(index as u32))
}

/// `CSSStyleSheet.prototype.deleteRule(index)`
fn css_style_sheet_delete_rule(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("deleteRule called on non-object"))?;

    let index = args.get_or_undefined(0).to_u32(context)? as usize;

    if let Some(mut data) = this_obj.downcast_mut::<CSSStyleSheetData>() {
        if index >= data.rules.len() {
            return Err(JsNativeError::range()
                .with_message("deleteRule index out of bounds")
                .into());
        }
        data.rules.remove(index);
    }

    rebuild_css_rules_array(&this_obj, context)?;

    Ok(JsValue::undefined())
}

/// `CSSStyleSheet.prototype.replace(text)` - async replacement (returns Promise)
fn css_style_sheet_replace(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    // replace() is async per spec but we implement it synchronously
    // and return a resolved Promise
    css_style_sheet_replace_sync(this, args, context)?;

    // Return a resolved Promise with the stylesheet
    let promise = boa_engine::builtins::promise::Promise::promise_resolve(
        &context.intrinsics().constructors().promise().constructor(),
        this.clone(),
        context,
    )?;
    Ok(promise.into())
}

/// `CSSStyleSheet.prototype.replaceSync(text)` - synchronous replacement
fn css_style_sheet_replace_sync(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("replaceSync called on non-object"))?;

    let css_text = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    if let Some(mut data) = this_obj.downcast_mut::<CSSStyleSheetData>() {
        data.rules.clear();
        // Simple rule splitting by closing brace
        for rule in css_text.split('}') {
            let trimmed = rule.trim();
            if !trimmed.is_empty() {
                data.rules.push(CSSRuleEntry {
                    css_text: format!("{}}}", trimmed),
                });
            }
        }
    }

    rebuild_css_rules_array(&this_obj, context)?;

    Ok(JsValue::undefined())
}

/// Rebuild the cssRules array property from internal rule data
fn rebuild_css_rules_array(sheet_obj: &JsObject, context: &mut Context) -> JsResult<()> {
    let rules = if let Some(data) = sheet_obj.downcast_ref::<CSSStyleSheetData>() {
        data.rules.clone()
    } else {
        return Ok(());
    };

    let css_rules =
        boa_engine::builtins::array::Array::array_create(rules.len() as u64, None, context)?;

    for (i, rule) in rules.iter().enumerate() {
        let rule_obj = JsObject::with_object_proto(context.intrinsics());
        rule_obj.set(
            js_string!("cssText"),
            js_string!(rule.css_text.clone()),
            false,
            context,
        )?;
        rule_obj.set(js_string!("type"), 1, false, context)?; // CSSRule.STYLE_RULE
        rule_obj.set(
            js_string!("parentStyleSheet"),
            sheet_obj.clone(),
            false,
            context,
        )?;
        css_rules.set(i as u32, rule_obj, false, context)?;
    }

    // Add length property
    css_rules.set(js_string!("length"), rules.len() as u32, false, context)?;

    sheet_obj.set(js_string!("cssRules"), css_rules, false, context)?;

    Ok(())
}

/// Create the getComputedStyle global function
pub fn create_get_computed_style_function(context: &mut Context) -> JsResult<JsValue> {
    let func = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(get_computed_style),
    )
    .name(js_string!("getComputedStyle"))
    .length(1)
    .build();

    Ok(func.into())
}

/// `getComputedStyle(element, pseudoElt)`
fn get_computed_style(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    // Return a new CSSStyleDeclaration
    let style_constructor = context
        .intrinsics()
        .constructors()
        .css_style_declaration()
        .constructor();
    CSSStyleDeclaration::constructor(&style_constructor.clone().into(), &[], context)
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
    fn test_css_style_declaration_exists() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                "typeof CSSStyleDeclaration === 'function'",
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_get_computed_style_exists() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes("typeof getComputedStyle === 'function'"))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_css_style_declaration_set_property() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            const style = new CSSStyleDeclaration();
            style.setProperty('color', 'red');
            style.getPropertyValue('color') === 'red';
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
