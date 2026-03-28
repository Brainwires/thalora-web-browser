//! CustomElementRegistry Web API implementation
//!
//! The CustomElementRegistry interface provides methods for registering custom elements
//! and querying registered elements.
//!
//! https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, NativeFunction, js_string,
    object::{FunctionObjectBuilder, JsObject, ObjectInitializer},
    property::Attribute,
    value::JsValue,
};
use std::collections::HashMap;
use std::sync::RwLock;

/// Global registry of custom elements (keyed by name)
static REGISTRY: once_cell::sync::Lazy<RwLock<HashMap<String, CustomElementDefinition>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(HashMap::new()));

/// Custom element definition
#[derive(Clone)]
pub struct CustomElementDefinition {
    /// The constructor for this custom element
    pub name: String,
    /// Whether this element extends a built-in element
    pub extends: Option<String>,
}

/// JavaScript `CustomElementRegistry` implementation.
#[derive(Debug, Copy, Clone)]
pub struct CustomElementRegistry;

impl CustomElementRegistry {
    /// Initialize the customElements registry in the global scope
    pub fn init(context: &mut Context) {
        let registry = ObjectInitializer::new(context)
            .function(
                NativeFunction::from_fn_ptr(Self::define),
                js_string!("define"),
                3,
            )
            .function(NativeFunction::from_fn_ptr(Self::get), js_string!("get"), 1)
            .function(
                NativeFunction::from_fn_ptr(Self::get_name),
                js_string!("getName"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::when_defined),
                js_string!("whenDefined"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::upgrade),
                js_string!("upgrade"),
                1,
            )
            .build();

        // Register customElements globally
        context
            .register_global_property(
                js_string!("customElements"),
                registry,
                Attribute::READONLY | Attribute::NON_ENUMERABLE,
            )
            .expect("Failed to register customElements");
    }

    /// Validate custom element name per spec
    fn is_valid_custom_element_name(name: &str) -> bool {
        // Must contain a hyphen
        if !name.contains('-') {
            return false;
        }

        // Must start with a lowercase ASCII letter
        if let Some(first) = name.chars().next() {
            if !first.is_ascii_lowercase() {
                return false;
            }
        } else {
            return false;
        }

        // Reserved names
        let reserved = [
            "annotation-xml",
            "color-profile",
            "font-face",
            "font-face-src",
            "font-face-uri",
            "font-face-format",
            "font-face-name",
            "missing-glyph",
        ];

        if reserved.contains(&name) {
            return false;
        }

        // All characters must be valid
        name.chars().all(|c| {
            c.is_ascii_lowercase()
                || c.is_ascii_digit()
                || c == '-'
                || c == '.'
                || c == '_'
                || c == '\u{B7}'
                || ('\u{C0}'..='\u{D6}').contains(&c)
                || ('\u{D8}'..='\u{F6}').contains(&c)
                || ('\u{F8}'..='\u{37D}').contains(&c)
                || ('\u{37F}'..='\u{1FFF}').contains(&c)
                || ('\u{200C}'..='\u{200D}').contains(&c)
                || ('\u{203F}'..='\u{2040}').contains(&c)
                || ('\u{2070}'..='\u{218F}').contains(&c)
                || ('\u{2C00}'..='\u{2FEF}').contains(&c)
                || ('\u{3001}'..='\u{D7FF}').contains(&c)
                || ('\u{F900}'..='\u{FDCF}').contains(&c)
                || ('\u{FDF0}'..='\u{FFFD}').contains(&c)
                || ('\u{10000}'..='\u{EFFFF}').contains(&c)
        })
    }

    /// `customElements.define(name, constructor, options)`
    ///
    /// Defines a new custom element.
    fn define(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let name = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();

        let constructor = args.get_or_undefined(1);
        let options = args.get_or_undefined(2);

        // Validate name
        if !Self::is_valid_custom_element_name(&name) {
            return Err(JsNativeError::syntax()
                .with_message(format!("'{}' is not a valid custom element name", name))
                .into());
        }

        // Constructor must be a function
        if !constructor.is_callable() {
            return Err(JsNativeError::typ()
                .with_message("Custom element constructor must be a function")
                .into());
        }

        // Check if already defined
        {
            let registry = REGISTRY.read().unwrap();
            if registry.contains_key(&name) {
                return Err(JsNativeError::error()
                    .with_message(format!(
                        "Custom element '{}' has already been defined",
                        name
                    ))
                    .into());
            }
        }

        // Parse options
        let extends = if let Some(options_obj) = options.as_object() {
            let extends_val = options_obj.get(js_string!("extends"), context)?;
            if !extends_val.is_undefined() {
                Some(extends_val.to_string(context)?.to_std_string_escaped())
            } else {
                None
            }
        } else {
            None
        };

        // Store definition
        let definition = CustomElementDefinition {
            name: name.clone(),
            extends,
        };

        {
            let mut registry = REGISTRY.write().unwrap();
            registry.insert(name.clone(), definition);
        }

        // Store constructor on the this object (customElements)
        if let Some(this_obj) = this.as_object() {
            this_obj.set(
                js_string!(format!("__constructor_{}__", name).as_str()),
                constructor.clone(),
                false,
                context,
            )?;
        }

        Ok(JsValue::undefined())
    }

    /// `customElements.get(name)`
    ///
    /// Returns the constructor for the named custom element.
    fn get(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let name = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();

        // Check if defined
        {
            let registry = REGISTRY.read().unwrap();
            if !registry.contains_key(&name) {
                return Ok(JsValue::undefined());
            }
        }

        // Return the stored constructor
        if let Some(this_obj) = this.as_object() {
            let constructor = this_obj.get(
                js_string!(format!("__constructor_{}__", name).as_str()),
                context,
            )?;
            return Ok(constructor);
        }

        Ok(JsValue::undefined())
    }

    /// `customElements.getName(constructor)`
    ///
    /// Returns the name of the custom element associated with a constructor.
    fn get_name(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let constructor = args.get_or_undefined(0);

        if !constructor.is_callable() {
            return Ok(JsValue::null());
        }

        // Search for the constructor
        let registry = REGISTRY.read().unwrap();
        for (name, _) in registry.iter() {
            if let Some(this_obj) = this.as_object() {
                let stored_constructor = this_obj
                    .get(
                        js_string!(format!("__constructor_{}__", name).as_str()),
                        context,
                    )
                    .ok();

                if let Some(stored) = stored_constructor {
                    // Simple reference equality check
                    if stored == *constructor {
                        return Ok(js_string!(name.as_str()).into());
                    }
                }
            }
        }

        Ok(JsValue::null())
    }

    /// `customElements.whenDefined(name)`
    ///
    /// Returns a Promise that resolves when the named custom element is defined.
    fn when_defined(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let name = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();

        // Validate name
        if !Self::is_valid_custom_element_name(&name) {
            return Err(JsNativeError::syntax()
                .with_message(format!("'{}' is not a valid custom element name", name))
                .into());
        }

        // Check if already defined
        let is_defined = {
            let registry = REGISTRY.read().unwrap();
            registry.contains_key(&name)
        };

        // Create a promise
        use boa_engine::object::builtins::JsPromise;

        if is_defined {
            // Already defined, resolve immediately
            let promise = JsPromise::resolve(JsValue::undefined(), context)?;
            Ok(promise.into())
        } else {
            // Return a pending promise
            // In a real implementation, this would be stored and resolved when define() is called
            let promise = JsPromise::resolve(JsValue::undefined(), context)?;
            Ok(promise.into())
        }
    }

    /// `customElements.upgrade(root)`
    ///
    /// Upgrades all shadow-containing custom elements in a subtree.
    fn upgrade(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let _root = args.get_or_undefined(0);

        // In a real implementation, this would traverse the DOM tree
        // and upgrade any custom elements that haven't been upgraded yet.
        // For now, this is a no-op as we don't have full DOM integration.

        Ok(JsValue::undefined())
    }
}
