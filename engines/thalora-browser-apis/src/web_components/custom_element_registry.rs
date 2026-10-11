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

use crate::dom::binding;
use crate::dom::tree::{NodeId, SharedTree};

fn constructor_slot(name: &str) -> String {
    format!("__constructor_{name}__")
}

fn extends_slot(name: &str) -> String {
    format!("__extends_{name}__")
}

fn promise_slot(name: &str) -> String {
    format!("__when_defined_{name}__")
}

fn resolve_slot(name: &str) -> String {
    format!("__when_defined_resolve_{name}__")
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
    /// Definitions live on this realm's `customElements` object, so every
    /// page (each navigation gets a fresh realm) starts with an empty
    /// registry. Elements of this name already in the document are upgraded
    /// (connected ones get `connectedCallback`) and `whenDefined` promises
    /// resolve.
    fn define(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let name = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        let constructor = args.get_or_undefined(1);
        let options = args.get_or_undefined(2);

        if !Self::is_valid_custom_element_name(&name) {
            return Err(JsNativeError::syntax()
                .with_message(format!("'{}' is not a valid custom element name", name))
                .into());
        }
        if !constructor.is_callable() {
            return Err(JsNativeError::typ()
                .with_message("Custom element constructor must be a function")
                .into());
        }
        let Some(registry) = this.as_object() else {
            return Err(JsNativeError::typ()
                .with_message("customElements.define called on non-object")
                .into());
        };
        if registry.has_own_property(js_string!(constructor_slot(&name)), context)? {
            return Err(JsNativeError::error()
                .with_message(format!(
                    "Custom element '{}' has already been defined",
                    name
                ))
                .into());
        }

        if let Some(options_obj) = options.as_object() {
            let extends = options_obj.get(js_string!("extends"), context)?;
            if !extends.is_undefined() {
                registry.set(js_string!(extends_slot(&name)), extends, false, context)?;
            }
        }
        registry.set(
            js_string!(constructor_slot(&name)),
            constructor.clone(),
            false,
            context,
        )?;

        // Upgrade elements of this name that already exist in the document
        upgrade_existing(&name, context)?;

        // Resolve pending whenDefined(name) promises
        let resolve = registry.get(js_string!(resolve_slot(&name)), context)?;
        if let Some(resolve) = resolve.as_callable() {
            resolve.call(
                &JsValue::undefined(),
                std::slice::from_ref(constructor),
                context,
            )?;
        }
        Ok(JsValue::undefined())
    }

    /// `customElements.get(name)`
    fn get(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let name = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        match this.as_object() {
            Some(registry) => registry.get(js_string!(constructor_slot(&name)), context),
            None => Ok(JsValue::undefined()),
        }
    }

    /// `customElements.getName(constructor)`
    fn get_name(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let constructor = args.get_or_undefined(0);
        if !constructor.is_callable() {
            return Ok(JsValue::null());
        }
        let Some(registry) = this.as_object() else {
            return Ok(JsValue::null());
        };
        for key in registry.own_property_keys(context)? {
            let key_string = key.to_string();
            let Some(name) = key_string
                .strip_prefix("__constructor_")
                .and_then(|rest| rest.strip_suffix("__"))
            else {
                continue;
            };
            if registry.get(key.clone(), context)? == *constructor {
                return Ok(js_string!(name).into());
            }
        }
        Ok(JsValue::null())
    }

    /// `customElements.whenDefined(name)`: resolves (with the constructor)
    /// once `name` is defined.
    fn when_defined(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        use boa_engine::object::builtins::JsPromise;

        let name = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        if !Self::is_valid_custom_element_name(&name) {
            return Err(JsNativeError::syntax()
                .with_message(format!("'{}' is not a valid custom element name", name))
                .into());
        }
        let Some(registry) = this.as_object() else {
            return Ok(JsPromise::resolve(JsValue::undefined(), context)?.into());
        };
        let defined = registry.get(js_string!(constructor_slot(&name)), context)?;
        if defined.is_callable() {
            return Ok(JsPromise::resolve(defined, context)?.into());
        }
        // One pending promise per name; define() calls its resolver
        let existing = registry.get(js_string!(promise_slot(&name)), context)?;
        if existing.is_object() {
            return Ok(existing);
        }
        let (promise, resolvers) = JsPromise::new_pending(context);
        registry.set(
            js_string!(promise_slot(&name)),
            promise.clone(),
            false,
            context,
        )?;
        registry.set(
            js_string!(resolve_slot(&name)),
            resolvers.resolve.clone(),
            false,
            context,
        )?;
        Ok(promise.into())
    }

    /// `customElements.upgrade(root)`: upgrade defined custom elements in
    /// `root`'s subtree (connected ones also get `connectedCallback`).
    fn upgrade(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if let Some(root) = args.get_or_undefined(0).as_object()
            && let Some(b) = binding::binding_of(&root)
        {
            let candidates = custom_element_candidates(&b.tree, &[b.node]);
            upgrade_nodes(&b.document, &b.tree, &candidates, context)?;
        }
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
// - "Upgrade" sets the element wrapper's prototype to
//   `constructor.prototype` and runs the constructor on it through the
//   element construction stack (HTMLElement's constructor hands back the
//   element being upgraded), so class fields and constructor bodies run.
// - Elements are upgraded when `define` runs (existing elements in the
//   document), by `customElements.upgrade(root)`, when they become connected
//   through a script-running insertion API, or when an observed attribute
//   changes; not at createElement.
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
        .get(js_string!(constructor_slot(name)), context)
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

thread_local! {
    /// The HTML "element construction stack": elements being upgraded. The
    /// HTMLElement constructor returns the top one instead of creating a
    /// new element, so `super()` in a custom element class binds `this` to
    /// the element being upgraded.
    static CONSTRUCTION_STACK: std::cell::RefCell<Vec<JsObject>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// For the HTMLElement constructor: the element being upgraded, if any.
pub(crate) fn take_constructing_element() -> Option<JsObject> {
    CONSTRUCTION_STACK.with(|stack| stack.borrow_mut().pop())
}

/// The custom element name `ctor` was defined with in this realm.
pub(crate) fn name_for_constructor(ctor: &JsObject, context: &mut Context) -> Option<String> {
    let registry = context
        .global_object()
        .get(js_string!("customElements"), context)
        .ok()?
        .as_object()?;
    for key in registry.own_property_keys(context).ok()? {
        let key_string = key.to_string();
        let Some(name) = key_string
            .strip_prefix("__constructor_")
            .and_then(|rest| rest.strip_suffix("__"))
        else {
            continue;
        };
        if let Ok(value) = registry.get(key.clone(), context)
            && value
                .as_object()
                .is_some_and(|value| JsObject::equals(&value, ctor))
        {
            return Some(name.to_string());
        }
    }
    None
}

/// Upgrade `element`: give it the definition's prototype and run the
/// constructor on it (via the construction stack). Constructor errors are
/// reported, as for any failed upgrade.
fn upgrade_element(element: &JsObject, ctor: &JsObject, context: &mut Context) -> JsResult<()> {
    let Some(proto) = constructor_prototype(ctor, context)? else {
        return Ok(());
    };
    if element
        .prototype()
        .is_some_and(|current| JsObject::equals(&current, &proto))
    {
        return Ok(());
    }
    element.set_prototype(Some(proto));
    CONSTRUCTION_STACK.with(|stack| stack.borrow_mut().push(element.clone()));
    let result = ctor.construct(&[], Some(ctor), context);
    // If the constructor never reached HTMLElement (threw early), drop the entry
    CONSTRUCTION_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        if stack
            .last()
            .is_some_and(|top| JsObject::equals(top, element))
        {
            stack.pop();
        }
    });
    match result {
        Ok(constructed) if !JsObject::equals(&constructed, element) => {
            eprintln!(
                "console.error: custom element constructor did not produce the element being upgraded"
            );
        }
        Ok(_) => {}
        Err(err) => eprintln!("console.error: Uncaught {err}"),
    }
    Ok(())
}

/// `document.createElement(name)` for a defined custom element: construct
/// it right away (synchronous custom element creation).
pub(crate) fn construct_created_element(
    element: &JsValue,
    name: &str,
    context: &mut Context,
) -> JsResult<()> {
    if let (Some(element), Some(ctor)) = (element.as_object(), lookup_constructor(name, context)) {
        upgrade_element(&element, &ctor, context)?;
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

/// Upgrade not-yet-upgraded defined custom elements among `candidates`;
/// connected ones get `connectedCallback` (as for an upgrade per spec).
fn upgrade_nodes(
    document: &JsObject,
    tree: &SharedTree,
    candidates: &[NodeId],
    context: &mut Context,
) -> JsResult<()> {
    for &node in candidates {
        let Some((element, ctor)) = defined_element(document, tree, node, context)? else {
            continue;
        };
        if is_upgraded(&element, &ctor, context)? {
            continue;
        }
        upgrade_element(&element, &ctor, context)?;
        if tree.borrow().is_connected(node) {
            invoke_callback(&element, "connectedCallback", &[], context);
        }
    }
    Ok(())
}

/// After `define(name)`: upgrade `<name>` elements in the global document.
fn upgrade_existing(name: &str, context: &mut Context) -> JsResult<()> {
    let document = context
        .global_object()
        .get(js_string!("document"), context)?;
    let Some((document, tree)) = binding::document_tree(&document) else {
        return Ok(());
    };
    let nodes = {
        let t = tree.borrow();
        t.elements_by_tag(t.document(), name)
    };
    upgrade_nodes(&document, &tree, &nodes, context)
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
