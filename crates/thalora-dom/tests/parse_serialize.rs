//! Parsing and serialization round trips.

use thalora_dom::{Document, NodeData, NodeType, QuirksMode};

fn body_html(html: &str) -> String {
    let doc = Document::parse_html(html);
    doc.inner_html(doc.body().expect("body")).unwrap()
}

#[test]
fn full_document_round_trip() {
    let src = r#"<!DOCTYPE html><html><head><title>T</title></head><body><p class="x">y</p><!--c--></body></html>"#;
    let doc = Document::parse_html(src);
    assert_eq!(doc.outer_html(doc.document()).unwrap(), src);
    assert_eq!(doc.inner_html(doc.document()).unwrap(), src);
    assert_eq!(doc.quirks_mode(), QuirksMode::NoQuirks);
    let doctype = doc.doctype().expect("doctype");
    assert_eq!(doc.node_type(doctype).unwrap(), NodeType::DocumentType);
    assert_eq!(doc.node_name(doctype).unwrap(), "html");
}

#[test]
fn empty_input_gets_implied_structure_and_quirks() {
    let doc = Document::parse_html("");
    assert_eq!(
        doc.outer_html(doc.document()).unwrap(),
        "<html><head></head><body></body></html>"
    );
    assert_eq!(doc.quirks_mode(), QuirksMode::Quirks);
    assert!(doc.head().is_some());
    assert!(doc.body().is_some());
    assert!(doc.doctype().is_none());
}

#[test]
fn implied_end_tags() {
    assert_eq!(body_html("<p>a<p>b"), "<p>a</p><p>b</p>");
    assert_eq!(
        body_html("<ul><li>1<li>2</ul>"),
        "<ul><li>1</li><li>2</li></ul>"
    );
}

#[test]
fn void_elements() {
    assert_eq!(
        body_html("<br><img src=x><input type=text><hr/>"),
        r#"<br><img src="x"><input type="text"><hr>"#
    );
    let doc = Document::parse_html("<img id=i>");
    let img = doc.get_element_by_id("i").unwrap();
    assert_eq!(doc.inner_html(img).unwrap(), "");
    assert_eq!(doc.outer_html(img).unwrap(), r#"<img id="i">"#);
}

#[test]
fn entities_and_escaping() {
    let doc = Document::parse_html("<p id=p>a &amp; b &lt; c&nbsp;&copy;</p>");
    let p = doc.get_element_by_id("p").unwrap();
    assert_eq!(
        doc.text_content(p).unwrap().as_deref(),
        Some("a & b < c\u{a0}\u{a9}")
    );
    assert_eq!(doc.inner_html(p).unwrap(), "a &amp; b &lt; c&nbsp;\u{a9}");
}

#[test]
fn attribute_escaping() {
    assert_eq!(
        body_html(r#"<a title='x"y<z>&amp;'>t</a>"#),
        r#"<a title="x&quot;y&lt;z&gt;&amp;">t</a>"#
    );
}

#[test]
fn raw_text_is_not_escaped() {
    let src = "<script>if (a < b && c) {}</script><style>p > a {}</style>";
    assert_eq!(body_html(&format!("<body>{src}")), src);
}

#[test]
fn tables_are_fixed_up() {
    assert_eq!(
        body_html("<table><tr><td>a</td></tr></table>"),
        "<table><tbody><tr><td>a</td></tr></tbody></table>"
    );
    // Foster parenting: stray text moves in front of the table.
    assert_eq!(
        body_html("<table>x<tr><td>a</td></tr></table>"),
        "x<table><tbody><tr><td>a</td></tr></tbody></table>"
    );
}

#[test]
fn template_contents_are_a_separate_fragment() {
    let doc = Document::parse_html("<body><template id=t><p>hi</p></template></body>");
    let t = doc.get_element_by_id("t").unwrap();
    assert_eq!(doc.first_child(t).unwrap(), None);
    let contents = doc.template_contents(t).unwrap().expect("contents");
    assert_eq!(doc.node_type(contents).unwrap(), NodeType::DocumentFragment);
    assert_eq!(doc.inner_html(contents).unwrap(), "<p>hi</p>");
    assert_eq!(doc.inner_html(t).unwrap(), "<p>hi</p>");
    assert_eq!(
        doc.outer_html(t).unwrap(),
        r#"<template id="t"><p>hi</p></template>"#
    );
    // Template contents are not descendants of the document.
    assert_eq!(doc.query_selector(doc.document(), "p").unwrap(), None);
    assert!(!doc.is_connected(contents).unwrap());
}

#[test]
fn svg_names_keep_their_case() {
    let src = r##"<svg viewBox="0 0 1 1"><foreignObject><p>x</p></foreignObject><use xlink:href="#a"></use></svg>"##;
    assert_eq!(body_html(src), src);
    let doc = Document::parse_html(src);
    let svg = doc.query_selector(doc.document(), "svg").unwrap().unwrap();
    let e = doc.element_data(svg).unwrap();
    assert_eq!(&**e.namespace(), "http://www.w3.org/2000/svg");
    assert_eq!(e.tag_name(), "svg");
}

#[test]
fn navigation_and_iterators() {
    let doc = Document::parse_html("<div id=d><a></a>t<b><i></i></b></div>");
    let d = doc.get_element_by_id("d").unwrap();
    let kids: Vec<_> = doc.children(d).unwrap().collect();
    assert_eq!(kids.len(), 3);
    assert_eq!(doc.node_name(kids[0]).unwrap(), "A");
    assert_eq!(doc.node_name(kids[1]).unwrap(), "#text");
    assert_eq!(doc.next_sibling(kids[0]).unwrap(), Some(kids[1]));
    assert_eq!(doc.previous_sibling(kids[2]).unwrap(), Some(kids[1]));
    assert_eq!(doc.last_child(d).unwrap(), Some(kids[2]));
    assert_eq!(doc.parent(kids[2]).unwrap(), Some(d));
    let names: Vec<String> = doc
        .descendants(d)
        .unwrap()
        .map(|n| doc.node_name(n).unwrap())
        .collect();
    assert_eq!(names, ["A", "#text", "B", "I"]);
    let elems: Vec<_> = doc.element_children(d).unwrap().collect();
    assert_eq!(elems, [kids[0], kids[2]]);
    let i = doc.first_child(kids[2]).unwrap().unwrap();
    let up: Vec<String> = doc
        .ancestors(i)
        .unwrap()
        .map(|n| doc.node_name(n).unwrap())
        .collect();
    assert_eq!(up, ["B", "DIV", "BODY", "HTML", "#document"]);
    assert!(doc.contains(d, i).unwrap());
    assert!(!doc.contains(i, d).unwrap());
    assert!(matches!(doc.data(kids[1]).unwrap(), NodeData::Text(t) if t == "t"));
}

#[test]
fn parse_fragment_returns_detached_fragment() {
    let mut doc = Document::parse_html("<div id=d></div>");
    let d = doc.get_element_by_id("d").unwrap();
    let frag = doc.parse_fragment(d, "<i>1</i><i>2</i>").unwrap();
    assert_eq!(doc.node_type(frag).unwrap(), NodeType::DocumentFragment);
    assert_eq!(doc.parent(frag).unwrap(), None);
    assert_eq!(doc.inner_html(frag).unwrap(), "<i>1</i><i>2</i>");
    assert_eq!(doc.query_selector_all(frag, "i").unwrap().len(), 2);
}
