//! Binding between JS DOM objects and the persistent [`DomTree`].
//!
//! A document whose HTML was loaded keeps a [`DomTree`]; every JS node object
//! for one of its nodes is *bound* to it (document + tree + node id) and
//! reads and writes the tree instead of its own legacy fields. Wrappers are
//! cached per document, so the same node is always the same JS object.
//!
//! Elements that aren't bound (empty documents, `THALORA_DOM=legacy`) keep
//! the legacy behaviour: the natives below are only reached through
//! [`bound_element`] checks in `element.rs` / `document.rs`.
//!
//! Rule: never hold a tree borrow while calling back into JS. Each native
//! reads what it needs, drops the borrow, then creates wrappers.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsString,
    builtins::{BuiltInBuilder, BuiltInConstructor, array::Array},
    js_string,
    object::JsObject,
    value::JsValue,
};
use boa_gc::{Finalize, Trace};
use std::rc::Rc;

use crate::dom::document::DocumentData;
use crate::dom::element::ElementData;
use crate::dom::text::TextData;
use crate::dom::tree::{DomTree, NodeId, NodeKind, SelectError, SharedTree, TreeError};

/// Where a JS node lives in its document's tree.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct DomBinding {
    /// The owning document object (keeps the wrapper cache alive).
    pub document: JsObject,
    #[unsafe_ignore_trace]
    pub tree: SharedTree,
    #[unsafe_ignore_trace]
    pub node: NodeId,
}

impl DomBinding {
    fn at(&self, node: NodeId) -> Self {
        Self {
            document: self.document.clone(),
            tree: self.tree.clone(),
            node,
        }
    }
}

/// Whether documents build a persistent tree (`THALORA_DOM=legacy` opts out).
pub fn tree_dom_enabled() -> bool {
    std::env::var("THALORA_DOM").map_or(true, |v| !v.eq_ignore_ascii_case("legacy"))
}

fn tree_error(e: TreeError) -> boa_engine::JsError {
    JsNativeError::typ().with_message(e.to_string()).into()
}

fn select_error(e: SelectError) -> boa_engine::JsError {
    JsNativeError::syntax().with_message(e.to_string()).into()
}

/// HTML elements get lowercased attribute names; SVG/MathML keep case.
fn is_html_element(tree: &DomTree, node: NodeId) -> bool {
    let mut current = Some(node);
    while let Some(id) = current {
        if tree
            .tag(id)
            .is_some_and(|t| t.eq_ignore_ascii_case("svg") || t.eq_ignore_ascii_case("math"))
        {
            return false;
        }
        current = tree.parent(id);
    }
    true
}

/// Normalize an attribute name for `node` (lowercase on HTML elements).
pub fn attr_name(b: &DomBinding, name: &str) -> String {
    if is_html_element(&b.tree.borrow(), b.node) {
        name.to_ascii_lowercase()
    } else {
        name.to_string()
    }
}

// ---------------------------------------------------------------------------
// Lookups
// ---------------------------------------------------------------------------

/// The binding of `this` if it is a bound element.
pub fn bound_element(this: &JsValue) -> Option<DomBinding> {
    let obj = this.as_object()?;
    let element = obj.downcast_ref::<ElementData>()?;
    element.binding()
}

/// The binding of a bound element, text or comment object.
pub fn binding_of(obj: &JsObject) -> Option<DomBinding> {
    if let Some(element) = obj.downcast_ref::<ElementData>() {
        return element.binding();
    }
    if let Some(text) = obj.downcast_ref::<TextData>() {
        return text.character_data().binding();
    }
    None
}

/// `(document object, tree)` if `this` is a document with a tree.
pub fn document_tree(this: &JsValue) -> Option<(JsObject, SharedTree)> {
    let obj = this.as_object()?;
    let tree = obj.downcast_ref::<DocumentData>()?.tree()?;
    Some((obj, tree))
}

/// The JS object for `node`, creating and caching it on first use.
/// Returns `null` for doctypes and unknown ids.
pub fn wrapper_for(
    document: &JsObject,
    tree: &SharedTree,
    node: NodeId,
    context: &mut Context,
) -> JsResult<JsValue> {
    let cached = {
        let Some(doc) = document.downcast_ref::<DocumentData>() else {
            return Ok(JsValue::null());
        };
        doc.cached_wrapper(node)
    };
    if let Some(cached) = cached {
        return Ok(cached.into());
    }
    let kind = tree.borrow().kind(node).cloned();
    let binding = DomBinding {
        document: document.clone(),
        tree: tree.clone(),
        node,
    };
    let object = match kind {
        None | Some(NodeKind::Doctype { .. }) => return Ok(JsValue::null()),
        Some(NodeKind::Document) => {
            // The document node of a document's own tree is the document
            return Ok(if node == tree.borrow().document() {
                document.clone().into()
            } else {
                JsValue::null()
            });
        }
        Some(NodeKind::Element { tag, .. }) => {
            let value = crate::dom::document::build_element_object(&tag, context)?;
            let Some(object) = value.as_object() else {
                return Ok(JsValue::null());
            };
            if let Some(element) = object.downcast_ref::<ElementData>() {
                element.set_binding(binding.clone());
            }
            refresh_layout(&binding, &object);
            object
        }
        Some(NodeKind::Text(data)) => {
            let constructor = context.intrinsics().constructors().text().constructor();
            let value = crate::dom::text::Text::constructor(
                &constructor.into(),
                &[JsString::from(data).into()],
                context,
            )?;
            let Some(object) = value.as_object() else {
                return Ok(JsValue::null());
            };
            if let Some(text) = object.downcast_ref::<TextData>() {
                text.character_data().set_binding(binding.clone());
            }
            object
        }
        Some(NodeKind::Comment(data)) => {
            let value = crate::dom::document::create_comment_object(&data, context)?;
            let Some(object) = value.as_object() else {
                return Ok(JsValue::null());
            };
            object
        }
    };
    if let Some(doc) = document.downcast_ref::<DocumentData>() {
        doc.cache_wrapper(node, object.clone());
    }
    Ok(object.into())
}

fn wrap(b: &DomBinding, node: Option<NodeId>, context: &mut Context) -> JsResult<JsValue> {
    match node {
        Some(node) => wrapper_for(&b.document, &b.tree, node, context),
        None => Ok(JsValue::null()),
    }
}

fn wrap_all(b: &DomBinding, nodes: Vec<NodeId>, context: &mut Context) -> JsResult<Vec<JsValue>> {
    let mut out = Vec::with_capacity(nodes.len());
    for node in nodes {
        let value = wrapper_for(&b.document, &b.tree, node, context)?;
        if !value.is_null() {
            out.push(value);
        }
    }
    Ok(out)
}

/// An array with a NodeList-style `item()` method.
pub fn node_array(values: Vec<JsValue>, context: &mut Context) -> JsResult<JsValue> {
    let array = Array::create_array_from_list(values, context);
    let item = BuiltInBuilder::callable(context.realm(), |this, args, ctx| {
        let index = args.get_or_undefined(0).to_u32(ctx)?;
        if let Some(arr) = this.as_object()
            && let Ok(val) = arr.get(index, ctx)
            && !val.is_undefined()
        {
            return Ok(val);
        }
        Ok(JsValue::null())
    })
    .name(js_string!("item"))
    .build();
    array.set(js_string!("item"), item, false, context)?;
    Ok(array.into())
}

fn wrap_array(b: &DomBinding, nodes: Vec<NodeId>, context: &mut Context) -> JsResult<JsValue> {
    let values = wrap_all(b, nodes, context)?;
    node_array(values, context)
}

/// Copy the element's layout rect (keyed by its CSS path) onto it.
pub fn refresh_layout(b: &DomBinding, object: &JsObject) {
    let Some(path) = b.tree.borrow().css_path(b.node) else {
        return;
    };
    let rect = b
        .document
        .downcast_ref::<DocumentData>()
        .and_then(|doc| doc.get_layout_rect(&path));
    if let (Some(rect), Some(element)) = (rect, object.downcast_ref::<ElementData>()) {
        element.set_bounding_rect(rect.x, rect.y, rect.width, rect.height);
    }
}

/// Refresh `this`'s layout rect if it is a bound element.
pub fn refresh_layout_of(this: &JsValue) {
    if let (Some(b), Some(obj)) = (bound_element(this), this.as_object()) {
        refresh_layout(&b, &obj);
    }
}

// ---------------------------------------------------------------------------
// Bringing arguments into the tree
// ---------------------------------------------------------------------------

/// The node for a JS argument in `b`'s tree, adopting unbound (or
/// foreign) elements and text nodes by copying them into the tree and
/// binding the object to the copy. Strings become text nodes.
pub fn node_of(b: &DomBinding, value: &JsValue, context: &mut Context) -> JsResult<NodeId> {
    let Some(obj) = value.as_object() else {
        let text = value.to_string(context)?.to_std_string_escaped();
        return Ok(b.tree.borrow_mut().create_text(&text));
    };
    if let Some(existing) = binding_of(&obj)
        && Rc::ptr_eq(&existing.tree, &b.tree)
    {
        return Ok(existing.node);
    }
    adopt(b, &obj)?.ok_or_else(|| {
        JsNativeError::typ()
            .with_message("HierarchyRequestError: argument is not a Node")
            .into()
    })
}

/// Like [`node_of`] but `null`/`undefined` (and non-objects) give `None`.
fn optional_node(
    b: &DomBinding,
    value: &JsValue,
    context: &mut Context,
) -> JsResult<Option<NodeId>> {
    if value.is_null_or_undefined() {
        Ok(None)
    } else {
        node_of(b, value, context).map(Some)
    }
}

/// Copy an object that isn't in `b`'s tree into it.
fn adopt(b: &DomBinding, obj: &JsObject) -> JsResult<Option<NodeId>> {
    // Text
    let text = obj
        .downcast_ref::<TextData>()
        .map(|t| t.character_data().get_data());
    if let Some(data) = text {
        let node = b.tree.borrow_mut().create_text(&data);
        bind_object(b, obj, node);
        return Ok(Some(node));
    }

    // Element (from another tree, or never bound)
    let legacy = obj.downcast_ref::<ElementData>().map(|element| {
        let foreign = element
            .binding()
            .map(|f| f.tree.borrow().outer_html(f.node));
        (
            element.get_tag_name(),
            element.legacy_attributes(),
            element.get_children(),
            element.legacy_inner_html(),
            foreign,
        )
    });
    if let Some((tag, attrs, children, inner_html, foreign)) = legacy {
        let node = if let Some(markup) = foreign {
            let mut tree = b.tree.borrow_mut();
            let nodes = tree.parse_fragment(&markup);
            match nodes.first() {
                Some(&n) => n,
                None => tree.create_element(&tag.to_ascii_lowercase()),
            }
        } else {
            let node = b
                .tree
                .borrow_mut()
                .create_element_with_attrs(&tag.to_ascii_lowercase(), attrs);
            if children.is_empty() {
                if !inner_html.is_empty() {
                    b.tree
                        .borrow_mut()
                        .set_inner_html(node, &inner_html)
                        .map_err(tree_error)?;
                }
            } else {
                for child in children {
                    if let Some(child_node) = adopt(b, &child)? {
                        b.tree
                            .borrow_mut()
                            .append(node, child_node)
                            .map_err(tree_error)?;
                    }
                }
            }
            node
        };
        bind_object(b, obj, node);
        return Ok(Some(node));
    }

    // DocumentFragment: its children are inserted instead; callers handle
    // fragments through `nodes_of`.
    Ok(None)
}

fn bind_object(b: &DomBinding, obj: &JsObject, node: NodeId) {
    let binding = b.at(node);
    if let Some(element) = obj.downcast_ref::<ElementData>() {
        element.set_binding(binding);
    } else if let Some(text) = obj.downcast_ref::<TextData>() {
        text.character_data().set_binding(binding);
    }
    if let Some(doc) = b.document.downcast_ref::<DocumentData>() {
        doc.cache_wrapper(node, obj.clone());
    }
}

/// Nodes to insert for one argument: a DocumentFragment contributes its
/// children (and is emptied), anything else one node.
fn nodes_of(b: &DomBinding, value: &JsValue, context: &mut Context) -> JsResult<Vec<NodeId>> {
    if let Some(obj) = value.as_object() {
        let fragment_children = obj
            .downcast_ref::<crate::dom::document_fragment::DocumentFragmentData>()
            .map(|f| f.get_children());
        if let Some(children) = fragment_children {
            let mut nodes = Vec::with_capacity(children.len());
            for child in &children {
                nodes.push(node_of(b, &JsValue::from(child.clone()), context)?);
            }
            if let Some(fragment) =
                obj.downcast_ref::<crate::dom::document_fragment::DocumentFragmentData>()
            {
                let _ = fragment.replace_children_impl(Vec::new());
            }
            return Ok(nodes);
        }
    }
    Ok(vec![node_of(b, value, context)?])
}

fn args_to_nodes(b: &DomBinding, args: &[JsValue], context: &mut Context) -> JsResult<Vec<NodeId>> {
    let mut nodes = Vec::new();
    for arg in args {
        nodes.extend(nodes_of(b, arg, context)?);
    }
    Ok(nodes)
}

// ---------------------------------------------------------------------------
// Mutation notifications
// ---------------------------------------------------------------------------

/// Called after any child-list change under `parent` (R7/R8 hook).
fn after_child_list_change(
    b: &DomBinding,
    parent: NodeId,
    added: &[NodeId],
    removed: &[NodeId],
    context: &mut Context,
) -> JsResult<()> {
    crate::dom::mutation_bridge::child_list_changed(b, parent, added, removed, context)
}

/// Insert `nodes` under `parent` before `before` and notify.
fn insert_nodes(
    b: &DomBinding,
    parent: NodeId,
    nodes: &[NodeId],
    before: Option<NodeId>,
    context: &mut Context,
) -> JsResult<()> {
    if nodes.is_empty() {
        return Ok(());
    }
    {
        let mut tree = b.tree.borrow_mut();
        for &node in nodes {
            tree.insert_before(parent, node, before)
                .map_err(tree_error)?;
        }
    }
    after_child_list_change(b, parent, nodes, &[], context)
}

fn remove_node(b: &DomBinding, node: NodeId, context: &mut Context) -> JsResult<()> {
    let parent = b.tree.borrow().parent(node);
    if let Some(parent) = parent {
        b.tree.borrow_mut().remove(node).map_err(tree_error)?;
        after_child_list_change(b, parent, &[], &[node], context)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Element natives (bound path)
// ---------------------------------------------------------------------------

pub fn children(b: &DomBinding, context: &mut Context) -> JsResult<JsValue> {
    let nodes = b.tree.borrow().element_children(b.node);
    wrap_array(b, nodes, context)
}

pub fn child_nodes(b: &DomBinding, context: &mut Context) -> JsResult<JsValue> {
    let nodes = b.tree.borrow().children(b.node).to_vec();
    wrap_array(b, nodes, context)
}

pub fn parent_node(b: &DomBinding, context: &mut Context) -> JsResult<JsValue> {
    let parent = b.tree.borrow().parent(b.node);
    wrap(b, parent, context)
}

pub fn parent_element(b: &DomBinding, context: &mut Context) -> JsResult<JsValue> {
    let parent = {
        let tree = b.tree.borrow();
        tree.parent(b.node).filter(|&p| tree.is_element(p))
    };
    wrap(b, parent, context)
}

pub fn first_child(b: &DomBinding, context: &mut Context) -> JsResult<JsValue> {
    let node = b.tree.borrow().children(b.node).first().copied();
    wrap(b, node, context)
}

pub fn last_child(b: &DomBinding, context: &mut Context) -> JsResult<JsValue> {
    let node = b.tree.borrow().children(b.node).last().copied();
    wrap(b, node, context)
}

pub fn next_sibling(b: &DomBinding, context: &mut Context) -> JsResult<JsValue> {
    let node = b.tree.borrow().next_sibling(b.node);
    wrap(b, node, context)
}

pub fn previous_sibling(b: &DomBinding, context: &mut Context) -> JsResult<JsValue> {
    let node = b.tree.borrow().previous_sibling(b.node);
    wrap(b, node, context)
}

pub fn first_element_child(b: &DomBinding, context: &mut Context) -> JsResult<JsValue> {
    let node = b.tree.borrow().element_children(b.node).first().copied();
    wrap(b, node, context)
}

pub fn last_element_child(b: &DomBinding, context: &mut Context) -> JsResult<JsValue> {
    let node = b.tree.borrow().element_children(b.node).last().copied();
    wrap(b, node, context)
}

pub fn next_element_sibling(b: &DomBinding, context: &mut Context) -> JsResult<JsValue> {
    let node = {
        let tree = b.tree.borrow();
        let mut current = tree.next_sibling(b.node);
        while let Some(n) = current
            && !tree.is_element(n)
        {
            current = tree.next_sibling(n);
        }
        current
    };
    wrap(b, node, context)
}

pub fn previous_element_sibling(b: &DomBinding, context: &mut Context) -> JsResult<JsValue> {
    let node = {
        let tree = b.tree.borrow();
        let mut current = tree.previous_sibling(b.node);
        while let Some(n) = current
            && !tree.is_element(n)
        {
            current = tree.previous_sibling(n);
        }
        current
    };
    wrap(b, node, context)
}

pub fn child_element_count(b: &DomBinding) -> JsValue {
    (b.tree.borrow().element_children(b.node).len() as u32).into()
}

pub fn is_connected(b: &DomBinding) -> JsValue {
    b.tree.borrow().is_connected(b.node).into()
}

pub fn append_child(b: &DomBinding, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let child = args.get_or_undefined(0).clone();
    if !child.is_object() {
        return Err(JsNativeError::typ()
            .with_message("appendChild: argument is not a Node")
            .into());
    }
    let nodes = nodes_of(b, &child, context)?;
    insert_nodes(b, b.node, &nodes, None, context)?;
    Ok(child)
}

pub fn insert_before(b: &DomBinding, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let child = args.get_or_undefined(0).clone();
    if !child.is_object() {
        return Err(JsNativeError::typ()
            .with_message("insertBefore: argument is not a Node")
            .into());
    }
    let before = optional_node(b, args.get_or_undefined(1), context)?;
    if let Some(before) = before
        && b.tree.borrow().parent(before) != Some(b.node)
    {
        return Err(JsNativeError::typ()
            .with_message("NotFoundError: reference node is not a child of this node")
            .into());
    }
    let nodes = nodes_of(b, &child, context)?;
    insert_nodes(b, b.node, &nodes, before, context)?;
    Ok(child)
}

pub fn remove_child(b: &DomBinding, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let child = args.get_or_undefined(0).clone();
    let node = optional_node(b, &child, context)?;
    match node {
        Some(node) if b.tree.borrow().parent(node) == Some(b.node) => {
            remove_node(b, node, context)?;
            Ok(child)
        }
        _ => Err(JsNativeError::typ()
            .with_message("NotFoundError: the node to be removed is not a child of this node")
            .into()),
    }
}

pub fn replace_child(b: &DomBinding, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let new_child = args.get_or_undefined(0).clone();
    let old_child = args.get_or_undefined(1).clone();
    let old = optional_node(b, &old_child, context)?;
    let Some(old) = old.filter(|&o| b.tree.borrow().parent(o) == Some(b.node)) else {
        return Err(JsNativeError::typ()
            .with_message("NotFoundError: the node to be replaced is not a child of this node")
            .into());
    };
    let new_nodes = nodes_of(b, &new_child, context)?;
    {
        let mut tree = b.tree.borrow_mut();
        for &node in &new_nodes {
            if node != old {
                tree.insert_before(b.node, node, Some(old))
                    .map_err(tree_error)?;
            }
        }
        if !new_nodes.contains(&old) {
            tree.remove(old).map_err(tree_error)?;
        }
    }
    after_child_list_change(b, b.node, &new_nodes, &[old], context)?;
    Ok(old_child)
}

pub fn clone_node(b: &DomBinding, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let deep = args.get_or_undefined(0).to_boolean();
    let copy = b
        .tree
        .borrow_mut()
        .clone_node(b.node, deep)
        .map_err(tree_error)?;
    wrap(b, Some(copy), context)
}

pub fn contains(b: &DomBinding, args: &[JsValue]) -> JsValue {
    let Some(other) = args.get_or_undefined(0).as_object() else {
        return false.into();
    };
    match binding_of(&other) {
        Some(o) if Rc::ptr_eq(&o.tree, &b.tree) => {
            b.tree.borrow().is_inclusive_ancestor(b.node, o.node).into()
        }
        _ => false.into(),
    }
}

pub fn closest(b: &DomBinding, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let selector = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    let found = b
        .tree
        .borrow()
        .closest(b.node, &selector)
        .map_err(select_error)?;
    wrap(b, found, context)
}

pub fn matches(b: &DomBinding, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let selector = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    Ok(b.tree
        .borrow()
        .matches(b.node, &selector)
        .map_err(select_error)?
        .into())
}

pub fn query_selector(
    b: &DomBinding,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let selector = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    let found = b
        .tree
        .borrow()
        .select_first(b.node, &selector)
        .map_err(select_error)?;
    wrap(b, found, context)
}

pub fn query_selector_all(
    b: &DomBinding,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let selector = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    let found = b
        .tree
        .borrow()
        .select(b.node, &selector)
        .map_err(select_error)?;
    wrap_array(b, found, context)
}

pub fn get_elements_by_tag_name(
    b: &DomBinding,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let tag = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    let found = b.tree.borrow().elements_by_tag(b.node, &tag);
    wrap_array(b, found, context)
}

pub fn get_elements_by_class_name(
    b: &DomBinding,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let names = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    let found = b.tree.borrow().elements_by_class(b.node, &names);
    wrap_array(b, found, context)
}

pub fn get_elements_by_name(
    b: &DomBinding,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let name = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    let found = b.tree.borrow().elements_by_name(b.node, &name);
    wrap_array(b, found, context)
}

/// `ChildNode.remove()`
pub fn remove(b: &DomBinding, context: &mut Context) -> JsResult<JsValue> {
    remove_node(b, b.node, context)?;
    Ok(JsValue::undefined())
}

/// `ParentNode.append(...nodes)`
pub fn append(b: &DomBinding, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let nodes = args_to_nodes(b, args, context)?;
    insert_nodes(b, b.node, &nodes, None, context)?;
    Ok(JsValue::undefined())
}

/// `ParentNode.prepend(...nodes)`
pub fn prepend(b: &DomBinding, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let nodes = args_to_nodes(b, args, context)?;
    let first = b.tree.borrow().children(b.node).first().copied();
    insert_nodes(b, b.node, &nodes, first, context)?;
    Ok(JsValue::undefined())
}

/// `ChildNode.before(...nodes)`
pub fn before(b: &DomBinding, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let nodes = args_to_nodes(b, args, context)?;
    let parent = b.tree.borrow().parent(b.node);
    if let Some(parent) = parent {
        insert_nodes(b, parent, &nodes, Some(b.node), context)?;
    }
    Ok(JsValue::undefined())
}

/// `ChildNode.after(...nodes)`
pub fn after(b: &DomBinding, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let nodes = args_to_nodes(b, args, context)?;
    let (parent, next) = {
        let tree = b.tree.borrow();
        (tree.parent(b.node), tree.next_sibling(b.node))
    };
    if let Some(parent) = parent {
        insert_nodes(b, parent, &nodes, next, context)?;
    }
    Ok(JsValue::undefined())
}

/// `ChildNode.replaceWith(...nodes)`
pub fn replace_with(b: &DomBinding, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let nodes = args_to_nodes(b, args, context)?;
    let parent = b.tree.borrow().parent(b.node);
    if let Some(parent) = parent {
        let nodes: Vec<NodeId> = nodes.into_iter().filter(|&n| n != b.node).collect();
        insert_nodes(b, parent, &nodes, Some(b.node), context)?;
        remove_node(b, b.node, context)?;
    }
    Ok(JsValue::undefined())
}

/// `Element.insertAdjacentHTML(position, html)`
pub fn insert_adjacent_html(
    b: &DomBinding,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let position = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped()
        .to_ascii_lowercase();
    let html = args
        .get_or_undefined(1)
        .to_string(context)?
        .to_std_string_escaped();
    let nodes = b.tree.borrow_mut().parse_fragment(&html);
    let (parent, first, next) = {
        let tree = b.tree.borrow();
        (
            tree.parent(b.node),
            tree.children(b.node).first().copied(),
            tree.next_sibling(b.node),
        )
    };
    match position.as_str() {
        "beforebegin" => {
            if let Some(parent) = parent {
                insert_nodes(b, parent, &nodes, Some(b.node), context)?;
            }
        }
        "afterbegin" => insert_nodes(b, b.node, &nodes, first, context)?,
        "beforeend" => insert_nodes(b, b.node, &nodes, None, context)?,
        "afterend" => {
            if let Some(parent) = parent {
                insert_nodes(b, parent, &nodes, next, context)?;
            }
        }
        _ => {
            return Err(JsNativeError::syntax()
                .with_message(format!("insertAdjacentHTML: invalid position '{position}'"))
                .into());
        }
    }
    Ok(JsValue::undefined())
}

/// `Element.insertAdjacentElement(position, element)`
pub fn insert_adjacent_element(
    b: &DomBinding,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let position = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped()
        .to_ascii_lowercase();
    let element = args.get_or_undefined(1).clone();
    let node = node_of(b, &element, context)?;
    let (parent, first, next) = {
        let tree = b.tree.borrow();
        (
            tree.parent(b.node),
            tree.children(b.node).first().copied(),
            tree.next_sibling(b.node),
        )
    };
    let inserted = match position.as_str() {
        "beforebegin" => match parent {
            Some(parent) => {
                insert_nodes(b, parent, &[node], Some(b.node), context).map(|_| true)?
            }
            None => false,
        },
        "afterbegin" => insert_nodes(b, b.node, &[node], first, context).map(|_| true)?,
        "beforeend" => insert_nodes(b, b.node, &[node], None, context).map(|_| true)?,
        "afterend" => match parent {
            Some(parent) => insert_nodes(b, parent, &[node], next, context).map(|_| true)?,
            None => false,
        },
        _ => {
            return Err(JsNativeError::syntax()
                .with_message(format!(
                    "insertAdjacentElement: invalid position '{position}'"
                ))
                .into());
        }
    };
    Ok(if inserted { element } else { JsValue::null() })
}

pub fn get_attribute_names(b: &DomBinding, context: &mut Context) -> JsResult<JsValue> {
    let names: Vec<JsValue> = b
        .tree
        .borrow()
        .attrs(b.node)
        .iter()
        .map(|(n, _)| JsString::from(n.as_str()).into())
        .collect();
    Ok(Array::create_array_from_list(names, context).into())
}

pub fn toggle_attribute(
    b: &DomBinding,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let name = attr_name(
        b,
        &args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped(),
    );
    let force = args
        .get(1)
        .filter(|v| !v.is_undefined())
        .map(JsValue::to_boolean);
    let present = b.tree.borrow().attr(b.node, &name).is_some();
    let want = force.unwrap_or(!present);
    if want && !present {
        set_attribute_notify(b, &name, "", context)?;
    } else if !want && present {
        remove_attribute_notify(b, &name, context)?;
    }
    Ok(want.into())
}

/// Set an attribute and queue attribute mutation records.
pub fn set_attribute_notify(
    b: &DomBinding,
    name: &str,
    value: &str,
    context: &mut Context,
) -> JsResult<()> {
    let old = b.tree.borrow().attr(b.node, name).map(str::to_string);
    b.tree
        .borrow_mut()
        .set_attr(b.node, name, value)
        .map_err(tree_error)?;
    crate::dom::mutation_bridge::attribute_changed(b, name, old, Some(value.to_string()), context)
}

/// Remove an attribute and queue attribute mutation records.
pub fn remove_attribute_notify(b: &DomBinding, name: &str, context: &mut Context) -> JsResult<()> {
    let old = b.tree.borrow().attr(b.node, name).map(str::to_string);
    if old.is_some() {
        b.tree
            .borrow_mut()
            .remove_attr(b.node, name)
            .map_err(tree_error)?;
        crate::dom::mutation_bridge::attribute_changed(b, name, old, None, context)?;
    }
    Ok(())
}

pub fn set_attribute(b: &DomBinding, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let name = attr_name(
        b,
        &args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped(),
    );
    let value = args
        .get_or_undefined(1)
        .to_string(context)?
        .to_std_string_escaped();
    set_attribute_notify(b, &name, &value, context)?;
    Ok(JsValue::undefined())
}

pub fn remove_attribute(
    b: &DomBinding,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let name = attr_name(
        b,
        &args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped(),
    );
    remove_attribute_notify(b, &name, context)?;
    Ok(JsValue::undefined())
}

pub fn set_inner_html(
    b: &DomBinding,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let html = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    let removed = b.tree.borrow().children(b.node).to_vec();
    let added = b
        .tree
        .borrow_mut()
        .set_inner_html(b.node, &html)
        .map_err(tree_error)?;
    // Scripts inserted via innerHTML never run (per spec), not even if moved later
    crate::dom::script_runner::mark_already_started(&b.tree, &added);
    crate::dom::mutation_bridge::record_child_list(b, b.node, &added, &removed, context)?;
    Ok(JsValue::undefined())
}

pub fn set_text_content(
    b: &DomBinding,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0);
    let text = if value.is_null() {
        String::new()
    } else {
        value.to_string(context)?.to_std_string_escaped()
    };
    let removed = b.tree.borrow().children(b.node).to_vec();
    b.tree
        .borrow_mut()
        .set_text_content(b.node, &text)
        .map_err(tree_error)?;
    let added = b.tree.borrow().children(b.node).to_vec();
    crate::dom::mutation_bridge::record_child_list(b, b.node, &added, &removed, context)?;
    Ok(JsValue::undefined())
}

pub fn get_outer_html(b: &DomBinding) -> JsValue {
    JsString::from(b.tree.borrow().outer_html(b.node)).into()
}

pub fn set_outer_html(
    b: &DomBinding,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let html = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    let parent = b.tree.borrow().parent(b.node);
    let Some(parent) = parent else {
        return Err(JsNativeError::typ()
            .with_message("NoModificationAllowedError: element has no parent")
            .into());
    };
    let nodes = b.tree.borrow_mut().parse_fragment(&html);
    {
        let mut tree = b.tree.borrow_mut();
        for &node in &nodes {
            tree.insert_before(parent, node, Some(b.node))
                .map_err(tree_error)?;
        }
        tree.remove(b.node).map_err(tree_error)?;
    }
    crate::dom::script_runner::mark_already_started(&b.tree, &nodes);
    crate::dom::mutation_bridge::record_child_list(b, parent, &nodes, &[b.node], context)?;
    Ok(JsValue::undefined())
}

// ---------------------------------------------------------------------------
// Document natives (tree path)
// ---------------------------------------------------------------------------

fn document_binding(document: &JsObject, tree: &SharedTree) -> DomBinding {
    let node = tree.borrow().document();
    DomBinding {
        document: document.clone(),
        tree: tree.clone(),
        node,
    }
}

/// Binding for the document node of a document with a tree.
pub fn document_root(this: &JsValue) -> Option<DomBinding> {
    let (document, tree) = document_tree(this)?;
    Some(document_binding(&document, &tree))
}

pub fn document_element(b: &DomBinding, context: &mut Context) -> JsResult<JsValue> {
    let node = b.tree.borrow().document_element();
    wrap(b, node, context)
}

pub fn document_body(b: &DomBinding, context: &mut Context) -> JsResult<JsValue> {
    let node = b.tree.borrow().html_child("body");
    wrap(b, node, context)
}

pub fn document_head(b: &DomBinding, context: &mut Context) -> JsResult<JsValue> {
    let node = b.tree.borrow().html_child("head");
    wrap(b, node, context)
}

pub fn get_element_by_id(
    b: &DomBinding,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let id = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    let node = b.tree.borrow().find_by_id(&id);
    wrap(b, node, context)
}

/// `document.forms` / `images` / `links` / `scripts`.
pub fn document_collection(
    b: &DomBinding,
    selector: &str,
    context: &mut Context,
) -> JsResult<JsValue> {
    let nodes = b
        .tree
        .borrow()
        .select(b.node, selector)
        .map_err(select_error)?;
    let values = wrap_all(b, nodes, context)?;
    Ok(Array::create_array_from_list(values, context).into())
}

/// `document.createElement` for a document with a tree: the new element is
/// a detached tree node from the start.
pub fn bind_new_element(document: &JsObject, tree: &SharedTree, element: &JsValue, tag: &str) {
    let Some(obj) = element.as_object() else {
        return;
    };
    let node = tree.borrow_mut().create_element(&tag.to_ascii_lowercase());
    let b = document_binding(document, tree);
    bind_object(&b, &obj, node);
}

/// `document.createTextNode` for a document with a tree.
pub fn bind_new_text(document: &JsObject, tree: &SharedTree, text: &JsValue) {
    let Some(obj) = text.as_object() else {
        return;
    };
    let data = obj
        .downcast_ref::<TextData>()
        .map(|t| t.character_data().get_data());
    if let Some(data) = data {
        let node = tree.borrow_mut().create_text(&data);
        let b = document_binding(document, tree);
        bind_object(&b, &obj, node);
    }
}

/// `document.write` while loading: parse and append to `<body>`.
pub fn document_write(b: &DomBinding, html: &str, context: &mut Context) -> JsResult<()> {
    let target = {
        let tree = b.tree.borrow();
        tree.html_child("body").or_else(|| tree.document_element())
    };
    let Some(target) = target else {
        return Ok(());
    };
    let nodes = b.tree.borrow_mut().parse_fragment(html);
    insert_nodes(b, target, &nodes, None, context)
}
