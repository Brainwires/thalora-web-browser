//! CSS selector matching over a [`DomTree`].
//!
//! Rather than implementing `selectors::Element` for the arena, the tree
//! containing the scope is serialized with every element stamped with its
//! node id, parsed by scraper (cached until the next mutation), and scraper's
//! matches are mapped back to node ids.
//!
//! Known limits: each mutation invalidates the cache, so selector queries
//! after a mutation cost O(document); markup the HTML parser would reshape
//! (e.g. `<p>` inside `<p>`, a detached `<tr>`) can match slightly
//! differently than the live tree.

use std::cell::RefCell;
use std::fmt;

use super::{DomTree, NODE_ID_ATTR, NodeId, NodeKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectError(pub String);

impl fmt::Display for SelectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SyntaxError: '{}' is not a valid selector", self.0)
    }
}

impl std::error::Error for SelectError {}

struct Parsed {
    generation: u64,
    root: NodeId,
    html: scraper::Html,
}

/// The last parse of a tree, valid until its generation changes.
#[derive(Default)]
pub(crate) struct SelectCache(RefCell<Option<Parsed>>);

fn parse_selector(selector: &str) -> Result<scraper::Selector, SelectError> {
    scraper::Selector::parse(selector).map_err(|_| SelectError(selector.to_string()))
}

/// All elements under `root` (inclusive) matching `selector`, as node ids
/// in document order.
fn matching_ids(tree: &DomTree, root: NodeId, selector: &scraper::Selector) -> Vec<NodeId> {
    let mut cache = tree.select_cache.0.borrow_mut();
    let fresh = cache
        .as_ref()
        .is_some_and(|p| p.generation == tree.generation() && p.root == root);
    if !fresh {
        let markup = tree.serialize_with_ids(root);
        let html = if matches!(tree.kind(root), Some(NodeKind::Document)) {
            scraper::Html::parse_document(&markup)
        } else {
            scraper::Html::parse_fragment(&markup)
        };
        *cache = Some(Parsed {
            generation: tree.generation(),
            root,
            html,
        });
    }
    let parsed = cache.as_ref().expect("cache filled above");
    parsed
        .html
        .select(selector)
        .filter_map(|el| el.value().attr(NODE_ID_ATTR)?.parse::<NodeId>().ok())
        .filter(|&id| (id as usize) < tree.len())
        .collect()
}

/// `scope.querySelectorAll(selector)`: matching descendants of `scope`.
pub(super) fn select(
    tree: &DomTree,
    scope: NodeId,
    selector: &str,
) -> Result<Vec<NodeId>, SelectError> {
    let selector = parse_selector(selector)?;
    let root = tree.root_of(scope);
    Ok(matching_ids(tree, root, &selector)
        .into_iter()
        .filter(|&id| id != scope && tree.is_inclusive_ancestor(scope, id))
        .collect())
}

/// `element.matches(selector)`.
pub(super) fn matches(tree: &DomTree, id: NodeId, selector: &str) -> Result<bool, SelectError> {
    let selector = parse_selector(selector)?;
    if !tree.is_element(id) {
        return Ok(false);
    }
    let root = tree.root_of(id);
    Ok(matching_ids(tree, root, &selector).contains(&id))
}
