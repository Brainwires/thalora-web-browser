//! Tree mutation, validity errors, stale ids, innerHTML, cloning, the ID
//! index and mutation records.

use thalora_dom::{Document, DomError, MutationKind, NodeId};

fn doc_with(html: &str) -> Document {
    Document::parse_html(&format!("<!DOCTYPE html><body>{html}</body>"))
}

fn id(doc: &Document, s: &str) -> NodeId {
    doc.get_element_by_id(s)
        .unwrap_or_else(|| panic!("#{s} missing"))
}

#[test]
fn append_insert_remove_replace() {
    let mut doc = doc_with("<div id=a><span id=s></span></div>");
    let a = id(&doc, "a");
    let s = id(&doc, "s");
    let p = doc.create_element("P").unwrap();
    assert_eq!(doc.append_child(a, p).unwrap(), p);
    assert_eq!(doc.inner_html(a).unwrap(), r#"<span id="s"></span><p></p>"#);
    let t = doc.create_text_node("x");
    doc.insert_before(a, t, Some(s)).unwrap();
    assert_eq!(
        doc.inner_html(a).unwrap(),
        r#"x<span id="s"></span><p></p>"#
    );
    let c = doc.create_comment("c");
    assert_eq!(doc.replace_child(a, c, s).unwrap(), s);
    assert_eq!(doc.inner_html(a).unwrap(), "x<!--c--><p></p>");
    assert_eq!(doc.parent(s).unwrap(), None);
    assert_eq!(doc.remove_child(a, t).unwrap(), t);
    assert_eq!(doc.inner_html(a).unwrap(), "<!--c--><p></p>");
    // Removed nodes stay valid and can be re-inserted.
    doc.append_child(a, t).unwrap();
    assert_eq!(doc.inner_html(a).unwrap(), "<!--c--><p></p>x");
    // Moving a node within the same parent.
    doc.insert_before(a, t, Some(c)).unwrap();
    assert_eq!(doc.inner_html(a).unwrap(), "x<!--c--><p></p>");
    doc.remove(p).unwrap();
    assert_eq!(doc.inner_html(a).unwrap(), "x<!--c-->");
}

#[test]
fn fragment_insertion_moves_children() {
    let mut doc = doc_with("<div id=a></div>");
    let a = id(&doc, "a");
    let frag = doc.create_document_fragment();
    let b = doc.create_element("b").unwrap();
    let i = doc.create_element("i").unwrap();
    doc.append_child(frag, b).unwrap();
    doc.append_child(frag, i).unwrap();
    doc.append_child(a, frag).unwrap();
    assert_eq!(doc.inner_html(a).unwrap(), "<b></b><i></i>");
    assert_eq!(doc.first_child(frag).unwrap(), None);
}

#[test]
fn hierarchy_request_errors() {
    let mut doc = doc_with("<div id=a><span id=s></span></div>");
    let a = id(&doc, "a");
    let s = id(&doc, "s");
    let root = doc.document();
    let text = doc.create_text_node("t");
    let el = doc.create_element("x").unwrap();

    // Parent must be a document, fragment or element.
    assert_eq!(doc.append_child(text, el), Err(DomError::HierarchyRequest));
    // A node can't be inserted into its own subtree.
    assert_eq!(doc.append_child(s, a), Err(DomError::HierarchyRequest));
    assert_eq!(doc.append_child(a, a), Err(DomError::HierarchyRequest));
    // The document node can't be inserted anywhere.
    assert_eq!(doc.append_child(a, root), Err(DomError::HierarchyRequest));
    // Document constraints: one element, no text, doctype only in documents.
    assert_eq!(doc.append_child(root, el), Err(DomError::HierarchyRequest));
    assert_eq!(
        doc.append_child(root, text),
        Err(DomError::HierarchyRequest)
    );
    let dt = doc.create_doctype("html", "", "").unwrap();
    assert_eq!(doc.append_child(a, dt), Err(DomError::HierarchyRequest));
    assert_eq!(doc.append_child(root, dt), Err(DomError::HierarchyRequest));
    // A template can't go into its own contents (host-including ancestor).
    let tpl = doc.create_element("template").unwrap();
    let contents = doc.template_contents(tpl).unwrap().unwrap();
    assert_eq!(
        doc.append_child(contents, tpl),
        Err(DomError::HierarchyRequest)
    );
    // Comments are fine in the document.
    let c = doc.create_comment("ok");
    assert!(doc.append_child(root, c).is_ok());
    assert_eq!(DomError::HierarchyRequest.name(), "HierarchyRequestError");
}

#[test]
fn not_found_errors() {
    let mut doc = doc_with("<div id=a></div><div id=b><i id=i></i></div>");
    let a = id(&doc, "a");
    let b = id(&doc, "b");
    let i = id(&doc, "i");
    let el = doc.create_element("x").unwrap();
    assert_eq!(doc.insert_before(a, el, Some(i)), Err(DomError::NotFound));
    assert_eq!(doc.remove_child(a, i), Err(DomError::NotFound));
    assert_eq!(doc.replace_child(a, el, i), Err(DomError::NotFound));
    assert!(doc.remove_child(b, i).is_ok());
    assert_eq!(DomError::NotFound.name(), "NotFoundError");
}

#[test]
fn replace_document_element() {
    let mut doc = doc_with("");
    let root = doc.document();
    let html = doc.document_element().unwrap();
    let new_html = doc.create_element("html").unwrap();
    assert_eq!(doc.replace_child(root, new_html, html).unwrap(), html);
    assert_eq!(doc.document_element(), Some(new_html));
    // An element can't replace the doctype while another element exists.
    let doctype = doc.doctype().unwrap();
    let other = doc.create_element("x").unwrap();
    assert_eq!(
        doc.replace_child(root, other, doctype),
        Err(DomError::HierarchyRequest)
    );
}

#[test]
fn invalid_names() {
    let mut doc = doc_with("<div id=a></div>");
    let a = id(&doc, "a");
    assert_eq!(doc.create_element("1x"), Err(DomError::InvalidCharacter));
    assert_eq!(doc.create_element(""), Err(DomError::InvalidCharacter));
    assert_eq!(
        doc.set_attribute(a, "a b", "v"),
        Err(DomError::InvalidCharacter)
    );
    assert_eq!(
        doc.create_element_ns(None, "svg:rect"),
        Err(DomError::Namespace)
    );
    let rect = doc
        .create_element_ns(Some("http://www.w3.org/2000/svg"), "rect")
        .unwrap();
    assert_eq!(doc.node_name(rect).unwrap(), "rect");
}

#[test]
fn stale_ids_are_detected() {
    let mut doc = doc_with("<div id=a></div>");
    let a = id(&doc, "a");
    let el = doc.create_element("p").unwrap();
    let child = doc.create_text_node("c");
    doc.append_child(el, child).unwrap();
    let serial = doc.serial(el).unwrap();

    // Attached nodes and the document can't be freed.
    assert_eq!(doc.free_subtree(a), Err(DomError::HierarchyRequest));
    assert_eq!(
        doc.free_subtree(doc.document()),
        Err(DomError::NotSupported)
    );

    doc.free_subtree(el).unwrap();
    assert!(!doc.is_valid(el));
    assert!(!doc.is_valid(child));
    assert_eq!(doc.parent(el), Err(DomError::StaleNode(el)));
    assert_eq!(doc.append_child(a, child), Err(DomError::StaleNode(child)));
    assert_eq!(DomError::StaleNode(el).name(), "InvalidStateError");

    // Slots are reused with a new generation; old ids stay stale and the
    // serial is never reused.
    let fresh = doc.create_element("q").unwrap();
    let fresh2 = doc.create_element("r").unwrap();
    assert_ne!(fresh, el);
    assert_ne!(fresh2, el);
    assert!(!doc.is_valid(el));
    assert!(doc.serial(fresh).unwrap() > serial);
    assert!(doc.serial(fresh2).unwrap() > serial);

    // Garbage ids are errors, not panics.
    let junk = NodeId::from_bits(u64::MAX);
    assert_eq!(doc.data(junk).err(), Some(DomError::StaleNode(junk)));
    assert_eq!(NodeId::from_bits(fresh.to_bits()), fresh);
}

#[test]
fn removal_keeps_other_ids_valid() {
    let mut doc = doc_with("<div id=a><i id=x></i><b id=y></b></div>");
    let a = id(&doc, "a");
    let x = id(&doc, "x");
    let y = id(&doc, "y");
    doc.remove_child(a, x).unwrap();
    assert!(doc.is_valid(x));
    assert!(doc.is_valid(y));
    assert_eq!(doc.first_child(a).unwrap(), Some(y));
}

#[test]
fn serials_are_unique() {
    let mut doc = doc_with("<p></p>");
    let a = doc.create_element("a").unwrap();
    let b = doc.create_element("b").unwrap();
    let s = [
        doc.serial(doc.document()).unwrap(),
        doc.serial(a).unwrap(),
        doc.serial(b).unwrap(),
    ];
    assert!(s[0] < s[1] && s[1] < s[2]);
}

#[test]
fn attributes() {
    let mut doc = doc_with("<div id=a title=t></div>");
    let a = id(&doc, "a");
    assert_eq!(doc.get_attribute(a, "TITLE").unwrap(), Some("t"));
    doc.set_attribute(a, "DATA-X", "1").unwrap();
    assert_eq!(doc.get_attribute(a, "data-x").unwrap(), Some("1"));
    doc.set_attribute(a, "title", "u").unwrap();
    assert_eq!(
        doc.outer_html(a).unwrap(),
        r#"<div id="a" title="u" data-x="1"></div>"#
    );
    assert!(doc.remove_attribute(a, "title").unwrap());
    assert!(!doc.remove_attribute(a, "title").unwrap());
    assert!(!doc.has_attribute(a, "title").unwrap());
    doc.set_attribute_ns(a, Some("urn:x"), "p:k", "v").unwrap();
    assert_eq!(
        doc.get_attribute_ns(a, Some("urn:x"), "k").unwrap(),
        Some("v")
    );
    assert_eq!(doc.get_attribute(a, "p:k").unwrap(), Some("v"));
    assert!(doc.remove_attribute_ns(a, Some("urn:x"), "k").unwrap());
    assert_eq!(doc.attributes(a).unwrap().len(), 2);
    let t = doc.create_text_node("t");
    assert_eq!(doc.get_attribute(t, "x"), Err(DomError::InvalidNodeType));
}

#[test]
fn class_list_tokens() {
    let doc = doc_with("<div id=a class='  x y\tx z '></div><p id=p></p>");
    let a = id(&doc, "a");
    assert_eq!(
        doc.class_list(a).unwrap().collect::<Vec<_>>(),
        ["x", "y", "z"]
    );
    let p = id(&doc, "p");
    assert_eq!(doc.class_list(p).unwrap().count(), 0);
}

#[test]
fn get_element_by_id_tracks_mutations() {
    let mut doc = doc_with("<div id=a></div>");
    let a = id(&doc, "a");
    let body = doc.body().unwrap();

    // Detached elements are not found; inserting makes them findable.
    let x = doc.create_element("span").unwrap();
    doc.set_attribute(x, "id", "x").unwrap();
    assert_eq!(doc.get_element_by_id("x"), None);
    doc.append_child(a, x).unwrap();
    assert_eq!(doc.get_element_by_id("x"), Some(x));

    // Changing the attribute moves the entry.
    doc.set_attribute(x, "id", "y").unwrap();
    assert_eq!(doc.get_element_by_id("x"), None);
    assert_eq!(doc.get_element_by_id("y"), Some(x));
    doc.remove_attribute(x, "id").unwrap();
    assert_eq!(doc.get_element_by_id("y"), None);
    doc.set_attribute(x, "id", "x").unwrap();

    // Removing an ancestor removes descendants from the index.
    doc.remove_child(body, a).unwrap();
    assert_eq!(doc.get_element_by_id("a"), None);
    assert_eq!(doc.get_element_by_id("x"), None);
    doc.append_child(body, a).unwrap();
    assert_eq!(doc.get_element_by_id("x"), Some(x));

    // Duplicates: the first in tree order wins.
    let dup = doc.create_element("b").unwrap();
    doc.set_attribute(dup, "id", "x").unwrap();
    doc.append_child(body, dup).unwrap();
    assert_eq!(doc.get_element_by_id("x"), Some(x));
    doc.insert_before(body, dup, Some(a)).unwrap();
    assert_eq!(doc.get_element_by_id("x"), Some(dup));
    doc.remove(dup).unwrap();
    assert_eq!(doc.get_element_by_id("x"), Some(x));

    // innerHTML adds and removes ids.
    doc.set_inner_html(a, "<i id=n></i>").unwrap();
    assert!(doc.get_element_by_id("n").is_some());
    assert_eq!(doc.get_element_by_id("x"), None);
    doc.set_inner_html(a, "").unwrap();
    assert_eq!(doc.get_element_by_id("n"), None);
    assert_eq!(doc.get_element_by_id(""), None);
}

#[test]
fn inner_html_and_text_content() {
    let mut doc = doc_with("<div id=a><b>old</b></div><table id=t></table>");
    let a = id(&doc, "a");
    let markup = "<p>a</p>text<!--c--><br>";
    doc.set_inner_html(a, markup).unwrap();
    assert_eq!(doc.inner_html(a).unwrap(), markup);
    assert_eq!(doc.text_content(a).unwrap().as_deref(), Some("atext"));

    doc.set_text_content(a, "x<y").unwrap();
    assert_eq!(doc.inner_html(a).unwrap(), "x&lt;y");
    assert_eq!(doc.children(a).unwrap().count(), 1);
    doc.set_text_content(a, "").unwrap();
    assert_eq!(doc.first_child(a).unwrap(), None);
    assert_eq!(doc.text_content(doc.document()).unwrap(), None);

    // The context element drives the fragment parser (table context).
    let t = id(&doc, "t");
    doc.set_inner_html(t, "<tr><td>1</td></tr>").unwrap();
    assert_eq!(
        doc.inner_html(t).unwrap(),
        "<tbody><tr><td>1</td></tr></tbody>"
    );

    // Templates: innerHTML targets the contents.
    let tpl = doc.create_element("template").unwrap();
    doc.set_inner_html(tpl, "<i>z</i>").unwrap();
    assert_eq!(doc.first_child(tpl).unwrap(), None);
    assert_eq!(doc.inner_html(tpl).unwrap(), "<i>z</i>");

    // Character data.
    let txt = doc.create_text_node("one");
    doc.set_data(txt, "two").unwrap();
    assert_eq!(doc.text_content(txt).unwrap().as_deref(), Some("two"));
    doc.set_text_content(txt, "three").unwrap();
    assert_eq!(doc.text_content(txt).unwrap().as_deref(), Some("three"));
    assert_eq!(doc.set_data(a, "x"), Err(DomError::InvalidNodeType));
}

#[test]
fn clone_node() {
    let mut doc =
        doc_with("<div id=a class=c><p>x<b>y</b></p></div><template id=t><i>in</i></template>");
    let a = id(&doc, "a");
    let deep = doc.clone_node(a, true).unwrap();
    assert_ne!(deep, a);
    assert_eq!(doc.parent(deep).unwrap(), None);
    assert_eq!(doc.outer_html(deep).unwrap(), doc.outer_html(a).unwrap());
    // The clone is independent and not in the ID index.
    assert_eq!(doc.get_element_by_id("a"), Some(a));
    let p = doc.first_child(deep).unwrap().unwrap();
    doc.set_text_content(p, "changed").unwrap();
    assert_eq!(
        doc.outer_html(a).unwrap(),
        r#"<div id="a" class="c"><p>x<b>y</b></p></div>"#
    );

    let shallow = doc.clone_node(a, false).unwrap();
    assert_eq!(
        doc.outer_html(shallow).unwrap(),
        r#"<div id="a" class="c"></div>"#
    );

    let t = id(&doc, "t");
    let tc = doc.clone_node(t, true).unwrap();
    assert_eq!(doc.inner_html(tc).unwrap(), "<i>in</i>");
    assert_ne!(
        doc.template_contents(tc).unwrap(),
        doc.template_contents(t).unwrap()
    );
    let ts = doc.clone_node(t, false).unwrap();
    assert_eq!(doc.inner_html(ts).unwrap(), "");

    assert_eq!(
        doc.clone_node(doc.document(), true),
        Err(DomError::NotSupported)
    );
}

#[test]
fn mutation_records() {
    let mut doc = doc_with("<div id=a><span id=s></span></div><div id=b></div>");
    let a = id(&doc, "a");
    let b = id(&doc, "b");
    let s = id(&doc, "s");

    // Off by default: nothing is recorded.
    let q = doc.create_element("q").unwrap();
    doc.append_child(b, q).unwrap();
    assert!(doc.take_mutation_records().is_empty());

    doc.set_mutation_recording(true);
    let p = doc.create_element("p").unwrap();
    doc.append_child(a, p).unwrap();
    let recs = doc.take_mutation_records();
    assert_eq!(recs.len(), 1);
    let r = &recs[0];
    assert_eq!(r.kind, MutationKind::ChildList);
    assert_eq!(r.kind.as_str(), "childList");
    assert_eq!(r.target, a);
    assert_eq!(r.added_nodes, [p]);
    assert!(r.removed_nodes.is_empty());
    assert_eq!(r.previous_sibling, Some(s));
    assert_eq!(r.next_sibling, None);
    assert!(doc.take_mutation_records().is_empty());

    // Removal.
    doc.remove_child(a, s).unwrap();
    let recs = doc.take_mutation_records();
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].target, a);
    assert_eq!(recs[0].removed_nodes, [s]);
    assert_eq!(recs[0].previous_sibling, None);
    assert_eq!(recs[0].next_sibling, Some(p));

    // Moving a node: removal from the old parent, then insertion.
    doc.append_child(b, p).unwrap();
    let recs = doc.take_mutation_records();
    assert_eq!(recs.len(), 2);
    assert_eq!(recs[0].target, a);
    assert_eq!(recs[0].removed_nodes, [p]);
    assert_eq!(recs[1].target, b);
    assert_eq!(recs[1].added_nodes, [p]);
    assert_eq!(recs[1].previous_sibling, Some(q));

    // Attributes: added (no old value), changed, removed.
    doc.set_attribute(a, "title", "x").unwrap();
    doc.set_attribute(a, "title", "y").unwrap();
    doc.remove_attribute(a, "title").unwrap();
    let recs = doc.take_mutation_records();
    assert_eq!(recs.len(), 3);
    for r in &recs {
        assert_eq!(r.kind, MutationKind::Attributes);
        assert_eq!(r.target, a);
        assert_eq!(r.attribute_name.as_deref(), Some("title"));
        assert_eq!(r.attribute_namespace, None);
    }
    assert_eq!(recs[0].old_value, None);
    assert_eq!(recs[1].old_value.as_deref(), Some("x"));
    assert_eq!(recs[2].old_value.as_deref(), Some("y"));

    // Character data.
    let t = doc.create_text_node("old");
    doc.set_data(t, "new").unwrap();
    let recs = doc.take_mutation_records();
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].kind, MutationKind::CharacterData);
    assert_eq!(recs[0].target, t);
    assert_eq!(recs[0].old_value.as_deref(), Some("old"));

    // innerHTML: a single replace-all record.
    doc.set_inner_html(b, "<i></i><u></u>").unwrap();
    let recs = doc.take_mutation_records();
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].target, b);
    assert_eq!(recs[0].removed_nodes, [q, p]);
    let added: Vec<_> = doc.children(b).unwrap().collect();
    assert_eq!(recs[0].added_nodes, added);
    assert_eq!(added.len(), 2);

    // Fragment insertion: the fragment loses its children, the parent
    // gains them.
    let frag = doc.create_document_fragment();
    let x = doc.create_element("x").unwrap();
    doc.append_child(frag, x).unwrap();
    doc.take_mutation_records();
    doc.append_child(a, frag).unwrap();
    let recs = doc.take_mutation_records();
    assert_eq!(recs.len(), 2);
    assert_eq!(recs[0].target, frag);
    assert_eq!(recs[0].removed_nodes, [x]);
    assert_eq!(recs[1].target, a);
    assert_eq!(recs[1].added_nodes, [x]);

    // Recording can be switched off again.
    doc.set_mutation_recording(false);
    doc.set_attribute(a, "k", "v").unwrap();
    assert!(doc.take_mutation_records().is_empty());
}
