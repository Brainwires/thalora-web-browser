//! Persistent DOM tree with node identity.
//!
//! A pure-Rust arena of DOM nodes, built once from the page HTML and then
//! mutated in place by DOM APIs. Nodes are addressed by [`NodeId`] and are
//! never freed while the tree lives (removed nodes become detached), so a JS
//! wrapper holding an id always refers to the same node.
//!
//! This module has no JS dependencies; the binding layer maps ids to
//! wrapper objects. Selector matching re-uses scraper (see [`select`]).

mod parse;
mod path;
pub mod select;
mod serialize;

#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

pub use select::SelectError;

/// Index of a node in its [`DomTree`].
pub type NodeId = u32;

/// A tree shared between a document and its bound nodes.
pub type SharedTree = Rc<RefCell<DomTree>>;

/// Attribute name used to map scraper matches back to tree nodes.
pub(crate) const NODE_ID_ATTR: &str = "data-thalora-node-id";

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Document,
    Doctype {
        name: String,
        public_id: String,
        system_id: String,
    },
    Element {
        tag: String,
        attrs: Vec<(String, String)>,
    },
    Text(String),
    Comment(String),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeError {
    /// The id does not name a node in this tree.
    NotFound(NodeId),
    /// The operation would create a cycle or put a node where it can't go.
    HierarchyRequest(&'static str),
}

impl fmt::Display for TreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TreeError::NotFound(id) => write!(f, "NotFoundError: no node {id}"),
            TreeError::HierarchyRequest(why) => write!(f, "HierarchyRequestError: {why}"),
        }
    }
}

impl std::error::Error for TreeError {}

/// Elements that never have children or an end tag.
pub(crate) fn is_void_element(tag: &str) -> bool {
    matches!(
        tag.to_ascii_lowercase().as_str(),
        "area"
            | "base"
            | "basefont"
            | "bgsound"
            | "br"
            | "col"
            | "embed"
            | "frame"
            | "hr"
            | "img"
            | "input"
            | "keygen"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

pub struct DomTree {
    nodes: Vec<Node>,
    generation: u64,
    select_cache: select::SelectCache,
}

impl Default for DomTree {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for DomTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DomTree")
            .field("nodes", &self.nodes.len())
            .field("generation", &self.generation)
            .finish()
    }
}

impl DomTree {
    /// An empty tree holding only the document node.
    pub fn new() -> Self {
        Self {
            nodes: vec![Node {
                kind: NodeKind::Document,
                parent: None,
                children: Vec::new(),
            }],
            generation: 0,
            select_cache: Default::default(),
        }
    }

    /// Build a tree from a full HTML document.
    pub fn from_html(html: &str) -> Self {
        let mut tree = Self::new();
        parse::build_document(&mut tree, html);
        tree
    }

    /// Wrap the tree for sharing.
    pub fn into_shared(self) -> SharedTree {
        Rc::new(RefCell::new(self))
    }

    /// The document node.
    pub fn document(&self) -> NodeId {
        0
    }

    /// Bumped on every mutation.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    fn touch(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.len() <= 1
    }

    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id as usize)
    }

    fn node_mut(&mut self, id: NodeId) -> Result<&mut Node, TreeError> {
        self.nodes
            .get_mut(id as usize)
            .ok_or(TreeError::NotFound(id))
    }

    fn check(&self, id: NodeId) -> Result<&Node, TreeError> {
        self.node(id).ok_or(TreeError::NotFound(id))
    }

    pub fn kind(&self, id: NodeId) -> Option<&NodeKind> {
        self.node(id).map(|n| &n.kind)
    }

    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.node(id).and_then(|n| n.parent)
    }

    pub fn children(&self, id: NodeId) -> &[NodeId] {
        self.node(id).map_or(&[], |n| n.children.as_slice())
    }

    /// Element children only.
    pub fn element_children(&self, id: NodeId) -> Vec<NodeId> {
        self.children(id)
            .iter()
            .copied()
            .filter(|&c| self.is_element(c))
            .collect()
    }

    pub fn is_element(&self, id: NodeId) -> bool {
        matches!(self.kind(id), Some(NodeKind::Element { .. }))
    }

    /// Tag name as parsed (lowercase for HTML elements).
    pub fn tag(&self, id: NodeId) -> Option<&str> {
        match self.kind(id) {
            Some(NodeKind::Element { tag, .. }) => Some(tag),
            _ => None,
        }
    }

    pub fn previous_sibling(&self, id: NodeId) -> Option<NodeId> {
        let siblings = self.children(self.parent(id)?);
        let index = siblings.iter().position(|&c| c == id)?;
        index.checked_sub(1).map(|i| siblings[i])
    }

    pub fn next_sibling(&self, id: NodeId) -> Option<NodeId> {
        let siblings = self.children(self.parent(id)?);
        let index = siblings.iter().position(|&c| c == id)?;
        siblings.get(index + 1).copied()
    }

    /// Topmost ancestor (the document for connected nodes).
    pub fn root_of(&self, mut id: NodeId) -> NodeId {
        while let Some(parent) = self.parent(id) {
            id = parent;
        }
        id
    }

    /// Whether `id` is in the document.
    pub fn is_connected(&self, id: NodeId) -> bool {
        self.root_of(id) == self.document()
    }

    /// Whether `ancestor` is `node` or one of its ancestors.
    pub fn is_inclusive_ancestor(&self, ancestor: NodeId, node: NodeId) -> bool {
        let mut current = Some(node);
        while let Some(id) = current {
            if id == ancestor {
                return true;
            }
            current = self.parent(id);
        }
        false
    }

    /// `id` and its descendants in document (pre-)order.
    pub fn descendants(&self, id: NodeId) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack = vec![id];
        while let Some(current) = stack.pop() {
            out.push(current);
            stack.extend(self.children(current).iter().rev().copied());
        }
        out
    }

    /// The document's `<html>` element.
    pub fn document_element(&self) -> Option<NodeId> {
        self.element_children(self.document()).into_iter().next()
    }

    /// First child element of `<html>` named `tag` (`head` / `body`).
    pub fn html_child(&self, tag: &str) -> Option<NodeId> {
        let html = self.document_element()?;
        self.element_children(html)
            .into_iter()
            .find(|&c| self.tag(c).is_some_and(|t| t.eq_ignore_ascii_case(tag)))
    }

    // ----- creation -------------------------------------------------------

    fn push(&mut self, kind: NodeKind) -> NodeId {
        let id = self.nodes.len() as NodeId;
        self.nodes.push(Node {
            kind,
            parent: None,
            children: Vec::new(),
        });
        id
    }

    pub fn create_element(&mut self, tag: &str) -> NodeId {
        self.push(NodeKind::Element {
            tag: tag.to_string(),
            attrs: Vec::new(),
        })
    }

    pub fn create_element_with_attrs(&mut self, tag: &str, attrs: Vec<(String, String)>) -> NodeId {
        self.push(NodeKind::Element {
            tag: tag.to_string(),
            attrs,
        })
    }

    pub fn create_text(&mut self, data: &str) -> NodeId {
        self.push(NodeKind::Text(data.to_string()))
    }

    pub fn create_comment(&mut self, data: &str) -> NodeId {
        self.push(NodeKind::Comment(data.to_string()))
    }

    /// Copy `id` (and its subtree when `deep`) as a new detached node.
    pub fn clone_node(&mut self, id: NodeId, deep: bool) -> Result<NodeId, TreeError> {
        let kind = self.check(id)?.kind.clone();
        let copy = self.push(kind);
        if deep {
            let children = self.children(id).to_vec();
            for child in children {
                let child_copy = self.clone_node(child, true)?;
                self.attach(copy, child_copy, None);
            }
        }
        Ok(copy)
    }

    // ----- mutation -------------------------------------------------------

    fn detach(&mut self, id: NodeId) {
        if let Some(parent) = self.parent(id) {
            if let Some(node) = self.nodes.get_mut(parent as usize) {
                node.children.retain(|&c| c != id);
            }
            if let Some(node) = self.nodes.get_mut(id as usize) {
                node.parent = None;
            }
        }
    }

    /// Insert an already-validated, detached `child` under `parent`.
    fn attach(&mut self, parent: NodeId, child: NodeId, before: Option<NodeId>) {
        let siblings = &mut self.nodes[parent as usize].children;
        let index = before
            .and_then(|b| siblings.iter().position(|&c| c == b))
            .unwrap_or(siblings.len());
        siblings.insert(index, child);
        self.nodes[child as usize].parent = Some(parent);
    }

    fn validate_insert(
        &self,
        parent: NodeId,
        child: NodeId,
        before: Option<NodeId>,
    ) -> Result<(), TreeError> {
        let parent_node = self.check(parent)?;
        let child_node = self.check(child)?;
        if matches!(
            parent_node.kind,
            NodeKind::Text(_) | NodeKind::Comment(_) | NodeKind::Doctype { .. }
        ) {
            return Err(TreeError::HierarchyRequest("parent cannot have children"));
        }
        if matches!(child_node.kind, NodeKind::Document) {
            return Err(TreeError::HierarchyRequest("cannot insert a document"));
        }
        if self.is_inclusive_ancestor(child, parent) {
            return Err(TreeError::HierarchyRequest(
                "the new child is an ancestor of the parent",
            ));
        }
        if let Some(before) = before
            && self.parent(before) != Some(parent)
        {
            return Err(TreeError::NotFound(before));
        }
        Ok(())
    }

    /// `parent.appendChild(child)`: moves `child` if it is already in a tree.
    pub fn append(&mut self, parent: NodeId, child: NodeId) -> Result<(), TreeError> {
        self.insert_before(parent, child, None)
    }

    /// `parent.insertBefore(child, before)`; `None` appends.
    pub fn insert_before(
        &mut self,
        parent: NodeId,
        child: NodeId,
        before: Option<NodeId>,
    ) -> Result<(), TreeError> {
        self.validate_insert(parent, child, before)?;
        if before == Some(child) {
            return Ok(());
        }
        self.detach(child);
        self.attach(parent, child, before);
        self.touch();
        Ok(())
    }

    /// Detach `child` from its parent (it stays in the arena).
    pub fn remove(&mut self, child: NodeId) -> Result<(), TreeError> {
        self.check(child)?;
        if self.parent(child).is_some() {
            self.detach(child);
            self.touch();
        }
        Ok(())
    }

    /// `parent.replaceChild(new, old)`.
    pub fn replace(&mut self, parent: NodeId, new: NodeId, old: NodeId) -> Result<(), TreeError> {
        if self.parent(old) != Some(parent) {
            return Err(TreeError::NotFound(old));
        }
        if new == old {
            return Ok(());
        }
        self.validate_insert(parent, new, Some(old))?;
        self.detach(new);
        self.attach(parent, new, Some(old));
        self.detach(old);
        self.touch();
        Ok(())
    }

    /// Remove all children of `id`.
    pub fn clear_children(&mut self, id: NodeId) -> Result<(), TreeError> {
        let children = std::mem::take(&mut self.node_mut(id)?.children);
        for child in &children {
            self.nodes[*child as usize].parent = None;
        }
        if !children.is_empty() {
            self.touch();
        }
        Ok(())
    }

    // ----- attributes -----------------------------------------------------

    pub fn attrs(&self, id: NodeId) -> &[(String, String)] {
        match self.kind(id) {
            Some(NodeKind::Element { attrs, .. }) => attrs,
            _ => &[],
        }
    }

    pub fn attr(&self, id: NodeId, name: &str) -> Option<&str> {
        self.attrs(id)
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    pub fn set_attr(&mut self, id: NodeId, name: &str, value: &str) -> Result<(), TreeError> {
        let NodeKind::Element { attrs, .. } = &mut self.node_mut(id)?.kind else {
            return Err(TreeError::HierarchyRequest("not an element"));
        };
        match attrs.iter_mut().find(|(n, _)| n == name) {
            Some((_, v)) => *v = value.to_string(),
            None => attrs.push((name.to_string(), value.to_string())),
        }
        self.touch();
        Ok(())
    }

    /// Returns whether the attribute existed.
    pub fn remove_attr(&mut self, id: NodeId, name: &str) -> Result<bool, TreeError> {
        let NodeKind::Element { attrs, .. } = &mut self.node_mut(id)?.kind else {
            return Ok(false);
        };
        let before = attrs.len();
        attrs.retain(|(n, _)| n != name);
        let removed = attrs.len() != before;
        if removed {
            self.touch();
        }
        Ok(removed)
    }

    /// Whitespace-separated class list.
    pub fn classes(&self, id: NodeId) -> Vec<&str> {
        self.attr(id, "class")
            .map(|c| c.split_ascii_whitespace().collect())
            .unwrap_or_default()
    }

    // ----- text -----------------------------------------------------------

    /// Character data of a text or comment node.
    pub fn data(&self, id: NodeId) -> Option<&str> {
        match self.kind(id) {
            Some(NodeKind::Text(s) | NodeKind::Comment(s)) => Some(s),
            _ => None,
        }
    }

    pub fn set_data(&mut self, id: NodeId, data: &str) -> Result<(), TreeError> {
        match &mut self.node_mut(id)?.kind {
            NodeKind::Text(s) | NodeKind::Comment(s) => *s = data.to_string(),
            _ => return Err(TreeError::HierarchyRequest("not character data")),
        }
        self.touch();
        Ok(())
    }

    /// DOM `textContent` (concatenated descendant text; `None` for the
    /// document and doctypes).
    pub fn text_content(&self, id: NodeId) -> Option<String> {
        match self.kind(id)? {
            NodeKind::Document | NodeKind::Doctype { .. } => None,
            NodeKind::Text(s) | NodeKind::Comment(s) => Some(s.clone()),
            NodeKind::Element { .. } => {
                let mut out = String::new();
                for node in self.descendants(id) {
                    if let Some(NodeKind::Text(s)) = self.kind(node) {
                        out.push_str(s);
                    }
                }
                Some(out)
            }
        }
    }

    /// DOM `textContent = value`: replaces children with one text node.
    pub fn set_text_content(&mut self, id: NodeId, value: &str) -> Result<(), TreeError> {
        match self.check(id)?.kind {
            NodeKind::Text(_) | NodeKind::Comment(_) => self.set_data(id, value),
            NodeKind::Element { .. } => {
                self.clear_children(id)?;
                if !value.is_empty() {
                    let text = self.create_text(value);
                    self.attach(id, text, None);
                }
                self.touch();
                Ok(())
            }
            _ => Ok(()),
        }
    }

    // ----- markup ---------------------------------------------------------

    /// Parse `html` as a fragment into new detached nodes (in order).
    pub fn parse_fragment(&mut self, html: &str) -> Vec<NodeId> {
        parse::build_fragment(self, html)
    }

    /// `element.innerHTML = html`; returns the new children.
    pub fn set_inner_html(&mut self, id: NodeId, html: &str) -> Result<Vec<NodeId>, TreeError> {
        if !self.is_element(id) && id != self.document() {
            return Err(TreeError::HierarchyRequest("not an element"));
        }
        self.clear_children(id)?;
        let nodes = self.parse_fragment(html);
        for &node in &nodes {
            self.attach(id, node, None);
        }
        self.touch();
        Ok(nodes)
    }

    pub fn outer_html(&self, id: NodeId) -> String {
        serialize::serialize(self, id, true, false)
    }

    pub fn inner_html(&self, id: NodeId) -> String {
        serialize::serialize(self, id, false, false)
    }

    /// The whole document as HTML.
    pub fn to_html(&self) -> String {
        self.inner_html(self.document())
    }

    /// Like [`outer_html`](Self::outer_html), with every element stamped
    /// with its node id.
    pub fn serialize_with_ids(&self, id: NodeId) -> String {
        serialize::serialize(self, id, true, true)
    }

    // ----- lookups --------------------------------------------------------

    /// First connected element with `id="…"`, in document order.
    pub fn find_by_id(&self, element_id: &str) -> Option<NodeId> {
        if element_id.is_empty() {
            return None;
        }
        self.descendants(self.document())
            .into_iter()
            .find(|&n| self.attr(n, "id") == Some(element_id))
    }

    /// Descendant elements of `scope` (excluding it) with the given tag
    /// (`*` for all), case-insensitively.
    pub fn elements_by_tag(&self, scope: NodeId, tag: &str) -> Vec<NodeId> {
        self.descendants(scope)
            .into_iter()
            .skip(1)
            .filter(|&n| {
                self.tag(n)
                    .is_some_and(|t| tag == "*" || t.eq_ignore_ascii_case(tag))
            })
            .collect()
    }

    /// Descendant elements of `scope` having every class in `names`.
    pub fn elements_by_class(&self, scope: NodeId, names: &str) -> Vec<NodeId> {
        let wanted: Vec<&str> = names.split_ascii_whitespace().collect();
        if wanted.is_empty() {
            return Vec::new();
        }
        self.descendants(scope)
            .into_iter()
            .skip(1)
            .filter(|&n| {
                let classes = self.classes(n);
                self.is_element(n) && wanted.iter().all(|w| classes.contains(w))
            })
            .collect()
    }

    /// Descendant elements of `scope` with `name="…"`.
    pub fn elements_by_name(&self, scope: NodeId, name: &str) -> Vec<NodeId> {
        self.descendants(scope)
            .into_iter()
            .skip(1)
            .filter(|&n| self.attr(n, "name") == Some(name))
            .collect()
    }

    /// Selector path used for layout lookups (see [`path`]).
    pub fn css_path(&self, id: NodeId) -> Option<String> {
        path::css_path(self, id)
    }

    /// `scope.querySelectorAll(selector)` (descendants only, document order).
    pub fn select(&self, scope: NodeId, selector: &str) -> Result<Vec<NodeId>, SelectError> {
        select::select(self, scope, selector)
    }

    /// `scope.querySelector(selector)`.
    pub fn select_first(
        &self,
        scope: NodeId,
        selector: &str,
    ) -> Result<Option<NodeId>, SelectError> {
        Ok(self.select(scope, selector)?.into_iter().next())
    }

    /// `element.matches(selector)`.
    pub fn matches(&self, id: NodeId, selector: &str) -> Result<bool, SelectError> {
        select::matches(self, id, selector)
    }

    /// `element.closest(selector)`.
    pub fn closest(&self, id: NodeId, selector: &str) -> Result<Option<NodeId>, SelectError> {
        let mut current = Some(id);
        while let Some(node) = current {
            if self.is_element(node) && self.matches(node, selector)? {
                return Ok(Some(node));
            }
            current = self.parent(node);
        }
        Ok(None)
    }
}
