//! Inserted-script execution and custom element reactions.

use boa_engine::{Context, Source};

use crate::dom::document::DocumentData;

const PAGE: &str = r#"<!DOCTYPE html><html><head><title>t</title></head>
<body><div id="main"><p id="p1">One</p></div></body></html>"#;

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

fn eval_string(context: &mut Context, code: &str) -> String {
    match context.eval(Source::from_bytes(code)) {
        Ok(v) => v.to_string(context).unwrap().to_std_string_escaped(),
        Err(e) => panic!("{code}\n=> {e}"),
    }
}

#[test]
fn appended_script_runs_once() {
    let mut ctx = context_with_page();
    eval(
        &mut ctx,
        r#"
        globalThis.s = doc.createElement('script');
        s.textContent = 'globalThis.ran = (globalThis.ran||0)+1';
        doc.body.appendChild(s);
        "#,
    );
    assert_eq!(eval_string(&mut ctx, "String(globalThis.ran)"), "1");

    // Moving it again does not re-run it
    eval(&mut ctx, "doc.getElementById('main').appendChild(s)");
    eval(&mut ctx, "s.remove(); doc.body.appendChild(s)");
    assert_eq!(eval_string(&mut ctx, "String(globalThis.ran)"), "1");
}

#[test]
fn inner_html_scripts_do_not_run() {
    let mut ctx = context_with_page();
    eval(
        &mut ctx,
        r#"doc.getElementById('main').innerHTML = '<script>globalThis.inner = 1</script>';"#,
    );
    assert_eq!(
        eval_string(&mut ctx, "typeof globalThis.inner"),
        "undefined"
    );
}

#[test]
fn script_in_detached_subtree_runs_when_connected() {
    let mut ctx = context_with_page();
    eval(
        &mut ctx,
        r#"
        globalThis.div = doc.createElement('div');
        const s = doc.createElement('script');
        s.textContent = 'globalThis.later = (globalThis.later||0)+1';
        div.appendChild(s);
        "#,
    );
    assert_eq!(
        eval_string(&mut ctx, "typeof globalThis.later"),
        "undefined"
    );
    eval(&mut ctx, "doc.body.appendChild(div)");
    assert_eq!(eval_string(&mut ctx, "String(globalThis.later)"), "1");
}

#[test]
fn scripts_run_in_tree_order_and_skip_non_javascript() {
    let mut ctx = context_with_page();
    eval(
        &mut ctx,
        r#"
        globalThis.order = [];
        const div = doc.createElement('div');
        for (const [type, n] of [[null, 'a'], ['text/template', 'b'], ['module', 'c'], ['text/javascript', 'd']]) {
            const s = doc.createElement('script');
            if (type) s.setAttribute('type', type);
            s.textContent = 'order.push(' + JSON.stringify(n) + ')';
            div.appendChild(s);
        }
        doc.body.appendChild(div);
        "#,
    );
    assert_eq!(eval_string(&mut ctx, "order.join(',')"), "a,d");
}

#[test]
fn script_errors_do_not_reach_the_inserting_caller() {
    let mut ctx = context_with_page();
    eval(
        &mut ctx,
        r#"
        const bad = doc.createElement('script');
        bad.textContent = 'throw new Error("boom")';
        doc.body.appendChild(bad);
        globalThis.after = true;
        "#,
    );
    assert_eq!(eval_string(&mut ctx, "String(globalThis.after)"), "true");
}

#[test]
fn external_scripts_are_not_run_and_fire_error() {
    let mut ctx = context_with_page();
    eval(
        &mut ctx,
        r#"
        globalThis.events = [];
        const s = doc.createElement('script');
        s.setAttribute('src', 'https://example.invalid/x.js');
        s.textContent = 'globalThis.inlineOfExternal = 1';
        s.addEventListener('error', () => events.push('error'));
        s.addEventListener('load', () => events.push('load'));
        doc.body.appendChild(s);
        "#,
    );
    assert_eq!(
        eval_string(&mut ctx, "typeof globalThis.inlineOfExternal"),
        "undefined"
    );
    assert_eq!(eval_string(&mut ctx, "events.join(',')"), "error");
}

#[test]
fn custom_element_reactions() {
    let mut ctx = context_with_page();
    eval(
        &mut ctx,
        r#"
        globalThis.log = [];
        class R8Element extends HTMLElement {
            static get observedAttributes() { return ['watched']; }
            connectedCallback() { log.push('connected:' + (this instanceof R8Element)); }
            disconnectedCallback() { log.push('disconnected'); }
            attributeChangedCallback(name, oldValue, newValue) {
                log.push('attr:' + name + ':' + oldValue + ':' + newValue);
            }
        }
        customElements.define('r8-reaction-el', R8Element);

        globalThis.el = doc.createElement('r8-reaction-el');
        const wrapper = doc.createElement('div');
        wrapper.appendChild(el);          // detached: no connectedCallback
        doc.body.appendChild(wrapper);    // connected through an ancestor
        el.setAttribute('watched', 'a');
        el.setAttribute('watched', 'b');
        el.setAttribute('ignored', 'x');  // not observed
        el.removeAttribute('watched');
        wrapper.remove();
        "#,
    );
    assert_eq!(
        eval_string(&mut ctx, "log.join('|')"),
        "connected:true|attr:watched:null:a|attr:watched:a:b|attr:watched:b:null|disconnected"
    );
}
