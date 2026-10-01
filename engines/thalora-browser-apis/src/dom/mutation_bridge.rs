//! Side effects of tree mutations made through DOM APIs: MutationObserver
//! records, inserted-script execution and custom element reactions.
//!
//! MutationObserver integration follows the spec's "queue a mutation
//! record": find the interested observers (an observed target that is the
//! mutated node, or an inclusive ancestor of it when `subtree` is set, and
//! whose options match), build one record per observer and queue it. The
//! records are delivered from a single microtask per batch
//! (see `observers::mutation_observer::notify_mutation_observers`).
//!
//! No tree or observer borrow is held while wrappers are created or JS runs.

use std::rc::Rc;

use boa_engine::{Context, JsResult, object::JsObject};

use crate::dom::binding::{self, DomBinding};
use crate::dom::document::DocumentData;
use crate::dom::tree::{NodeId, SharedTree};
use crate::observers::mutation_observer::{self as mo, MutationObserverConfig, MutationRecordData};

/// Nodes were inserted under / removed from `parent` by a DOM API that
/// runs inserted scripts (appendChild, insertBefore, append, ...).
pub fn child_list_changed(
    b: &DomBinding,
    parent: NodeId,
    added: &[NodeId],
    removed: &[NodeId],
    context: &mut Context,
) -> JsResult<()> {
    record_child_list(b, parent, added, removed, context)
}

/// Child-list change that must not run scripts (innerHTML, outerHTML,
/// textContent).
pub fn record_child_list(
    b: &DomBinding,
    parent: NodeId,
    added: &[NodeId],
    removed: &[NodeId],
    context: &mut Context,
) -> JsResult<()> {
    if added.is_empty() && removed.is_empty() {
        return Ok(());
    }
    let interested =
        interested_observers(&b.tree, parent, |config| config.child_list.then_some(false));
    if interested.is_empty() {
        return Ok(());
    }

    // Siblings around the inserted run (cheap when nodes were added; a
    // pure removal no longer knows where the nodes were).
    let (previous, next) = {
        let tree = b.tree.borrow();
        match (added.first(), added.last()) {
            (Some(&first), Some(&last))
                if tree.parent(first) == Some(parent) && tree.parent(last) == Some(parent) =>
            {
                (tree.previous_sibling(first), tree.next_sibling(last))
            }
            _ => (None, None),
        }
    };

    let Some(target) = wrapper_object(b, Some(parent), context)? else {
        return Ok(());
    };
    let added_nodes = wrapper_objects(b, added, context)?;
    let removed_nodes = wrapper_objects(b, removed, context)?;
    let previous_sibling = wrapper_object(b, previous, context)?;
    let next_sibling = wrapper_object(b, next, context)?;

    for (observer, _) in interested {
        let record = MutationRecordData::child_list(
            target.clone(),
            added_nodes.clone(),
            removed_nodes.clone(),
            previous_sibling.clone(),
            next_sibling.clone(),
        );
        mo::queue_mutation_record(&observer, record, context);
    }
    Ok(())
}

/// An attribute of `b.node` changed.
pub fn attribute_changed(
    b: &DomBinding,
    name: &str,
    old: Option<String>,
    _new: Option<String>,
    context: &mut Context,
) -> JsResult<()> {
    let interested = interested_observers(&b.tree, b.node, |config| {
        config
            .should_observe_attribute(name)
            .then(|| config.attribute_old_value.unwrap_or(false))
    });
    if interested.is_empty() {
        return Ok(());
    }
    let Some(target) = wrapper_object(b, Some(b.node), context)? else {
        return Ok(());
    };
    for (observer, want_old) in interested {
        let record = MutationRecordData::attributes(
            target.clone(),
            name.to_string(),
            None,
            if want_old { old.clone() } else { None },
        );
        mo::queue_mutation_record(&observer, record, context);
    }
    Ok(())
}

/// The data of the text/comment node `b.node` changed from `old`.
///
/// Not wired up yet: `CharacterData::sync_to_tree` has no `Context`, so the
/// caller must pass one in once the CharacterData natives can provide it.
pub fn character_data_changed(
    b: &DomBinding,
    old: Option<String>,
    context: &mut Context,
) -> JsResult<()> {
    let interested = interested_observers(&b.tree, b.node, |config| {
        config
            .observes_character_data()
            .then(|| config.character_data_old_value.unwrap_or(false))
    });
    if interested.is_empty() {
        return Ok(());
    }
    let Some(target) = wrapper_object(b, Some(b.node), context)? else {
        return Ok(());
    };
    for (observer, want_old) in interested {
        let record = MutationRecordData::character_data(
            target.clone(),
            if want_old { old.clone() } else { None },
        );
        mo::queue_mutation_record(&observer, record, context);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Observers interested in a mutation of `node`, each with whether it wants
/// the old value. `matches` returns `Some(wants_old_value)` when an
/// observation's options select this kind of mutation.
///
/// Per spec an observer gets at most one record per mutation even when
/// several of its observations match; it wants the old value if any of them
/// asked for it.
fn interested_observers(
    tree: &SharedTree,
    node: NodeId,
    matches: impl Fn(&MutationObserverConfig) -> Option<bool>,
) -> Vec<(JsObject, bool)> {
    let mut out = Vec::new();
    for observer in mo::registered_observers() {
        let mut interested = false;
        let mut want_old = false;
        for (target, config) in mo::observations_of(&observer) {
            let Some((target_tree, target_node)) = target_node(&target) else {
                continue;
            };
            if !Rc::ptr_eq(&target_tree, tree) {
                continue;
            }
            let in_scope = target_node == node
                || (config.subtree && tree.borrow().is_inclusive_ancestor(target_node, node));
            if !in_scope {
                continue;
            }
            if let Some(old) = matches(&config) {
                interested = true;
                want_old |= old;
            }
        }
        if interested {
            out.push((observer, want_old));
        }
    }
    out
}

/// The tree node an observed JS object stands for (bound nodes, or a
/// document with a tree).
fn target_node(obj: &JsObject) -> Option<(SharedTree, NodeId)> {
    if let Some(b) = binding::binding_of(obj) {
        return Some((b.tree, b.node));
    }
    let tree = obj.downcast_ref::<DocumentData>()?.tree()?;
    let node = tree.borrow().document();
    Some((tree, node))
}

fn wrapper_object(
    b: &DomBinding,
    node: Option<NodeId>,
    context: &mut Context,
) -> JsResult<Option<JsObject>> {
    let Some(node) = node else {
        return Ok(None);
    };
    Ok(binding::wrapper_for(&b.document, &b.tree, node, context)?.as_object())
}

fn wrapper_objects(
    b: &DomBinding,
    nodes: &[NodeId],
    context: &mut Context,
) -> JsResult<Vec<JsObject>> {
    let mut out = Vec::with_capacity(nodes.len());
    for &node in nodes {
        if let Some(obj) = wrapper_object(b, Some(node), context)? {
            out.push(obj);
        }
    }
    Ok(out)
}
