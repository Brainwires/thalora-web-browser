//! MutationObserver records for mutations of tree-backed documents.

use boa_engine::{Context, Source};

use crate::dom::document::DocumentData;

const PAGE: &str = r#"<!DOCTYPE html><html><head><title>t</title></head>
<body><div id="main" class="box"><p id="p1">One</p><p class="x">Two</p></div>
<ul id="list"><li>a</li><li>b</li></ul></body></html>"#;

/// A context whose global `doc` is a Document loaded with `PAGE`.
fn context_with_page() -> Context {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("browser APIs");
    let doc = context
        .eval(Source::from_bytes("globalThis.doc = new Document(); doc"))
        .unwrap();
    let doc = doc.as_object().unwrap();
    doc.downcast_ref::<DocumentData>()
        .unwrap()
        .set_html_content(PAGE);
    context
}

fn eval(context: &mut Context, code: &str) {
    if let Err(e) = context.eval(Source::from_bytes(code)) {
        panic!("{code}\n=> {e}");
    }
}

fn eval_bool(context: &mut Context, code: &str) -> bool {
    match context.eval(Source::from_bytes(code)) {
        Ok(v) => v.to_boolean(),
        Err(e) => panic!("{code}\n=> {e}"),
    }
}

fn eval_string(context: &mut Context, code: &str) -> String {
    match context.eval(Source::from_bytes(code)) {
        Ok(v) => v.to_string(context).unwrap().to_std_string_escaped(),
        Err(e) => panic!("{code}\n=> {e}"),
    }
}

fn run_jobs(context: &mut Context) {
    context.run_jobs().expect("jobs");
}

/// Defines `makeObserver(name)`: an observer whose deliveries are logged in
/// `logs[name]` (records) and `calls[name]` (callback invocations).
const OBSERVER_HELPERS: &str = r#"
    globalThis.logs = {};
    globalThis.calls = {};
    globalThis.makeObserver = function (name) {
        logs[name] = [];
        calls[name] = 0;
        const mo = new MutationObserver(function (records, observer) {
            calls[name] += 1;
            globalThis['self_' + name] = observer === mo && this === mo;
            for (const r of records) logs[name].push(r);
        });
        return mo;
    };
"#;

#[test]
fn child_list_records_after_append_child() {
    let mut ctx = context_with_page();
    eval(&mut ctx, OBSERVER_HELPERS);
    eval(
        &mut ctx,
        r#"
        globalThis.main = doc.getElementById('main');
        globalThis.mo = makeObserver('a');
        mo.observe(main, { childList: true });
        globalThis.span = doc.createElement('span');
        main.appendChild(span);
        globalThis.em = doc.createElement('em');
        main.appendChild(em);
        "#,
    );
    // Delivery is asynchronous (microtask)
    assert!(eval_bool(&mut ctx, "calls.a === 0"));
    run_jobs(&mut ctx);

    // Both mutations arrive in one batch
    assert!(eval_bool(&mut ctx, "calls.a === 1"));
    assert!(eval_bool(&mut ctx, "self_a"));
    assert_eq!(eval_string(&mut ctx, "logs.a.length"), "2");
    let r = "logs.a[0]";
    assert_eq!(eval_string(&mut ctx, &format!("{r}.type")), "childList");
    assert!(eval_bool(&mut ctx, &format!("{r}.target === main")));
    assert!(eval_bool(&mut ctx, &format!("{r}.addedNodes.length === 1")));
    assert!(eval_bool(&mut ctx, &format!("{r}.addedNodes[0] === span")));
    assert!(eval_bool(
        &mut ctx,
        &format!("{r}.removedNodes.length === 0")
    ));
    assert!(eval_bool(
        &mut ctx,
        &format!("{r}.previousSibling === doc.querySelector('.x')")
    ));
    assert!(eval_bool(&mut ctx, &format!("{r}.nextSibling === null")));
    assert!(eval_bool(&mut ctx, "logs.a[1].addedNodes[0] === em"));
    assert!(eval_bool(&mut ctx, "logs.a[1].previousSibling === span"));

    // Removal
    eval(&mut ctx, "main.removeChild(span);");
    run_jobs(&mut ctx);
    assert!(eval_bool(&mut ctx, "calls.a === 2"));
    assert!(eval_bool(&mut ctx, "logs.a[2].removedNodes[0] === span"));
    assert!(eval_bool(&mut ctx, "logs.a[2].addedNodes.length === 0"));

    // Nothing pending: no extra callback
    run_jobs(&mut ctx);
    assert!(eval_bool(&mut ctx, "calls.a === 2"));
}

#[test]
fn attribute_records_with_old_value() {
    let mut ctx = context_with_page();
    eval(&mut ctx, OBSERVER_HELPERS);
    eval(
        &mut ctx,
        r#"
        globalThis.p1 = doc.getElementById('p1');
        makeObserver('old').observe(p1, { attributes: true, attributeOldValue: true });
        makeObserver('plain').observe(p1, { attributes: true });
        makeObserver('filtered').observe(p1, { attributeFilter: ['title'] });
        makeObserver('children').observe(p1, { childList: true });
        p1.setAttribute('data-x', '1');
        p1.setAttribute('data-x', '2');
        p1.removeAttribute('data-x');
        "#,
    );
    run_jobs(&mut ctx);

    assert_eq!(eval_string(&mut ctx, "logs.old.length"), "3");
    assert_eq!(eval_string(&mut ctx, "logs.old[0].type"), "attributes");
    assert_eq!(eval_string(&mut ctx, "logs.old[0].attributeName"), "data-x");
    assert!(eval_bool(&mut ctx, "logs.old[0].target === p1"));
    assert!(eval_bool(&mut ctx, "logs.old[0].oldValue === null"));
    assert_eq!(eval_string(&mut ctx, "logs.old[1].oldValue"), "1");
    assert_eq!(eval_string(&mut ctx, "logs.old[2].oldValue"), "2");

    // Without attributeOldValue the old value is not reported
    assert_eq!(eval_string(&mut ctx, "logs.plain.length"), "3");
    assert!(eval_bool(
        &mut ctx,
        "logs.plain.every(r => r.oldValue === null)"
    ));

    // attributeFilter / childList-only observers see nothing
    assert!(eval_bool(&mut ctx, "calls.filtered === 0"));
    assert!(eval_bool(&mut ctx, "calls.children === 0"));

    eval(&mut ctx, "p1.setAttribute('title', 'hi');");
    run_jobs(&mut ctx);
    assert_eq!(eval_string(&mut ctx, "logs.filtered.length"), "1");
    assert_eq!(
        eval_string(&mut ctx, "logs.filtered[0].attributeName"),
        "title"
    );
}

#[test]
fn subtree_option_controls_descendant_records() {
    let mut ctx = context_with_page();
    eval(&mut ctx, OBSERVER_HELPERS);
    eval(
        &mut ctx,
        r#"
        globalThis.main = doc.getElementById('main');
        globalThis.p1 = doc.getElementById('p1');
        makeObserver('direct').observe(main, { childList: true });
        makeObserver('deep').observe(main, { childList: true, subtree: true });
        makeObserver('bodyAttrs').observe(doc.body, { attributes: true, subtree: true });
        makeObserver('list').observe(doc.getElementById('list'), {
            childList: true, attributes: true, subtree: true
        });
        makeObserver('document').observe(doc, { childList: true, subtree: true });
        globalThis.b = doc.createElement('b');
        p1.appendChild(b);
        p1.setAttribute('title', 't');
        "#,
    );
    run_jobs(&mut ctx);

    // childList on a grandchild: only subtree observers see it
    assert!(eval_bool(&mut ctx, "calls.direct === 0"));
    assert_eq!(eval_string(&mut ctx, "logs.deep.length"), "1");
    assert!(eval_bool(&mut ctx, "logs.deep[0].target === p1"));
    assert!(eval_bool(&mut ctx, "logs.deep[0].addedNodes[0] === b"));
    assert_eq!(eval_string(&mut ctx, "logs.document.length"), "1");

    // Attribute change below body
    assert_eq!(eval_string(&mut ctx, "logs.bodyAttrs.length"), "1");
    assert!(eval_bool(&mut ctx, "logs.bodyAttrs[0].target === p1"));

    // Unrelated subtree sees nothing
    assert!(eval_bool(&mut ctx, "calls.list === 0"));

    // A direct child change reaches the non-subtree observer
    eval(&mut ctx, "main.appendChild(doc.createElement('i'));");
    run_jobs(&mut ctx);
    assert!(eval_bool(&mut ctx, "calls.direct === 1"));
}

#[test]
fn disconnect_and_take_records_stop_delivery() {
    let mut ctx = context_with_page();
    eval(&mut ctx, OBSERVER_HELPERS);
    eval(
        &mut ctx,
        r#"
        globalThis.main = doc.getElementById('main');
        globalThis.mo = makeObserver('a');
        mo.observe(main, { childList: true });
        main.appendChild(doc.createElement('span'));
        mo.disconnect();
        "#,
    );
    run_jobs(&mut ctx);
    assert!(eval_bool(&mut ctx, "calls.a === 0"));

    // Still disconnected: later mutations aren't recorded either
    eval(&mut ctx, "main.appendChild(doc.createElement('span'));");
    run_jobs(&mut ctx);
    assert!(eval_bool(&mut ctx, "calls.a === 0"));
    assert!(eval_bool(&mut ctx, "mo.takeRecords().length === 0"));

    // takeRecords returns and clears the pending records
    eval(
        &mut ctx,
        r#"
        mo.observe(main, { childList: true });
        main.appendChild(doc.createElement('span'));
        globalThis.taken = mo.takeRecords();
        "#,
    );
    assert!(eval_bool(&mut ctx, "taken.length === 1"));
    assert_eq!(eval_string(&mut ctx, "taken[0].type"), "childList");
    assert!(eval_bool(&mut ctx, "mo.takeRecords().length === 0"));
    run_jobs(&mut ctx);
    assert!(eval_bool(&mut ctx, "calls.a === 0"));
}
