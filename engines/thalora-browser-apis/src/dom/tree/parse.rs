//! Build [`DomTree`] nodes from scraper's HTML parse.

use super::{DomTree, NodeId, NodeKind};

/// Convert one scraper node; `None` for nodes the tree doesn't model.
fn convert(node: &scraper::Node) -> Option<NodeKind> {
    Some(match node {
        scraper::Node::Doctype(doctype) => NodeKind::Doctype {
            name: doctype.name().to_string(),
            public_id: doctype.public_id().to_string(),
            system_id: doctype.system_id().to_string(),
        },
        scraper::Node::Element(element) => NodeKind::Element {
            tag: element.name().to_string(),
            attrs: element
                .attrs()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
        },
        scraper::Node::Text(text) => NodeKind::Text(text.to_string()),
        scraper::Node::Comment(comment) => NodeKind::Comment(comment.to_string()),
        _ => return None,
    })
}

/// Copy the children of `source` (recursively) under `parent`, without
/// recursion so deeply nested pages can't overflow the stack.
fn copy_children(tree: &mut DomTree, source: ego_tree_ref::NodeRef<'_>, parent: NodeId) {
    let mut stack: Vec<(ego_tree_ref::NodeRef<'_>, NodeId)> = source
        .children()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|child| (child, parent))
        .collect();
    while let Some((node, parent)) = stack.pop() {
        let Some(kind) = convert(node.value()) else {
            continue;
        };
        let id = tree.push(kind);
        tree.attach(parent, id, None);
        let children: Vec<_> = node.children().collect();
        stack.extend(children.into_iter().rev().map(|child| (child, id)));
    }
}

/// Fill the document node of an empty `tree` from `html`.
pub(super) fn build_document(tree: &mut DomTree, html: &str) {
    let parsed = scraper::Html::parse_document(html);
    let document = tree.document();
    copy_children(tree, parsed.tree.root(), document);
}

/// Parse `html` as body content into new detached nodes.
pub(super) fn build_fragment(tree: &mut DomTree, html: &str) -> Vec<NodeId> {
    let parsed = scraper::Html::parse_fragment(html);
    // scraper puts fragment content under an `<html>` context element
    let root = parsed.tree.root();
    let container = root
        .children()
        .find(|child| {
            child
                .value()
                .as_element()
                .is_some_and(|e| e.name().eq_ignore_ascii_case("html"))
        })
        .unwrap_or(root);

    // Parse into a scratch parent, then detach its children
    let holder = tree.push(NodeKind::Document);
    copy_children(tree, container, holder);
    let nodes = tree.children(holder).to_vec();
    for &node in &nodes {
        tree.nodes[node as usize].parent = None;
    }
    tree.nodes[holder as usize].children.clear();
    tree.touch();
    nodes
}

/// scraper's tree node handle, named through scraper so this crate needs no
/// direct ego_tree dependency.
mod ego_tree_ref {
    pub type NodeRef<'a> = <scraper::ElementRef<'a> as std::ops::Deref>::Target;
}
