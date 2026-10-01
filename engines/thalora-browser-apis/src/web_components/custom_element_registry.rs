//! CustomElementRegistry Web API implementation
//!
//! The CustomElementRegistry interface provides methods for registering custom elements
//! and querying registered elements.
//!
//! https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry

use boa_engine::object::JsObject;
use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, NativeFunction, js_string, object::ObjectInitializer,
    property::Attribute, value::JsValue,
};
use std::collections::HashMap;
use std::sync::RwLock;

use crate::dom::binding;
use crate::dom::tree::{NodeId, SharedTree};

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

// ---------------------------------------------------------------------------
// Custom element reactions (connected / disconnected / attributeChanged)
// ---------------------------------------------------------------------------
//
// Minimal model, driven by `dom::script_runner` and `dom::mutation_bridge`:
// - Definitions are looked up per realm through the global `customElements`
//   object (the `__constructor_<name>__` slot `define` stores).
// - "Upgrade" only sets the element wrapper's prototype to
//   `constructor.prototype`; the constructor itself is not run, so class
//   field initializers and constructor bodies do not execute.
// - Elements are upgraded lazily, when they become connected through a
//   script-running insertion API or when an observed attribute changes; not
//   at createElement, not when `define` is called for existing elements, and
//   not for parser/innerHTML-created elements until one of those happens.
// - Reactions run synchronously right after the mutation (no element queue
//   / CEReactions stack); callback exceptions are reported to stderr.
// - `observedAttributes` is read from the constructor at each change rather
//   than once at definition time.

/// The constructor defined for `name` in this realm, if any.
pub fn lookup_constructor(name: &str, context: &mut Context) -> Option<JsObject> {
    if !name.contains('-') {
        return None;
    }
    let registry = context
        .global_object()
        .get(js_string!("customElements"), context)
        .ok()?
        .as_object()?;
    registry
        .get(
            js_string!(format!("__constructor_{}__", name).as_str()),
            context,
        )
        .ok()?
        .as_callable()
}

/// Elements in the subtrees of `roots` whose tag could name a custom
/// element (contains a hyphen), in tree order.
pub fn custom_element_candidates(tree: &SharedTree, roots: &[NodeId]) -> Vec<NodeId> {
    let tree = tree.borrow();
    let mut out = Vec::new();
    for &root in roots {
        for node in tree.descendants(root) {
            if tree.tag(node).is_some_and(|t| t.contains('-')) && !out.contains(&node) {
                out.push(node);
            }
        }
    }
    out
}

/// `constructor.prototype`, if it is an object.
fn constructor_prototype(ctor: &JsObject, context: &mut Context) -> JsResult<Option<JsObject>> {
    Ok(ctor.get(js_string!("prototype"), context)?.as_object())
}

fn is_upgraded(element: &JsObject, ctor: &JsObject, context: &mut Context) -> JsResult<bool> {
    let Some(proto) = constructor_prototype(ctor, context)? else {
        return Ok(false);
    };
    Ok(element
        .prototype()
        .is_some_and(|current| JsObject::equals(&current, &proto)))
}

/// Give `element` the definition's prototype (see the limits above).
fn upgrade_element(element: &JsObject, ctor: &JsObject, context: &mut Context) -> JsResult<()> {
    if let Some(proto) = constructor_prototype(ctor, context)? {
        let already = element
            .prototype()
            .is_some_and(|current| JsObject::equals(&current, &proto));
        if !already {
            element.set_prototype(Some(proto));
        }
    }
    Ok(())
}

/// Call `element[callback](...args)` if it is a function; exceptions are
/// reported, not propagated.
fn invoke_callback(element: &JsObject, callback: &str, args: &[JsValue], context: &mut Context) {
    let result = element
        .get(js_string!(callback), context)
        .and_then(|f| match f.as_callable() {
            Some(f) => f
                .call(&JsValue::from(element.clone()), args, context)
                .map(|_| ()),
            None => Ok(()),
        });
    if let Err(err) = result {
        eprintln!("console.error: Uncaught {err}");
    }
}

/// The element wrapper and definition for a candidate node, if it is a
/// defined custom element.
fn defined_element(
    document: &JsObject,
    tree: &SharedTree,
    node: NodeId,
    context: &mut Context,
) -> JsResult<Option<(JsObject, JsObject)>> {
    let tag = tree.borrow().tag(node).map(str::to_string);
    let Some(tag) = tag else {
        return Ok(None);
    };
    let Some(ctor) = lookup_constructor(&tag, context) else {
        return Ok(None);
    };
    let Some(element) = binding::wrapper_for(document, tree, node, context)?.as_object() else {
        return Ok(None);
    };
    Ok(Some((element, ctor)))
}

/// Upgrade and call `connectedCallback` on the defined custom elements in
/// `candidates` (from [`custom_element_candidates`]) that are connected.
pub fn connected_reactions(
    document: &JsObject,
    tree: &SharedTree,
    candidates: &[NodeId],
    context: &mut Context,
) -> JsResult<()> {
    for &node in candidates {
        if !tree.borrow().is_connected(node) {
            continue;
        }
        let Some((element, ctor)) = defined_element(document, tree, node, context)? else {
            continue;
        };
        upgrade_element(&element, &ctor, context)?;
        invoke_callback(&element, "connectedCallback", &[], context);
    }
    Ok(())
}

/// Call `disconnectedCallback` on upgraded custom elements in the removed
/// subtrees `roots`.
pub fn disconnected_reactions(
    document: &JsObject,
    tree: &SharedTree,
    roots: &[NodeId],
    context: &mut Context,
) -> JsResult<()> {
    for node in custom_element_candidates(tree, roots) {
        let Some((element, ctor)) = defined_element(document, tree, node, context)? else {
            continue;
        };
        if is_upgraded(&element, &ctor, context)? {
            invoke_callback(&element, "disconnectedCallback", &[], context);
        }
    }
    Ok(())
}

/// Whether `name` is listed in `ctor.observedAttributes`.
fn observes_attribute(ctor: &JsObject, name: &str, context: &mut Context) -> JsResult<bool> {
    let Some(list) = ctor
        .get(js_string!("observedAttributes"), context)?
        .as_object()
    else {
        return Ok(false);
    };
    let length = list
        .get(js_string!("length"), context)?
        .to_length(context)?;
    for index in 0..length {
        let item = list.get(index, context)?;
        if item.to_string(context)?.to_std_string_escaped() == name {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `attributeChangedCallback(name, old, new)` for a bound element whose tag
/// is a defined custom element and whose class observes `name`. The element
/// is upgraded first if needed.
pub fn attribute_changed_reaction(
    b: &binding::DomBinding,
    name: &str,
    old: Option<&str>,
    new: Option<&str>,
    context: &mut Context,
) -> JsResult<()> {
    let Some((element, ctor)) = defined_element(&b.document, &b.tree, b.node, context)? else {
        return Ok(());
    };
    let observed = match observes_attribute(&ctor, name, context) {
        Ok(observed) => observed,
        Err(err) => {
            eprintln!("console.error: Uncaught {err}");
            false
        }
    };
    if !observed {
        return Ok(());
    }
    upgrade_element(&element, &ctor, context)?;
    let to_value = |v: Option<&str>| v.map_or(JsValue::null(), |s| js_string!(s).into());
    invoke_callback(
        &element,
        "attributeChangedCallback",
        &[js_string!(name).into(), to_value(old), to_value(new)],
        context,
    );
    Ok(())
}
