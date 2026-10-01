use super::*;

const PAGE: &str = r#"<!DOCTYPE html>
<html><head><title>T &amp; t</title><style>a > b { color: red }</style></head>
<body>
<!-- note -->
<div id="main" class="box wide"><p>One <b>bold</b></p><p class="x">Two</p>
<ul><li>a</li><li class="x">b</li><li>c</li></ul>
<input type="text" name="q" value="&quot;hi&quot;"><br>
<script>if (1 < 2 && 3 > 2) {}</script>
</div>
<div id="other"><span>s</span></div>
</body></html>"#;

fn tree() -> DomTree {
    DomTree::from_html(PAGE)
}

fn by_id(tree: &DomTree, id: &str) -> NodeId {
    tree.find_by_id(id).unwrap_or_else(|| panic!("no #{id}"))
}

#[test]
fn builds_document_structure() {
    let tree = tree();
    let html = tree.document_element().unwrap();
    assert_eq!(tree.tag(html), Some("html"));
    assert!(tree.html_child("head").is_some());
    let body = tree.html_child("body").unwrap();
    assert_eq!(tree.parent(body), Some(html));
    assert!(matches!(
        tree.kind(tree.children(tree.document())[0]),
        Some(NodeKind::Doctype { .. })
    ));
}

#[test]
fn serialization_round_trips() {
    let first = tree().to_html();
    let second = DomTree::from_html(&first).to_html();
    assert_eq!(first, second);
    assert!(first.starts_with("<!DOCTYPE html><html>"));
    // Escaping rules
    assert!(first.contains("<title>T &amp; t</title>"));
    assert!(first.contains("<style>a > b { color: red }</style>"));
    assert!(first.contains("if (1 < 2 && 3 > 2) {}"));
    assert!(first.contains(r#"value="&quot;hi&quot;""#));
    assert!(first.contains("<br>") && !first.contains("</br>"));
    assert!(!first.contains("</input>"));
    assert!(first.contains("<!-- note -->"));
}

#[test]
fn text_escaping_and_nbsp() {
    let mut tree = DomTree::new();
    let div = tree.create_element("div");
    tree.append(tree.document(), div).unwrap();
    tree.set_text_content(div, "a < b & c\u{a0}d").unwrap();
    assert_eq!(tree.outer_html(div), "<div>a &lt; b &amp; c&nbsp;d</div>");
    assert_eq!(tree.text_content(div).unwrap(), "a < b & c\u{a0}d");
}

#[test]
fn mutations_and_identity() {
    let mut tree = tree();
    let main = by_id(&tree, "main");
    let other = by_id(&tree, "other");
    let generation = tree.generation();

    let span = tree.create_element("span");
    tree.set_attr(span, "id", "new").unwrap();
    tree.append(main, span).unwrap();
    assert!(tree.generation() > generation);
    assert_eq!(tree.find_by_id("new"), Some(span));
    assert_eq!(tree.children(main).last(), Some(&span));

    // Moving keeps identity
    tree.insert_before(other, span, tree.children(other).first().copied())
        .unwrap();
    assert_eq!(tree.parent(span), Some(other));
    assert_eq!(tree.children(other)[0], span);
    assert!(!tree.children(main).contains(&span));

    // Removal detaches; the node still exists
    tree.remove(span).unwrap();
    assert!(!tree.is_connected(span));
    assert_eq!(tree.find_by_id("new"), None);
    assert_eq!(tree.attr(span, "id"), Some("new"));

    // Replace
    let first_p = tree.element_children(main)[0];
    tree.replace(main, span, first_p).unwrap();
    assert_eq!(tree.element_children(main)[0], span);
    assert_eq!(tree.parent(first_p), None);

    // Cycles are refused
    let body = tree.html_child("body").unwrap();
    assert!(matches!(
        tree.append(main, body),
        Err(TreeError::HierarchyRequest(_))
    ));
    assert!(matches!(
        tree.append(main, main),
        Err(TreeError::HierarchyRequest(_))
    ));
}

#[test]
fn attributes() {
    let mut tree = tree();
    let main = by_id(&tree, "main");
    assert_eq!(tree.classes(main), vec!["box", "wide"]);
    tree.set_attr(main, "data-x", "1").unwrap();
    tree.set_attr(main, "data-x", "2").unwrap();
    assert_eq!(tree.attr(main, "data-x"), Some("2"));
    assert!(tree.remove_attr(main, "data-x").unwrap());
    assert!(!tree.remove_attr(main, "data-x").unwrap());
    assert_eq!(tree.attr(main, "data-x"), None);
}

#[test]
fn inner_html_and_text_content() {
    let mut tree = tree();
    let other = by_id(&tree, "other");
    let nodes = tree
        .set_inner_html(other, "<em>x</em>tail<!--c-->")
        .unwrap();
    assert_eq!(nodes.len(), 3);
    assert_eq!(tree.inner_html(other), "<em>x</em>tail<!--c-->");
    assert_eq!(tree.text_content(other).unwrap(), "xtail");

    tree.set_text_content(other, "<plain>").unwrap();
    assert_eq!(tree.inner_html(other), "&lt;plain&gt;");
    assert_eq!(tree.children(other).len(), 1);

    tree.set_text_content(other, "").unwrap();
    assert!(tree.children(other).is_empty());
}

#[test]
fn lookups() {
    let tree = tree();
    let main = by_id(&tree, "main");
    let doc = tree.document();
    assert_eq!(tree.elements_by_tag(doc, "LI").len(), 3);
    assert_eq!(tree.elements_by_tag(main, "p").len(), 2);
    assert_eq!(tree.elements_by_class(doc, "x").len(), 2);
    assert_eq!(tree.elements_by_class(doc, "wide box"), vec![main]);
    assert_eq!(tree.elements_by_name(doc, "q").len(), 1);
    // Excludes the scope itself
    assert!(tree.elements_by_tag(main, "div").is_empty());
}

#[test]
fn clone_node_copies_subtree() {
    let mut tree = tree();
    let main = by_id(&tree, "main");
    let shallow = tree.clone_node(main, false).unwrap();
    assert!(tree.children(shallow).is_empty());
    assert_eq!(tree.attr(shallow, "id"), Some("main"));
    let deep = tree.clone_node(main, true).unwrap();
    assert_eq!(tree.inner_html(deep), tree.inner_html(main));
    assert_eq!(tree.parent(deep), None);
}

#[test]
fn css_path_matches_scraper_paths() {
    let tree = tree();
    let parsed = scraper::Html::parse_document(PAGE);
    let all = scraper::Selector::parse("*").unwrap();
    let expected: Vec<String> = parsed
        .select(&all)
        .map(|el| crate::dom::document::css_path_for_scraper_element(&el))
        .collect();
    let actual: Vec<String> = tree
        .descendants(tree.document())
        .into_iter()
        .filter_map(|n| tree.css_path(n))
        .collect();
    assert_eq!(actual, expected);
    let li = tree.select(tree.document(), "li.x").unwrap()[0];
    assert_eq!(
        tree.css_path(li).unwrap(),
        "html>body:nth-child(1)>div:nth-child(1)>ul:nth-child(1)>li:nth-child(2)"
    );
}

#[test]
fn selectors() {
    let tree = tree();
    let doc = tree.document();
    let main = by_id(&tree, "main");
    assert_eq!(tree.select(doc, "li").unwrap().len(), 3);
    assert_eq!(tree.select(doc, "#main > p").unwrap().len(), 2);
    assert_eq!(tree.select(doc, "ul li:not(.x)").unwrap().len(), 2);
    assert_eq!(tree.select(doc, "input[name=q]").unwrap().len(), 1);
    assert_eq!(tree.select(doc, "p + p").unwrap().len(), 1);
    assert_eq!(tree.select(doc, "li ~ li").unwrap().len(), 2);
    // Scoped: descendants of #other only
    let other = by_id(&tree, "other");
    assert_eq!(tree.select(other, "span").unwrap().len(), 1);
    assert!(tree.select(other, "li").unwrap().is_empty());
    // Scope itself is excluded, but ancestors still take part in matching
    assert!(tree.select(main, "div").unwrap().is_empty());
    assert_eq!(tree.select(main, "body p").unwrap().len(), 2);
    // matches / closest
    let b = tree.select_first(doc, "b").unwrap().unwrap();
    assert!(tree.matches(b, "#main b").unwrap());
    assert!(!tree.matches(b, "#other b").unwrap());
    assert_eq!(tree.closest(b, "div").unwrap(), Some(main));
    assert_eq!(tree.closest(b, "table").unwrap(), None);
    // Bad selectors are errors, not panics
    assert!(tree.select(doc, "li[").is_err());
    assert!(tree.matches(main, ":::").is_err());
}

#[test]
fn select_sees_mutations() {
    let mut tree = tree();
    let doc = tree.document();
    assert!(tree.select(doc, ".added").unwrap().is_empty());
    let other = by_id(&tree, "other");
    let em = tree.create_element("em");
    tree.set_attr(em, "class", "added").unwrap();
    tree.append(other, em).unwrap();
    assert_eq!(tree.select(doc, "#other > .added").unwrap(), vec![em]);
    tree.remove(em).unwrap();
    assert!(tree.select(doc, ".added").unwrap().is_empty());
    // Detached subtrees can be queried on their own
    let inner = tree.create_element("i");
    tree.append(em, inner).unwrap();
    assert_eq!(tree.select(em, "i").unwrap(), vec![inner]);
    assert!(tree.matches(em, "em.added").unwrap());
}

#[test]
fn fragment_parsing_creates_detached_nodes() {
    let mut tree = DomTree::new();
    let nodes = tree.parse_fragment("<p>a</p><p>b</p>");
    assert_eq!(nodes.len(), 2);
    assert!(nodes.iter().all(|&n| tree.parent(n).is_none()));
    assert_eq!(tree.outer_html(nodes[1]), "<p>b</p>");
}

#[test]
fn deep_documents_do_not_overflow() {
    let depth = 20_000;
    let html = format!("{}x{}", "<div>".repeat(depth), "</div>".repeat(depth));
    let tree = DomTree::from_html(&html);
    let out = tree.to_html();
    assert!(out.contains('x'));
    assert!(tree.len() > 1000);
}
