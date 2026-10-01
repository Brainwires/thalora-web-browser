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

/// Serve `body` as JavaScript on a local port; returns the port.
fn serve_script(body: &'static str) -> u16 {
    use std::io::{Read, Write};
    // SAFETY: tests that touch THALORA_ALLOW_LOOPBACK only ever set it to "1"
    unsafe { std::env::set_var("THALORA_ALLOW_LOOPBACK", "1") };
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: text/javascript\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
        }
    });
    port
}

#[cfg(feature = "native")]
#[test]
fn external_scripts_are_fetched_run_and_fire_load() {
    let port = serve_script("globalThis.externalRan = (globalThis.externalRan || 0) + 1;");
    let mut ctx = context_with_page();
    eval(
        &mut ctx,
        &format!(
            r#"
        globalThis.events = [];
        const s = doc.createElement('script');
        s.setAttribute('src', 'http://127.0.0.1:{port}/x.js');
        s.addEventListener('load', () => events.push('load:' + globalThis.externalRan));
        s.addEventListener('error', () => events.push('error'));
        doc.body.appendChild(s);
        globalThis.ranSynchronously = typeof globalThis.externalRan;
        "#
        ),
    );
    // DOM-inserted external scripts are async: nothing ran during insertion
    assert_eq!(eval_string(&mut ctx, "ranSynchronously"), "undefined");
    ctx.run_jobs().expect("jobs");
    assert_eq!(eval_string(&mut ctx, "events.join(',')"), "load:1");
    // Moving it doesn't fetch or run it again
    eval(
        &mut ctx,
        "doc.getElementById('main').appendChild(doc.querySelector('script'));",
    );
    ctx.run_jobs().expect("jobs");
    assert_eq!(eval_string(&mut ctx, "String(globalThis.externalRan)"), "1");
}

#[cfg(feature = "native")]
#[test]
fn unreachable_external_scripts_fire_error() {
    // SAFETY: see serve_script
    unsafe { std::env::set_var("THALORA_ALLOW_LOOPBACK", "1") };
    // A port nothing listens on
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let mut ctx = context_with_page();
    eval(
        &mut ctx,
        &format!(
            r#"
        globalThis.events = [];
        const s = doc.createElement('script');
        s.setAttribute('src', 'http://127.0.0.1:{port}/missing.js');
        s.textContent = 'globalThis.inlineOfExternal = 1';
        s.addEventListener('error', () => events.push('error'));
        s.addEventListener('load', () => events.push('load'));
        doc.body.appendChild(s);
        "#
        ),
    );
    ctx.run_jobs().expect("jobs");
    assert_eq!(eval_string(&mut ctx, "events.join(',')"), "error");
    // The inline text of a src script never runs
    assert_eq!(
        eval_string(&mut ctx, "typeof globalThis.inlineOfExternal"),
        "undefined"
    );
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

/// Like `context_with_page`, but the page is loaded into the global
/// `document` (as for a real page) and `doc` aliases it.
fn context_with_global_page() -> Context {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("browser APIs");
    let doc = context
        .eval(Source::from_bytes("globalThis.doc = document; doc"))
        .unwrap();
    let doc = doc.as_object().unwrap();
    doc.downcast_ref::<DocumentData>()
        .unwrap()
        .set_html_content(PAGE);
    context
}

#[test]
fn define_upgrades_existing_elements_and_resolves_when_defined() {
    let mut ctx = context_with_global_page();
    eval(
        &mut ctx,
        r#"
        globalThis.log = [];
        doc.getElementById('main').innerHTML = '<late-el id="a"></late-el>';
        const detached = doc.createElement('late-el');
        customElements.whenDefined('late-el').then(c => log.push('defined:' + (c === LateEl)));
        class LateEl extends HTMLElement {
            connectedCallback() { log.push('connected:' + this.id); }
            hello() { return 'hi ' + this.id; }
        }
        globalThis.LateEl = LateEl;
        customElements.define('late-el', LateEl);
        globalThis.detached = detached;
        "#,
    );
    ctx.run_jobs().expect("jobs");
    // The connected element was upgraded and got connectedCallback
    assert_eq!(
        eval_string(&mut ctx, "doc.getElementById('a').hello()"),
        "hi a"
    );
    assert_eq!(
        eval_string(&mut ctx, "log.join('|')"),
        "connected:a|defined:true"
    );
    // Detached elements wait until upgrade() (no connectedCallback)
    assert_eq!(eval_string(&mut ctx, "typeof detached.hello"), "undefined");
    eval(&mut ctx, "customElements.upgrade(detached);");
    assert_eq!(eval_string(&mut ctx, "typeof detached.hello"), "function");
    assert_eq!(eval_string(&mut ctx, "log.length"), "2");
    assert_eq!(
        eval_string(&mut ctx, "customElements.getName(LateEl)"),
        "late-el"
    );
}

#[test]
fn each_realm_has_its_own_registry() {
    // A fresh page (new realm) can define a name another page already used
    for _ in 0..2 {
        let mut ctx = context_with_page();
        eval(
            &mut ctx,
            "customElements.define('per-realm-el', class extends HTMLElement {});",
        );
        assert_eq!(
            eval_string(&mut ctx, "typeof customElements.get('per-realm-el')"),
            "function"
        );
    }
}

#[test]
fn custom_element_constructors_run_on_every_creation_path() {
    let mut ctx = context_with_global_page();
    eval(
        &mut ctx,
        r#"
        globalThis.made = [];
        doc.getElementById('main').innerHTML = '<ctor-el id="parsed"></ctor-el>';
        class CtorEl extends HTMLElement {
            field = 'set';
            constructor() { super(); made.push(this.id || 'new'); this.count = 1; }
        }
        customElements.define('ctor-el', CtorEl);
        globalThis.created = doc.createElement('ctor-el');
        globalThis.direct = new CtorEl();
        doc.body.appendChild(direct);
        "#,
    );
    // Upgrade of the parsed element ran the constructor on that same object
    let parsed = "doc.getElementById('parsed')";
    assert_eq!(
        eval_string(&mut ctx, &format!("{parsed}.field + {parsed}.count")),
        "set1"
    );
    assert_eq!(eval_string(&mut ctx, "made.join(',')"), "parsed,new,new");
    // createElement and `new` give real, correctly named, tree-backed elements
    assert_eq!(
        eval_string(&mut ctx, "created.field + created.tagName"),
        "setCTOR-EL"
    );
    assert_eq!(eval_string(&mut ctx, "direct.tagName"), "CTOR-EL");
    assert!(
        eval_string(&mut ctx, "doc.body.innerHTML").contains("<ctor-el></ctor-el>"),
        "{}",
        eval_string(&mut ctx, "doc.body.innerHTML")
    );
    assert_eq!(eval_string(&mut ctx, "String(direct.isConnected)"), "true");
}

#[test]
fn module_scripts_inserted_via_dom_run_as_modules() {
    let mut ctx = context_with_page();
    eval(
        &mut ctx,
        r#"
        const m = doc.createElement('script');
        m.type = 'module';
        m.setAttribute('type', 'module');
        m.textContent = 'var moduleLocal = 1; export const x = 2; globalThis.moduleRan = "ran:" + typeof this;';
        doc.body.appendChild(m);

        const bad = doc.createElement('script');
        bad.setAttribute('type', 'module');
        bad.textContent = 'export export;';
        doc.body.appendChild(bad);   // syntax error: reported, not thrown

        const skipped = doc.createElement('script');
        skipped.setAttribute('nomodule', '');
        skipped.textContent = 'globalThis.nomoduleRan = true;';
        doc.body.appendChild(skipped);
        "#,
    );
    ctx.run_jobs().expect("jobs");
    // Module code: `this` is undefined and top-level vars stay module-scoped
    assert_eq!(eval_string(&mut ctx, "globalThis.moduleRan"), "ran:undefined");
    assert_eq!(
        eval_string(&mut ctx, "typeof globalThis.moduleLocal"),
        "undefined"
    );
    assert_eq!(
        eval_string(&mut ctx, "typeof globalThis.nomoduleRan"),
        "undefined"
    );
}
