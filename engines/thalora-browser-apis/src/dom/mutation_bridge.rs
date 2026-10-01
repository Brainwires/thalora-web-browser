//! Side effects of tree mutations made through DOM APIs: MutationObserver
//! records, inserted-script execution and custom element reactions.

use boa_engine::{Context, JsResult};

use crate::dom::binding::DomBinding;
use crate::dom::tree::NodeId;

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
    _b: &DomBinding,
    _parent: NodeId,
    _added: &[NodeId],
    _removed: &[NodeId],
    _context: &mut Context,
) -> JsResult<()> {
    Ok(())
}

/// An attribute of `b.node` changed.
pub fn attribute_changed(
    _b: &DomBinding,
    _name: &str,
    _old: Option<String>,
    _new: Option<String>,
    _context: &mut Context,
) -> JsResult<()> {
    Ok(())
}
