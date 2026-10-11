//! Event propagation (capture / target / bubble) for tree-backed elements.

use boa_engine::{Context, Source};

use crate::dom::document::DocumentData;

const PAGE: &str = r#"<!DOCTYPE html><html><head><title>t</title></head>
<body><div id="main"><p id="p1">One</p></div></body></html>"#;

/// A context whose global `doc` is a Document loaded with `PAGE`, plus
/// globals `main` (the div), `p` (its child) and an empty `log` array.
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
    eval_string(
        &mut context,
        r#"
        globalThis.main = doc.getElementById('main');
        globalThis.p = doc.getElementById('p1');
        globalThis.log = [];
        "#,
    );
    context
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

#[test]
fn events_bubble_to_ancestors_and_document() {
    let mut ctx = context_with_page();
    assert!(eval_bool(
        &mut ctx,
        "p.parentNode === main && main !== null"
    ));
    let log = eval_string(
        &mut ctx,
        r#"
        p.addEventListener('ping', () => log.push('p'));
        main.addEventListener('ping', () => log.push('main'));
        doc.body.addEventListener('ping', () => log.push('body'));
        doc.addEventListener('ping', () => log.push('doc'));
        p.dispatchEvent(new Event('ping', { bubbles: true }));
        log.push('|');
        // Non-bubbling events only reach the target.
        p.dispatchEvent(new Event('ping'));
        log.join(',')
        "#,
    );
    assert_eq!(log, "p,main,body,doc,|,p");
}

#[test]
fn capture_listeners_run_before_target_and_bubble() {
    let mut ctx = context_with_page();
    let log = eval_string(
        &mut ctx,
        r#"
        const rec = (name) => (e) => log.push(name + ':' + e.eventPhase);
        main.addEventListener('t', rec('main-bubble'));
        main.addEventListener('t', rec('main-capture'), true);
        doc.addEventListener('t', rec('doc-capture'), { capture: true });
        doc.addEventListener('t', rec('doc-bubble'));
        p.addEventListener('t', rec('target'));
        p.dispatchEvent(new Event('t', { bubbles: true }));
        log.join(',')
        "#,
    );
    assert_eq!(
        log,
        "doc-capture:1,main-capture:1,target:2,main-bubble:3,doc-bubble:3"
    );
}

#[test]
fn remove_event_listener_matches_capture_flag() {
    let mut ctx = context_with_page();
    let log = eval_string(
        &mut ctx,
        r#"
        const f = (e) => log.push(e.eventPhase);
        main.addEventListener('r', f, true);
        main.addEventListener('r', f);
        main.addEventListener('r', f); // duplicate: ignored
        main.removeEventListener('r', f, { capture: true });
        p.dispatchEvent(new Event('r', { bubbles: true }));
        log.join(',')
        "#,
    );
    assert_eq!(log, "3");
}

#[test]
fn stop_propagation_finishes_current_target() {
    let mut ctx = context_with_page();
    let log = eval_string(
        &mut ctx,
        r#"
        p.addEventListener('s', (e) => { log.push('p1'); e.stopPropagation(); });
        p.addEventListener('s', () => log.push('p2'));
        main.addEventListener('s', () => log.push('main'));
        doc.addEventListener('s', () => log.push('doc'));
        p.dispatchEvent(new Event('s', { bubbles: true }));
        log.join(',')
        "#,
    );
    assert_eq!(log, "p1,p2");

    let log = eval_string(
        &mut ctx,
        r#"
        log.length = 0;
        main.addEventListener('c', (e) => { log.push('main-capture'); e.stopPropagation(); }, true);
        p.addEventListener('c', () => log.push('target'));
        p.dispatchEvent(new Event('c', { bubbles: true }));
        log.join(',')
        "#,
    );
    assert_eq!(log, "main-capture");
}

#[test]
fn stop_immediate_propagation_skips_remaining_listeners() {
    let mut ctx = context_with_page();
    let log = eval_string(
        &mut ctx,
        r#"
        p.addEventListener('i', (e) => { log.push('p1'); e.stopImmediatePropagation(); });
        p.addEventListener('i', () => log.push('p2'));
        main.addEventListener('i', () => log.push('main'));
        p.dispatchEvent(new Event('i', { bubbles: true }));
        log.join(',')
        "#,
    );
    assert_eq!(log, "p1");
}

#[test]
fn once_listeners_run_a_single_time() {
    let mut ctx = context_with_page();
    let log = eval_string(
        &mut ctx,
        r#"
        main.addEventListener('o', () => log.push('once'), { once: true });
        main.addEventListener('o', () => log.push('always'));
        p.dispatchEvent(new Event('o', { bubbles: true }));
        p.dispatchEvent(new Event('o', { bubbles: true }));
        log.join(',')
        "#,
    );
    assert_eq!(log, "once,always,always");
}

#[test]
fn listeners_see_current_target_and_this() {
    let mut ctx = context_with_page();
    assert!(eval_bool(
        &mut ctx,
        r#"
        let seen = [];
        main.addEventListener('ct', function (e) {
            seen.push(this === main, e.currentTarget === main, e.target === p);
        });
        doc.addEventListener('ct', function (e) {
            seen.push(this === doc, e.currentTarget === doc, e.target === p);
        });
        const handler = {
            handleEvent(e) { seen.push(this === handler, e.currentTarget === p); }
        };
        p.addEventListener('ct', handler);
        const ev = new Event('ct', { bubbles: true });
        p.dispatchEvent(ev);
        seen.length === 8 && seen.every((x) => x === true)
            && ev.currentTarget === null && ev.eventPhase === 0 && ev.target === p
        "#
    ));
}

#[test]
fn prevent_default_makes_dispatch_return_false() {
    let mut ctx = context_with_page();
    assert!(eval_bool(
        &mut ctx,
        r#"
        main.addEventListener('pd', (e) => e.preventDefault());
        const cancelable = new Event('pd', { bubbles: true, cancelable: true });
        const plain = new Event('pd', { bubbles: true });
        p.dispatchEvent(cancelable) === false
            && cancelable.defaultPrevented === true
            && p.dispatchEvent(plain) === true
            && p.dispatchEvent(new Event('nobody')) === true
        "#
    ));
}

#[test]
fn click_bubbles_and_is_cancelable() {
    let mut ctx = context_with_page();
    let log = eval_string(
        &mut ctx,
        r#"
        p.addEventListener('click', (e) => log.push('p:' + e.eventPhase));
        main.addEventListener('click', (e) => {
            log.push('main:' + e.type + ':' + e.target.id + ':' + e.bubbles + ':' + e.cancelable);
            e.preventDefault();
        });
        doc.addEventListener('click', (e) => log.push('doc:' + e.defaultPrevented));
        p.click();
        log.join(',')
        "#,
    );
    assert_eq!(log, "p:2,main:click:p1:true:true,doc:true");
}

#[test]
fn connected_events_reach_window_except_load() {
    let mut ctx = context_with_page();
    let log = eval_string(
        &mut ctx,
        r#"
        window.addEventListener('w', function (e) {
            log.push('window:' + (this === window) + ':' + (e.currentTarget === window));
        });
        addEventListener('w', () => log.push('global'));
        window.addEventListener('load', () => log.push('window-load'));
        doc.addEventListener('load', () => log.push('doc-load'));
        p.dispatchEvent(new Event('w', { bubbles: true }));
        p.dispatchEvent(new Event('load', { bubbles: true }));
        log.join(',')
        "#,
    );
    assert_eq!(log, "window:true:true,global,doc-load");
}

#[test]
fn detached_elements_dispatch_to_their_own_ancestors_only() {
    let mut ctx = context_with_page();
    let log = eval_string(
        &mut ctx,
        r#"
        main.remove();
        main.addEventListener('d', () => log.push('main'));
        doc.addEventListener('d', () => log.push('doc'));
        p.dispatchEvent(new Event('d', { bubbles: true }));
        log.join(',')
        "#,
    );
    assert_eq!(log, "main");
}
