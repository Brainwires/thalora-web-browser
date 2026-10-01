//! JS-level tests for documents backed by the persistent DOM tree.

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
fn lookups_preserve_identity() {
    let mut ctx = context_with_page();
    assert!(eval_bool(
        &mut ctx,
        "doc.getElementById('main') === doc.querySelector('#main')"
    ));
    assert!(eval_bool(&mut ctx, "doc.body === doc.body"));
    assert!(eval_bool(
        &mut ctx,
        "doc.getElementById('p1').parentNode === doc.getElementById('main')"
    ));
    assert!(eval_bool(
        &mut ctx,
        "doc.documentElement.parentNode === doc"
    ));
    assert_eq!(
        eval_string(&mut ctx, "doc.getElementById('p1').tagName"),
        "P"
    );
    assert_eq!(
        eval_string(&mut ctx, "doc.querySelectorAll('li').length"),
        "2"
    );
    assert!(eval_bool(&mut ctx, "doc.getElementById('nope') === null"));
}

#[test]
fn mutations_show_up_in_document_html() {
    let mut ctx = context_with_page();
    eval_string(
        &mut ctx,
        r#"
        const li = doc.createElement('li');
        li.textContent = 'c';
        li.setAttribute('data-k', '1');
        doc.getElementById('list').appendChild(li);
        doc.getElementById('p1').remove();
        doc.querySelector('.x').className = 'y';
        "#,
    );
    let html = eval_string(&mut ctx, "doc.documentElement.outerHTML");
    assert!(html.contains(r#"<li data-k="1">c</li></ul>"#), "{html}");
    assert!(!html.contains("p1"), "{html}");
    assert!(html.contains(r#"<p class="y">Two</p>"#), "{html}");
    // Created element keeps its identity once inserted
    assert!(eval_bool(&mut ctx, "doc.querySelector('[data-k]') === li"));
    assert!(eval_bool(
        &mut ctx,
        "li.parentElement.id === 'list' && li.isConnected"
    ));
}

#[test]
fn inner_html_and_traversal() {
    let mut ctx = context_with_page();
    eval_string(
        &mut ctx,
        "doc.getElementById('main').innerHTML = '<span id=s>hi</span> there';",
    );
    assert_eq!(
        eval_string(&mut ctx, "doc.getElementById('s').textContent"),
        "hi"
    );
    assert_eq!(
        eval_string(&mut ctx, "doc.getElementById('main').textContent"),
        "hi there"
    );
    assert!(eval_bool(
        &mut ctx,
        "doc.getElementById('main').firstElementChild === doc.getElementById('s')"
    ));
    assert_eq!(
        eval_string(&mut ctx, "doc.getElementById('main').childNodes.length"),
        "2"
    );
    assert_eq!(
        eval_string(&mut ctx, "doc.getElementById('s').closest('.box').id"),
        "main"
    );
    assert!(eval_bool(
        &mut ctx,
        "doc.getElementById('s').matches('div > span')"
    ));
}

#[test]
fn text_nodes_write_through() {
    let mut ctx = context_with_page();
    eval_string(
        &mut ctx,
        r#"
        const t = doc.createTextNode('x');
        doc.getElementById('p1').appendChild(t);
        t.data = 'yz';
        "#,
    );
    assert_eq!(
        eval_string(&mut ctx, "doc.getElementById('p1').textContent"),
        "Oneyz"
    );
}

#[test]
fn insert_adjacent_and_append_strings() {
    let mut ctx = context_with_page();
    eval_string(
        &mut ctx,
        r#"
        const ul = doc.getElementById('list');
        ul.insertAdjacentHTML('beforeend', '<li id="last">z</li>');
        ul.prepend('start');
        "#,
    );
    let html = eval_string(&mut ctx, "doc.getElementById('list').outerHTML");
    assert!(html.starts_with(r#"<ul id="list">start<li>"#), "{html}");
    assert!(html.ends_with(r#"<li id="last">z</li></ul>"#), "{html}");
}

#[test]
fn hierarchy_errors_are_thrown() {
    let mut ctx = context_with_page();
    assert!(eval_bool(
        &mut ctx,
        r#"
        let threw = false;
        try { doc.getElementById('p1').appendChild(doc.body); } catch (e) { threw = true; }
        threw
        "#,
    ));
}
