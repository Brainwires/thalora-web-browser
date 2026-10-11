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
//! - `src` scripts are fetched asynchronously (SSRF-safe page client), run,
//!   and get `load` (or `error`); `async`/`defer` ordering is not modelled.
//! - `type="module"` scripts (inline or `src`) are parsed and evaluated as
//!   modules; imports use the context's module loader. `nomodule` classic
//!   scripts are skipped, as in module-capable browsers.
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
            ScriptAction::Inline { text, module } => {
                run_source(&text, module, context);
                // Jobs queued by the script run at the caller's next checkpoint.
            }
            ScriptAction::External { src, module } => {
                load_external(b, script, &src, module, context)
            }
        }
    }
    Ok(())
}

enum ScriptAction {
    Inline { text: String, module: bool },
    External { src: String, module: bool },
}

/// Run script source: classic scripts are evaluated, module scripts are
/// parsed, linked (imports go through the context's module loader) and
/// evaluated. Errors are reported, never thrown at the inserting caller.
fn run_source(text: &str, module: bool, context: &mut Context) {
    if !module {
        if let Err(err) = context.eval(Source::from_bytes(text.as_bytes())) {
            report_error(&err.to_string());
        }
        return;
    }
    let parsed = boa_engine::Module::parse(Source::from_bytes(text.as_bytes()), None, context);
    match parsed {
        Ok(module) => {
            let promise = module.load_link_evaluate(context);
            let on_rejected = boa_engine::NativeFunction::from_fn_ptr(|_, args, _context| {
                let reason = args
                    .first()
                    .map(|v| v.display().to_string())
                    .unwrap_or_default();
                report_error(&reason);
                Ok(JsValue::undefined())
            });
            let on_rejected = on_rejected.to_js_function(context.realm());
            let _ = promise.catch(on_rejected, context);
        }
        Err(err) => report_error(&err.to_string()),
    }
}

/// "Prepare the script element" for a candidate: `None` when it must not
/// run (already started is checked by the caller).
fn prepare(tree: &DomTree, script: NodeId) -> Option<ScriptAction> {
    if !tree.is_connected(script) {
        return None;
    }
    let module = tree
        .attr(script, "type")
        .is_some_and(|t| t.trim().eq_ignore_ascii_case("module"));
    if !module && !is_classic_javascript(tree.attr(script, "type"), tree.attr(script, "language")) {
        return None;
    }
    // Module-capable browsers skip `nomodule` classic scripts
    if !module && tree.attr(script, "nomodule").is_some() {
        return None;
    }
    if let Some(src) = tree.attr(script, "src") {
        return Some(ScriptAction::External {
            src: src.to_string(),
            module,
        });
    }
    let text = child_text_content(tree, script);
    if text.is_empty() {
        // Spec: returns without setting "already started".
        return None;
    }
    Some(ScriptAction::Inline { text, module })
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
/// Fetch an external script (async, as for any DOM-inserted script), run
/// it, then fire `load` on the element; `error` if it can't be fetched.
#[cfg(feature = "native")]
fn load_external(b: &DomBinding, script: NodeId, src: &str, module: bool, context: &mut Context) {
    use boa_engine::job::NativeAsyncJob;

    let Some(url) = crate::page_url::resolve_url(context, src) else {
        report_error(&format!("<script src=\"{src}\">: invalid URL"));
        fire_event(b, script, "error", context);
        return;
    };
    let url = url.to_string();
    let b = b.clone();
    context.enqueue_job(
        NativeAsyncJob::new(async move |context| {
            // Same SSRF-safe client and network runtime as fetch()
            let client = crate::net::page_client();
            let request_url = url.clone();
            let fetched = crate::net::io(async move {
                let response = client
                    .get(&request_url)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;
                if !response.status().is_success() {
                    return Err(format!("HTTP {}", response.status()));
                }
                response.text().await.map_err(|e| e.to_string())
            })
            .await;
            let context = &mut context.borrow_mut();
            match fetched {
                Ok(source) => {
                    run_source(&source, module, context);
                    fire_event(&b, script, "load", context);
                }
                Err(err) => {
                    report_error(&format!("failed to load script {url}: {err}"));
                    fire_event(&b, script, "error", context);
                }
            }
            Ok(JsValue::undefined())
        })
        .into(),
    );
}

#[cfg(not(feature = "native"))]
fn load_external(b: &DomBinding, script: NodeId, _src: &str, _module: bool, context: &mut Context) {
    fire_event(b, script, "error", context);
}

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
