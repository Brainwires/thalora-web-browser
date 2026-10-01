//! Side effects of DOM insertions that run script: inserted `<script>`
//! elements and custom element connected/disconnected reactions.
//!
//! Called from `mutation_bridge::child_list_changed`, i.e. only for the
//! insertion APIs that run scripts (appendChild, insertBefore, append, ...).
//! innerHTML / outerHTML / textContent go through `record_child_list` and
//! never get here.
//!
//! Scripts follow the HTML "prepare the script element" steps for inline
//! classic scripts: once a connected script with non-empty source text is
//! prepared it is marked "already started" and its text is evaluated
//! synchronously, as part of the insertion. No tree borrow is held while JS
//! runs, and script errors are reported to stderr instead of being thrown at
//! the inserting caller.
//!
//! Limits:
//! - `src` scripts are not fetched (the crate has no script loader); they are
//!   marked started and get an `error` event, like a failed fetch.
//! - `type="module"` scripts and `nomodule` scripts are skipped.
//! - Inline scripts get no `load` event (per spec, only external ones do).
//! - Scripts that were in a tree before its first DOM-API insertion
//!   (parser-inserted ones) are treated as already started. Scripts created by
//!   innerHTML/outerHTML are only treated so if that path calls
//!   [`mark_already_started`]; otherwise moving one with appendChild runs it.
//! - Adding text to an already connected, empty script does not run it (the
//!   "children changed" steps are not implemented).
//! - `document.currentScript` is not set.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::{Rc, Weak};

use boa_engine::{Context, JsResult, JsValue, Source, js_string, object::JsObject};

use crate::dom::binding::{self, DomBinding};
use crate::dom::tree::{DomTree, NodeId, NodeKind, SharedTree};
use crate::web_components::custom_element_registry as ce;

/// Per-tree set of script nodes whose "already started" flag is set.
struct StartedScripts {
    tree: Weak<RefCell<DomTree>>,
    started: HashSet<NodeId>,
}

thread_local! {
    static STARTED: RefCell<Vec<StartedScripts>> = const { RefCell::new(Vec::new()) };
}

/// Run `f` on the started set of `tree`. `seed` provides the initial set the
/// first time the tree is seen. Entries of dropped trees are pruned.
fn with_started<R>(
    tree: &SharedTree,
    seed: impl FnOnce() -> HashSet<NodeId>,
    f: impl FnOnce(&mut HashSet<NodeId>) -> R,
) -> R {
    STARTED.with(|cell| {
        let mut all = cell.borrow_mut();
        all.retain(|entry| entry.tree.strong_count() > 0);
        let index = match all
            .iter()
            .position(|entry| entry.tree.upgrade().is_some_and(|t| Rc::ptr_eq(&t, tree)))
        {
            Some(index) => index,
            None => {
                all.push(StartedScripts {
                    tree: Rc::downgrade(tree),
                    started: seed(),
                });
                all.len() - 1
            }
        };
        f(&mut all[index].started)
    })
}

/// Set the "already started" flag on every script in the subtrees of
/// `roots` (for parser-like insertion such as innerHTML, whose scripts must
/// never run, even when moved later).
pub fn mark_already_started(tree: &SharedTree, roots: &[NodeId]) {
    let scripts = {
        let t = tree.borrow();
        scripts_in(&t, roots)
    };
    if scripts.is_empty() {
        return;
    }
    let seed = connected_scripts_seed(tree, &[]);
    with_started(tree, || seed, |started| started.extend(scripts));
}

/// Hook for `child_list_changed`: custom element reactions for removed and
/// inserted subtrees, and inserted scripts.
pub fn after_child_list_change(
    b: &DomBinding,
    parent: NodeId,
    added: &[NodeId],
    removed: &[NodeId],
    context: &mut Context,
) -> JsResult<()> {
    let parent_connected = b.tree.borrow().is_connected(parent);
    if !parent_connected {
        return Ok(());
    }
    // Removed subtrees were connected (their old parent still is).
    if !removed.is_empty() {
        ce::disconnected_reactions(&b.document, &b.tree, removed, context)?;
    }
    if added.is_empty() {
        return Ok(());
    }
    // Custom elements to connect are found before scripts can move things.
    let custom = ce::custom_element_candidates(&b.tree, added);
    run_inserted_scripts(b, added, context)?;
    ce::connected_reactions(&b.document, &b.tree, &custom, context)
}

/// Prepare and run the scripts in the inserted subtrees, in tree order.
fn run_inserted_scripts(b: &DomBinding, added: &[NodeId], context: &mut Context) -> JsResult<()> {
    let candidates = {
        let tree = b.tree.borrow();
        scripts_in(&tree, added)
    };
    if candidates.is_empty() {
        return Ok(());
    }
    // First contact with this tree: scripts already in the document were
    // inserted by the parser and count as started.
    let seed = connected_scripts_seed(&b.tree, added);
    with_started(&b.tree, || seed, |_| ());

    for script in candidates {
        let action = {
            let tree = b.tree.borrow();
            prepare(&tree, script)
        };
        let action = match action {
            Some(action) => action,
            None => continue,
        };
        let newly_started = with_started(&b.tree, HashSet::new, |started| started.insert(script));
        if !newly_started {
            continue;
        }
        // All borrows released: run JS.
        match action {
            ScriptAction::Inline(text) => {
                if let Err(err) = context.eval(Source::from_bytes(text.as_bytes())) {
                    report_error(&err.to_string());
                }
                // Jobs queued by the script run at the caller's next checkpoint.
            }
            ScriptAction::External => {
                eprintln!(
                    "console.warn: <script src> inserted via DOM is not fetched; firing error event"
                );
                fire_event(b, script, "error", context);
            }
        }
    }
    Ok(())
}

enum ScriptAction {
    Inline(String),
    External,
}

/// "Prepare the script element" for a candidate: `None` when it must not
/// run (already started is checked by the caller).
fn prepare(tree: &DomTree, script: NodeId) -> Option<ScriptAction> {
    if !tree.is_connected(script) {
        return None;
    }
    if !is_classic_javascript(tree.attr(script, "type"), tree.attr(script, "language")) {
        return None;
    }
    if tree.attr(script, "nomodule").is_some() {
        return None;
    }
    if tree.attr(script, "src").is_some() {
        return Some(ScriptAction::External);
    }
    let text = child_text_content(tree, script);
    if text.is_empty() {
        // Spec: returns without setting "already started".
        return None;
    }
    Some(ScriptAction::Inline(text))
}

/// Whether the type/language attributes select a classic JavaScript script.
fn is_classic_javascript(type_attr: Option<&str>, language: Option<&str>) -> bool {
    let type_string = match (type_attr, language) {
        (Some(t), _) => t.trim().to_ascii_lowercase(),
        (None, Some(l)) if !l.is_empty() => format!("text/{}", l.trim().to_ascii_lowercase()),
        (None, _) => String::new(),
    };
    if type_string.is_empty() {
        return true;
    }
    matches!(
        type_string.as_str(),
        "application/ecmascript"
            | "application/javascript"
            | "application/x-ecmascript"
            | "application/x-javascript"
            | "text/ecmascript"
            | "text/javascript"
            | "text/javascript1.0"
            | "text/javascript1.1"
            | "text/javascript1.2"
            | "text/javascript1.3"
            | "text/javascript1.4"
            | "text/javascript1.5"
            | "text/jscript"
            | "text/livescript"
            | "text/x-ecmascript"
            | "text/x-javascript"
    )
}

/// The script's source text: its Text children only, concatenated.
fn child_text_content(tree: &DomTree, node: NodeId) -> String {
    let mut out = String::new();
    for &child in tree.children(node) {
        if let Some(NodeKind::Text(data)) = tree.kind(child) {
            out.push_str(data);
        }
    }
    out
}

/// `<script>` elements in the subtrees of `roots`, in tree order, deduped.
fn scripts_in(tree: &DomTree, roots: &[NodeId]) -> Vec<NodeId> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for &root in roots {
        for node in tree.descendants(root) {
            if tree
                .tag(node)
                .is_some_and(|t| t.eq_ignore_ascii_case("script"))
                && seen.insert(node)
            {
                out.push(node);
            }
        }
    }
    out
}

/// Scripts connected now, minus those in the `added` subtrees.
fn connected_scripts_seed(tree: &SharedTree, added: &[NodeId]) -> HashSet<NodeId> {
    let t = tree.borrow();
    let document = t.document();
    let added_scripts: HashSet<NodeId> = scripts_in(&t, added).into_iter().collect();
    scripts_in(&t, &[document])
        .into_iter()
        .filter(|node| !added_scripts.contains(node))
        .collect()
}

/// Dispatch a plain `Event(type)` on the node's wrapper through its
/// JS-visible `dispatchEvent`. Failures are reported, not propagated.
fn fire_event(b: &DomBinding, node: NodeId, event_type: &str, context: &mut Context) {
    let result = (|| -> JsResult<()> {
        let Some(target) = binding::wrapper_for(&b.document, &b.tree, node, context)?.as_object()
        else {
            return Ok(());
        };
        let Some(event_ctor) = context
            .global_object()
            .get(js_string!("Event"), context)?
            .as_object()
        else {
            return Ok(());
        };
        let event: JsObject =
            event_ctor.construct(&[js_string!(event_type).into()], None, context)?;
        let dispatch = target.get(js_string!("dispatchEvent"), context)?;
        if let Some(dispatch) = dispatch.as_callable() {
            dispatch.call(&JsValue::from(target.clone()), &[event.into()], context)?;
        }
        Ok(())
    })();
    if let Err(err) = result {
        report_error(&err.to_string());
    }
}

/// Report an uncaught exception the way a console would.
pub(crate) fn report_error(message: &str) {
    eprintln!("console.error: Uncaught {message}");
}

#[cfg(test)]
mod unit_tests {
    use super::is_classic_javascript;

    #[test]
    fn classic_javascript_types() {
        assert!(is_classic_javascript(None, None));
        assert!(is_classic_javascript(Some(""), None));
        assert!(is_classic_javascript(Some(" Text/JavaScript "), None));
        assert!(is_classic_javascript(None, Some("javascript")));
        assert!(is_classic_javascript(None, Some("")));
        assert!(!is_classic_javascript(Some("module"), None));
        assert!(!is_classic_javascript(Some("text/template"), None));
        assert!(!is_classic_javascript(Some("application/json"), None));
        assert!(!is_classic_javascript(None, Some("vbscript")));
    }
}
