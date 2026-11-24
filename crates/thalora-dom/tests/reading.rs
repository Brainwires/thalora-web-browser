//! Read-only conveniences: child text, element-only walks, `as_element`.

use thalora_dom::{Document, NodeId};

fn qs(doc: &Document, sel: &str) -> NodeId {
    doc.query_selector(doc.document(), sel)
        .unwrap()
        .expect("match")
}

#[test]
fn child_text_content_ignores_descendant_elements() {
    let doc = Document::parse_html("<p id=p>one <b>bold</b> two<!--c--></p>");
    let p = qs(&doc, "#p");
    assert_eq!(doc.child_text_content(p).unwrap(), "one  two");
    assert_eq!(doc.text_content(p).unwrap().unwrap(), "one bold two");
    let b = qs(&doc, "b");
    assert_eq!(doc.child_text_content(b).unwrap(), "bold");
    // The document has no text children
    assert_eq!(doc.child_text_content(doc.document()).unwrap(), "");
}

#[test]
fn descendant_elements_are_in_tree_order_and_skip_template_contents() {
    let doc = Document::parse_html(
        "<div id=d><span>a</span>text<p><i>b</i></p><template><em>t</em></template></div>",
    );
    let d = qs(&doc, "#d");
    let names: Vec<String> = doc
        .descendant_elements(d)
        .unwrap()
        .map(|n| doc.as_element(n).unwrap().local_name().to_string())
        .collect();
    assert_eq!(names, ["span", "p", "i", "template"]);
}

#[test]
fn ancestor_elements_stop_at_the_document() {
    let doc = Document::parse_html("<div><p><i id=i>x</i></p></div>");
    let i = qs(&doc, "#i");
    let names: Vec<String> = doc
        .ancestor_elements(i)
        .unwrap()
        .map(|n| doc.as_element(n).unwrap().local_name().to_string())
        .collect();
    assert_eq!(names, ["p", "div", "body", "html"]);
    let html = doc.document_element().unwrap();
    assert_eq!(doc.ancestor_elements(html).unwrap().count(), 0);
}

#[test]
fn as_element_and_has_attr() {
    let mut doc = Document::parse_html("<input id=x checked>");
    let x = qs(&doc, "#x");
    let e = doc.as_element(x).unwrap();
    assert!(e.has_attr("checked"));
    assert!(!e.has_attr("disabled"));
    assert!(doc.as_element(doc.document()).is_none());
    let text = doc.create_text_node("t");
    assert!(doc.as_element(text).is_none());
    // Stale ids are `None`, not a panic
    doc.remove(x).unwrap();
    doc.free_subtree(x).unwrap();
    assert!(doc.as_element(x).is_none());
}

#[test]
fn descendant_texts_yields_each_text_node() {
    let doc = Document::parse_html("<div id=d>a<b>b</b><!--c--><p>c<i>d</i></p></div>");
    let d = qs(&doc, "#d");
    let pieces: Vec<&str> = doc.descendant_texts(d).unwrap().collect();
    assert_eq!(pieces, ["a", "b", "c", "d"]);
    assert_eq!(pieces.join(" "), "a b c d");
    assert_eq!(doc.text_content(d).unwrap().unwrap(), pieces.concat());
}
