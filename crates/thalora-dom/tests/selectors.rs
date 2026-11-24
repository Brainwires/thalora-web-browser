//! Selector matching: querySelector(All), matches, closest.

use thalora_dom::{Document, DomError, NodeId, Selector};

const PAGE: &str = concat!(
    "<!DOCTYPE html><body>",
    "<div id=root class='box main'>",
    "<p class=x>1</p>",
    "<span class='x y'>2</span>",
    "<ul><li>a</li><li class=k>b</li><li>c</li><li class=k>d</li></ul>",
    "<a href='http://e.com/x' data-n=1>l</a>",
    "<section><img alt=''></section>",
    "<form><input type=checkbox checked><input disabled><input required></form>",
    "</div></body>",
);

fn page() -> Document {
    Document::parse_html(PAGE)
}

fn qs(doc: &Document, sel: &str) -> Option<NodeId> {
    doc.query_selector(doc.document(), sel).unwrap()
}

fn count(doc: &Document, sel: &str) -> usize {
    doc.query_selector_all(doc.document(), sel).unwrap().len()
}

fn text(doc: &Document, n: Option<NodeId>) -> String {
    doc.text_content(n.expect("match")).unwrap().unwrap()
}

#[test]
fn simple_selectors() {
    let doc = page();
    assert_eq!(qs(&doc, "#root"), doc.get_element_by_id("root"));
    assert_eq!(qs(&doc, "DIV#root"), doc.get_element_by_id("root"));
    assert_eq!(count(&doc, ".x"), 2);
    assert_eq!(count(&doc, ".x.y"), 1);
    assert_eq!(count(&doc, ".box.main"), 1);
    assert_eq!(count(&doc, "li"), 4);
    assert_eq!(
        count(&doc, "*"),
        doc.descendants(doc.document())
            .unwrap()
            .filter(|&n| doc.element_data(n).is_ok())
            .count()
    );
    assert_eq!(qs(&doc, "nope"), None);
}

#[test]
fn combinators() {
    let doc = page();
    assert_eq!(text(&doc, qs(&doc, "div > p")), "1");
    assert_eq!(count(&doc, "div li"), 4);
    assert_eq!(text(&doc, qs(&doc, "p + span")), "2");
    assert_eq!(count(&doc, "p ~ *"), 5);
    assert_eq!(count(&doc, "body > li"), 0);
}

#[test]
fn attribute_selectors() {
    let doc = page();
    let a = qs(&doc, "a").unwrap();
    assert_eq!(qs(&doc, "[data-n='1']"), Some(a));
    assert_eq!(qs(&doc, "[href^='http']"), Some(a));
    assert_eq!(qs(&doc, "a[href$='/x']"), Some(a));
    assert_eq!(qs(&doc, "[href*='e.com']"), Some(a));
    assert_eq!(qs(&doc, "[class~='y']"), qs(&doc, "span"));
    assert_eq!(qs(&doc, "[data-n='2']"), None);
    assert_eq!(qs(&doc, "[DATA-N]"), Some(a));
}

#[test]
fn structural_and_logical_pseudo_classes() {
    let doc = page();
    assert_eq!(count(&doc, "li:not(.k)"), 2);
    assert_eq!(text(&doc, qs(&doc, "li:nth-child(2)")), "b");
    assert_eq!(count(&doc, "li:nth-child(odd)"), 2);
    assert_eq!(text(&doc, qs(&doc, "li:nth-child(2 of .k)")), "d");
    assert_eq!(text(&doc, qs(&doc, "li:last-child")), "d");
    assert_eq!(text(&doc, qs(&doc, "li:first-of-type")), "a");
    assert_eq!(text(&doc, qs(&doc, "li:nth-last-child(1)")), "d");
    assert_eq!(count(&doc, ":is(p, span)"), 2);
    assert_eq!(count(&doc, ":where(p, span).y"), 1);
    assert!(qs(&doc, "section:has(> img)").is_some());
    assert!(qs(&doc, "section:has(> p)").is_none());
    assert_eq!(qs(&doc, "img:empty"), qs(&doc, "img"));
    assert_eq!(qs(&doc, ":root"), doc.document_element());
}

#[test]
fn non_tree_structural_pseudo_classes() {
    let doc = page();
    assert_eq!(qs(&doc, ":any-link"), qs(&doc, "a"));
    assert_eq!(count(&doc, "input:checked"), 1);
    assert_eq!(count(&doc, "input:disabled"), 1);
    assert_eq!(count(&doc, "input:enabled"), 2);
    assert_eq!(count(&doc, "input:required"), 1);
    assert_eq!(count(&doc, ":hover"), 0);
    // Pseudo-elements parse but never match.
    assert_eq!(qs(&doc, "p::before"), None);
}

#[test]
fn scoped_queries() {
    let doc = page();
    let root = doc.get_element_by_id("root").unwrap();
    let ul = doc.query_selector(root, "ul").unwrap().unwrap();
    let first_li = doc.query_selector(ul, "li").unwrap();
    assert_eq!(text(&doc, first_li), "a");
    // Selectors match against the whole tree, not just the scope subtree.
    assert_eq!(doc.query_selector(ul, "div li").unwrap(), first_li);
    assert_eq!(
        doc.query_selector(root, ":scope > ul > li").unwrap(),
        first_li
    );
    assert_eq!(doc.query_selector(ul, ":scope > p").unwrap(), None);
    // The scope itself is never a result.
    assert_eq!(doc.query_selector(ul, "ul").unwrap(), None);
    let t = doc.first_child(first_li.unwrap()).unwrap().unwrap();
    assert_eq!(doc.query_selector(t, "p"), Err(DomError::InvalidNodeType));
}

#[test]
fn matches_and_closest() {
    let doc = page();
    let span = qs(&doc, "span").unwrap();
    assert!(doc.matches(span, ".y").unwrap());
    assert!(doc.matches(span, "div > span.x").unwrap());
    assert!(!doc.matches(span, "p").unwrap());
    let li = qs(&doc, "li").unwrap();
    let ul = qs(&doc, "ul").unwrap();
    assert_eq!(doc.closest(li, "ul").unwrap(), Some(ul));
    assert_eq!(doc.closest(li, "li").unwrap(), Some(li));
    assert_eq!(
        doc.closest(li, "#root").unwrap(),
        doc.get_element_by_id("root")
    );
    assert_eq!(doc.closest(li, "table").unwrap(), None);
    let t = doc.first_child(li).unwrap().unwrap();
    assert_eq!(doc.matches(t, "li"), Err(DomError::InvalidNodeType));
}

#[test]
fn invalid_selectors_are_syntax_errors() {
    let doc = page();
    for bad in ["div[", "", ":nope", "p >", "#", "a,,b", "::nope"] {
        let err = doc.query_selector(doc.document(), bad).unwrap_err();
        assert!(matches!(err, DomError::Syntax(_)), "{bad:?} gave {err:?}");
        assert_eq!(err.name(), "SyntaxError");
        assert!(doc.query_selector_all(doc.document(), bad).is_err());
    }
    let span = qs(&doc, "span").unwrap();
    assert!(matches!(doc.matches(span, "["), Err(DomError::Syntax(_))));
    assert!(matches!(doc.closest(span, "]"), Err(DomError::Syntax(_))));
    assert!(Selector::parse("div, p").is_ok());
}

#[test]
fn queries_follow_mutations() {
    let mut doc = page();
    let ul = qs(&doc, "ul").unwrap();
    let li = doc.create_element("li").unwrap();
    doc.set_attribute(li, "class", "k new").unwrap();
    doc.append_child(ul, li).unwrap();
    assert_eq!(count(&doc, "li.k"), 3);
    assert_eq!(qs(&doc, "li:last-child.new"), Some(li));
    doc.remove(li).unwrap();
    assert_eq!(count(&doc, ".new"), 0);
    // A compiled selector can be reused.
    let sel = Selector::parse("li.k").unwrap();
    assert_eq!(doc.select_all(doc.document(), &sel).unwrap().len(), 2);
    assert!(
        doc.element_matches(qs(&doc, "li.k").unwrap(), &sel)
            .unwrap()
    );
}
