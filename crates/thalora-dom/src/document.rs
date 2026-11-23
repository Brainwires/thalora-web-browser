//! The arena-backed document: storage, navigation and mutation.

use std::cmp::Ordering;
use std::collections::HashMap;

use html5ever::{LocalName, Namespace, QualName, ns};

use crate::mutation::MutationRecord;
use crate::names::{self, NameContext};
use crate::node::{Attr, ElementData, Node, NodeData, NodeType, QuirksMode};
use crate::{DomError, DomResult, NodeId};

#[derive(Debug, Clone)]
struct Slot {
    generation: u32,
    node: Option<Node>,
}

/// A live, mutable DOM tree.
///
/// The document owns every node it ever created: the tree rooted at the
/// document node, detached nodes and subtrees, document fragments and
/// template contents. Nodes are addressed by [`NodeId`].
///
/// # Node lifetime
///
/// Detaching a node (`remove_child`, `replace_child`, re-parenting) never
/// frees it; the binding layer keeps it alive as long as JS can reach it and
/// then calls [`Document::free_subtree`] on the detached root. Freed slots
/// are reused through a free list, with a per-slot generation so stale
/// [`NodeId`]s are reported as [`DomError::StaleNode`]. Every node also has
/// a never-reused `u64` [serial](Document::serial) for wrapper caching.
#[derive(Debug, Clone)]
pub struct Document {
    slots: Vec<Slot>,
    free: Vec<u32>,
    next_serial: u64,
    root: u32,
    quirks_mode: QuirksMode,
    /// Connected elements by ID. Unordered; lookups pick the first in tree
    /// order.
    ids: HashMap<String, Vec<u32>>,
    recording: bool,
    records: Vec<MutationRecord>,
}

impl Default for Document {
    fn default() -> Self {
        Document::new()
    }
}

impl Document {
    /// An empty document: just the document node, no children.
    pub fn new() -> Document {
        let mut doc = Document {
            slots: Vec::new(),
            free: Vec::new(),
            next_serial: 1,
            root: 0,
            quirks_mode: QuirksMode::NoQuirks,
            ids: HashMap::new(),
            recording: false,
            records: Vec::new(),
        };
        doc.root = doc.alloc(NodeData::Document);
        doc
    }

    // ----- arena internals -------------------------------------------------

    pub(crate) fn alloc(&mut self, data: NodeData) -> u32 {
        let serial = self.next_serial;
        self.next_serial += 1;
        let node = Node {
            serial,
            parent: None,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
            host: None,
            data,
        };
        if let Some(index) = self.free.pop() {
            self.slots[index as usize].node = Some(node);
            index
        } else {
            let index = u32::try_from(self.slots.len()).expect("more than u32::MAX DOM nodes");
            self.slots.push(Slot {
                generation: 0,
                node: Some(node),
            });
            index
        }
    }

    /// Allocates an element; creates the template contents fragment for an
    /// HTML `<template>`.
    pub(crate) fn alloc_element(&mut self, name: QualName, attrs: Vec<Attr>) -> u32 {
        let is_template = name.ns == ns!(html) && &*name.local == "template";
        let index = self.alloc(NodeData::Element(ElementData::new(name, attrs)));
        if is_template {
            let contents = self.alloc(NodeData::DocumentFragment);
            self.node_mut(contents).host = Some(index);
            if let NodeData::Element(e) = &mut self.node_mut(index).data {
                e.template_contents = Some(contents);
            }
        }
        index
    }

    /// Internal access by index. Indices stored in tree links always name
    /// live nodes (an invariant of this module), so this does not fail for
    /// them; public entry points validate ids with [`Document::check`].
    pub(crate) fn node(&self, index: u32) -> &Node {
        self.slots[index as usize]
            .node
            .as_ref()
            .expect("thalora-dom invariant: tree link to a freed node")
    }

    pub(crate) fn node_mut(&mut self, index: u32) -> &mut Node {
        self.slots[index as usize]
            .node
            .as_mut()
            .expect("thalora-dom invariant: tree link to a freed node")
    }

    pub(crate) fn id_of(&self, index: u32) -> NodeId {
        NodeId {
            index,
            generation: self.slots[index as usize].generation,
        }
    }

    fn opt_id(&self, index: Option<u32>) -> Option<NodeId> {
        index.map(|i| self.id_of(i))
    }

    /// Validates a public id, returning its index.
    pub(crate) fn check(&self, id: NodeId) -> DomResult<u32> {
        match self.slots.get(id.index as usize) {
            Some(slot) if slot.generation == id.generation && slot.node.is_some() => Ok(id.index),
            _ => Err(DomError::StaleNode(id)),
        }
    }

    fn check_element(&self, id: NodeId) -> DomResult<u32> {
        let i = self.check(id)?;
        if self.node(i).data.is_element() {
            Ok(i)
        } else {
            Err(DomError::InvalidNodeType)
        }
    }

    pub(crate) fn element(&self, index: u32) -> Option<&ElementData> {
        self.node(index).data.as_element()
    }

    pub(crate) fn root_index(&self) -> u32 {
        self.root
    }

    pub(crate) fn set_quirks(&mut self, mode: QuirksMode) {
        self.quirks_mode = mode;
    }

    pub(crate) fn child_indices(&self, parent: u32) -> Vec<u32> {
        let mut out = Vec::new();
        let mut c = self.node(parent).first_child;
        while let Some(i) = c {
            out.push(i);
            c = self.node(i).next_sibling;
        }
        out
    }

    /// Pre-order successor of `cur` within the subtree of `root` (excluding
    /// `root` itself from being returned again).
    pub(crate) fn next_in_subtree(&self, cur: u32, root: u32) -> Option<u32> {
        if let Some(c) = self.node(cur).first_child {
            return Some(c);
        }
        let mut n = cur;
        loop {
            if n == root {
                return None;
            }
            if let Some(s) = self.node(n).next_sibling {
                return Some(s);
            }
            n = self.node(n).parent?;
        }
    }

    /// `root` and all its descendants, in tree order.
    fn inclusive_subtree(&self, root: u32) -> Vec<u32> {
        let mut out = vec![root];
        let mut cur = root;
        while let Some(n) = self.next_in_subtree(cur, root) {
            out.push(n);
            cur = n;
        }
        out
    }

    /// Links a parentless node as a child of `parent`, before `before`
    /// (append if `None`). No validation, records or index updates.
    pub(crate) fn link_before(&mut self, parent: u32, node: u32, before: Option<u32>) {
        debug_assert!(self.node(node).parent.is_none());
        let prev = match before {
            Some(b) => self.node(b).prev_sibling,
            None => self.node(parent).last_child,
        };
        {
            let n = self.node_mut(node);
            n.parent = Some(parent);
            n.prev_sibling = prev;
            n.next_sibling = before;
        }
        match prev {
            Some(p) => self.node_mut(p).next_sibling = Some(node),
            None => self.node_mut(parent).first_child = Some(node),
        }
        match before {
            Some(b) => self.node_mut(b).prev_sibling = Some(node),
            None => self.node_mut(parent).last_child = Some(node),
        }
    }

    /// Unlinks a node from its parent, if any. No records or index updates.
    pub(crate) fn unlink(&mut self, node: u32) {
        let (parent, prev, next) = {
            let n = self.node(node);
            match n.parent {
                Some(p) => (p, n.prev_sibling, n.next_sibling),
                None => return,
            }
        };
        match prev {
            Some(p) => self.node_mut(p).next_sibling = next,
            None => self.node_mut(parent).first_child = next,
        }
        match next {
            Some(n) => self.node_mut(n).prev_sibling = prev,
            None => self.node_mut(parent).last_child = prev,
        }
        let n = self.node_mut(node);
        n.parent = None;
        n.prev_sibling = None;
        n.next_sibling = None;
    }

    fn root_of_index(&self, mut i: u32) -> u32 {
        while let Some(p) = self.node(i).parent {
            i = p;
        }
        i
    }

    fn is_connected_index(&self, i: u32) -> bool {
        self.root_of_index(i) == self.root
    }

    fn queue(&mut self, record: impl FnOnce(&Document) -> MutationRecord) {
        if self.recording {
            let r = record(self);
            self.records.push(r);
        }
    }

    // ----- ID index ----------------------------------------------------------

    fn index_add(&mut self, id: &str, element: u32) {
        if !id.is_empty() {
            self.ids.entry(id.to_string()).or_default().push(element);
        }
    }

    fn index_remove(&mut self, id: &str, element: u32) {
        if let Some(list) = self.ids.get_mut(id) {
            if let Some(pos) = list.iter().position(|&e| e == element) {
                list.swap_remove(pos);
            }
            if list.is_empty() {
                self.ids.remove(id);
            }
        }
    }

    fn index_subtree(&mut self, root: u32, add: bool) {
        for n in self.inclusive_subtree(root) {
            let id = self.element(n).and_then(|e| e.id()).map(str::to_string);
            if let Some(id) = id {
                if add {
                    self.index_add(&id, n);
                } else {
                    self.index_remove(&id, n);
                }
            }
        }
    }

    /// Rebuilds the ID index from scratch (after parsing).
    pub(crate) fn rebuild_id_index(&mut self) {
        self.ids.clear();
        self.index_subtree(self.root, true);
    }

    /// Tree-order comparison of two nodes in the same tree.
    fn tree_order(&self, a: u32, b: u32) -> Ordering {
        if a == b {
            return Ordering::Equal;
        }
        let path = |mut n: u32| {
            let mut p = vec![n];
            while let Some(parent) = self.node(n).parent {
                p.push(parent);
                n = parent;
            }
            p.reverse();
            p
        };
        let (pa, pb) = (path(a), path(b));
        let common = pa.iter().zip(&pb).take_while(|(x, y)| x == y).count();
        match (pa.get(common), pb.get(common)) {
            (None, _) => Ordering::Less,
            (_, None) => Ordering::Greater,
            (Some(&x), Some(&y)) => {
                let mut s = self.node(x).next_sibling;
                while let Some(i) = s {
                    if i == y {
                        return Ordering::Less;
                    }
                    s = self.node(i).next_sibling;
                }
                Ordering::Greater
            }
        }
    }

    // ----- identity and reading ---------------------------------------------

    /// The document node.
    pub fn document(&self) -> NodeId {
        self.id_of(self.root)
    }

    /// Whether `id` names a live node of this document.
    pub fn is_valid(&self, id: NodeId) -> bool {
        self.check(id).is_ok()
    }

    /// The node's never-reused serial number (stable for the node's whole
    /// life; use it to key JS wrapper caches).
    pub fn serial(&self, id: NodeId) -> DomResult<u64> {
        Ok(self.node(self.check(id)?).serial)
    }

    /// The node's kind and data.
    pub fn data(&self, id: NodeId) -> DomResult<&NodeData> {
        Ok(&self.node(self.check(id)?).data)
    }

    /// The node's DOM `nodeType`.
    pub fn node_type(&self, id: NodeId) -> DomResult<NodeType> {
        Ok(self.data(id)?.node_type())
    }

    /// The DOM `nodeName`.
    pub fn node_name(&self, id: NodeId) -> DomResult<String> {
        Ok(match self.data(id)? {
            NodeData::Document => "#document".into(),
            NodeData::Doctype { name, .. } => name.clone(),
            NodeData::Element(e) => e.tag_name(),
            NodeData::Text(_) => "#text".into(),
            NodeData::Comment(_) => "#comment".into(),
            NodeData::ProcessingInstruction { target, .. } => target.clone(),
            NodeData::DocumentFragment => "#document-fragment".into(),
        })
    }

    /// The element data of an element node.
    pub fn element_data(&self, id: NodeId) -> DomResult<&ElementData> {
        self.data(id)?.as_element().ok_or(DomError::InvalidNodeType)
    }

    /// The quirks mode the parser determined.
    pub fn quirks_mode(&self) -> QuirksMode {
        self.quirks_mode
    }

    // ----- navigation ---------------------------------------------------------

    /// The parent node.
    pub fn parent(&self, id: NodeId) -> DomResult<Option<NodeId>> {
        Ok(self.opt_id(self.node(self.check(id)?).parent))
    }

    /// The parent, if it is an element.
    pub fn parent_element(&self, id: NodeId) -> DomResult<Option<NodeId>> {
        let p = self.node(self.check(id)?).parent;
        Ok(p.filter(|&p| self.node(p).data.is_element())
            .map(|p| self.id_of(p)))
    }

    /// The first child.
    pub fn first_child(&self, id: NodeId) -> DomResult<Option<NodeId>> {
        Ok(self.opt_id(self.node(self.check(id)?).first_child))
    }

    /// The last child.
    pub fn last_child(&self, id: NodeId) -> DomResult<Option<NodeId>> {
        Ok(self.opt_id(self.node(self.check(id)?).last_child))
    }

    /// The previous sibling.
    pub fn previous_sibling(&self, id: NodeId) -> DomResult<Option<NodeId>> {
        Ok(self.opt_id(self.node(self.check(id)?).prev_sibling))
    }

    /// The next sibling.
    pub fn next_sibling(&self, id: NodeId) -> DomResult<Option<NodeId>> {
        Ok(self.opt_id(self.node(self.check(id)?).next_sibling))
    }

    /// The children, in order.
    pub fn children(&self, id: NodeId) -> DomResult<Children<'_>> {
        let i = self.check(id)?;
        Ok(Children {
            doc: self,
            next: self.node(i).first_child,
        })
    }

    /// The element children, in order (`ParentNode.children`).
    pub fn element_children(&self, id: NodeId) -> DomResult<impl Iterator<Item = NodeId> + '_> {
        Ok(self
            .children(id)?
            .filter(|&c| self.node(c.index).data.is_element()))
    }

    /// The descendants in tree order, excluding `id` itself. Template
    /// contents are not descendants of their template.
    pub fn descendants(&self, id: NodeId) -> DomResult<Descendants<'_>> {
        let i = self.check(id)?;
        Ok(Descendants {
            doc: self,
            root: i,
            next: self.node(i).first_child,
        })
    }

    /// The ancestors, nearest first, excluding `id` itself.
    pub fn ancestors(&self, id: NodeId) -> DomResult<Ancestors<'_>> {
        let i = self.check(id)?;
        Ok(Ancestors {
            doc: self,
            next: self.node(i).parent,
        })
    }

    /// The root of the node's tree (the document node if connected).
    pub fn root_of(&self, id: NodeId) -> DomResult<NodeId> {
        Ok(self.id_of(self.root_of_index(self.check(id)?)))
    }

    /// Whether the node is in the document tree (`isConnected`).
    pub fn is_connected(&self, id: NodeId) -> DomResult<bool> {
        Ok(self.is_connected_index(self.check(id)?))
    }

    /// Whether `other` is an inclusive descendant of `id` (`Node.contains`).
    pub fn contains(&self, id: NodeId, other: NodeId) -> DomResult<bool> {
        let a = self.check(id)?;
        let mut n = Some(self.check(other)?);
        while let Some(i) = n {
            if i == a {
                return Ok(true);
            }
            n = self.node(i).parent;
        }
        Ok(false)
    }

    /// The template contents fragment of an HTML `<template>`, `None` for
    /// other elements.
    pub fn template_contents(&self, id: NodeId) -> DomResult<Option<NodeId>> {
        let e = self.element_data(id)?;
        Ok(self.opt_id(e.template_contents))
    }

    /// The document's doctype child, if any.
    pub fn doctype(&self) -> Option<NodeId> {
        self.child_indices(self.root)
            .into_iter()
            .find(|&c| self.node(c).data.is_doctype())
            .map(|c| self.id_of(c))
    }

    /// The document element (`<html>`), if any.
    pub fn document_element(&self) -> Option<NodeId> {
        self.child_indices(self.root)
            .into_iter()
            .find(|&c| self.node(c).data.is_element())
            .map(|c| self.id_of(c))
    }

    fn html_child_of_root(&self, local: &str) -> Option<NodeId> {
        let html = self.document_element()?;
        if !self.element(html.index)?.is_html_named("html") {
            return None;
        }
        self.child_indices(html.index)
            .into_iter()
            .find(|&c| self.element(c).is_some_and(|e| e.is_html_named(local)))
            .map(|c| self.id_of(c))
    }

    /// The `<head>` element (first `head` child of the `<html>` root).
    pub fn head(&self) -> Option<NodeId> {
        self.html_child_of_root("head")
    }

    /// The `<body>` element (first `body` or `frameset` child of the
    /// `<html>` root, whichever comes first).
    pub fn body(&self) -> Option<NodeId> {
        let html = self.document_element()?;
        if !self.element(html.index)?.is_html_named("html") {
            return None;
        }
        self.child_indices(html.index)
            .into_iter()
            .find(|&c| {
                self.element(c)
                    .is_some_and(|e| e.is_html_named("body") || e.is_html_named("frameset"))
            })
            .map(|c| self.id_of(c))
    }

    /// `document.getElementById`: the first connected element, in tree
    /// order, whose ID is `id`. Kept correct under every mutation.
    pub fn get_element_by_id(&self, id: &str) -> Option<NodeId> {
        let list = self.ids.get(id)?;
        let first = list
            .iter()
            .copied()
            .min_by(|&a, &b| self.tree_order(a, b))?;
        Some(self.id_of(first))
    }

    /// The element's class list (`classList` tokens): the `class` attribute
    /// split on ASCII whitespace, duplicates removed, in order.
    pub fn class_list(&self, id: NodeId) -> DomResult<impl Iterator<Item = &str> + '_> {
        let e = self.element_data(id)?;
        let mut tokens: Vec<&str> = Vec::new();
        for t in e.attr("class").unwrap_or("").split_ascii_whitespace() {
            if !tokens.contains(&t) {
                tokens.push(t);
            }
        }
        Ok(tokens.into_iter())
    }

    /// `textContent`: the concatenated text of descendant text nodes for
    /// elements and fragments, the data for character data nodes, `None`
    /// for the document and doctypes.
    pub fn text_content(&self, id: NodeId) -> DomResult<Option<String>> {
        let i = self.check(id)?;
        Ok(match &self.node(i).data {
            NodeData::Document | NodeData::Doctype { .. } => None,
            NodeData::Text(s) | NodeData::Comment(s) => Some(s.clone()),
            NodeData::ProcessingInstruction { data, .. } => Some(data.clone()),
            NodeData::Element(_) | NodeData::DocumentFragment => {
                let mut out = String::new();
                let mut cur = i;
                while let Some(n) = self.next_in_subtree(cur, i) {
                    if let NodeData::Text(s) = &self.node(n).data {
                        out.push_str(s);
                    }
                    cur = n;
                }
                Some(out)
            }
        })
    }

    // ----- node creation ------------------------------------------------------

    /// `document.createElement`: an HTML element; the name is validated and
    /// ASCII-lowercased.
    pub fn create_element(&mut self, local_name: &str) -> DomResult<NodeId> {
        if !names::is_valid_element_local_name(local_name) {
            return Err(DomError::InvalidCharacter);
        }
        let name = QualName::new(
            None,
            ns!(html),
            LocalName::from(local_name.to_ascii_lowercase()),
        );
        let i = self.alloc_element(name, Vec::new());
        Ok(self.id_of(i))
    }

    /// `document.createElementNS`.
    pub fn create_element_ns(
        &mut self,
        namespace: Option<&str>,
        qualified_name: &str,
    ) -> DomResult<NodeId> {
        let name = names::validate_and_extract(namespace, qualified_name, NameContext::Element)?;
        let i = self.alloc_element(name, Vec::new());
        Ok(self.id_of(i))
    }

    /// `document.createTextNode`.
    pub fn create_text_node(&mut self, data: &str) -> NodeId {
        let i = self.alloc(NodeData::Text(data.to_string()));
        self.id_of(i)
    }

    /// `document.createComment`.
    pub fn create_comment(&mut self, data: &str) -> NodeId {
        let i = self.alloc(NodeData::Comment(data.to_string()));
        self.id_of(i)
    }

    /// `document.createProcessingInstruction`.
    pub fn create_processing_instruction(&mut self, target: &str, data: &str) -> DomResult<NodeId> {
        if !names::is_valid_element_local_name(target) || data.contains("?>") {
            return Err(DomError::InvalidCharacter);
        }
        let i = self.alloc(NodeData::ProcessingInstruction {
            target: target.to_string(),
            data: data.to_string(),
        });
        Ok(self.id_of(i))
    }

    /// `document.createDocumentFragment`.
    pub fn create_document_fragment(&mut self) -> NodeId {
        let i = self.alloc(NodeData::DocumentFragment);
        self.id_of(i)
    }

    /// `document.implementation.createDocumentType` (detached doctype).
    pub fn create_doctype(
        &mut self,
        name: &str,
        public_id: &str,
        system_id: &str,
    ) -> DomResult<NodeId> {
        if !names::is_valid_doctype_name(name) {
            return Err(DomError::InvalidCharacter);
        }
        let i = self.alloc(NodeData::Doctype {
            name: name.to_string(),
            public_id: public_id.to_string(),
            system_id: system_id.to_string(),
        });
        Ok(self.id_of(i))
    }

    // ----- tree mutation --------------------------------------------------------

    fn is_host_including_inclusive_ancestor(&self, ancestor: u32, mut n: u32) -> bool {
        loop {
            if n == ancestor {
                return true;
            }
            let node = self.node(n);
            match node.parent.or(node.host) {
                Some(p) => n = p,
                None => return false,
            }
        }
    }

    fn has_element_child(&self, parent: u32, except: Option<u32>) -> bool {
        self.child_indices(parent)
            .into_iter()
            .any(|c| Some(c) != except && self.node(c).data.is_element())
    }

    fn has_doctype_child(&self, parent: u32, except: Option<u32>) -> bool {
        self.child_indices(parent)
            .into_iter()
            .any(|c| Some(c) != except && self.node(c).data.is_doctype())
    }

    fn doctype_following(&self, child: u32) -> bool {
        let mut s = self.node(child).next_sibling;
        while let Some(i) = s {
            if self.node(i).data.is_doctype() {
                return true;
            }
            s = self.node(i).next_sibling;
        }
        false
    }

    fn element_preceding(&self, child: u32) -> bool {
        let mut s = self.node(child).prev_sibling;
        while let Some(i) = s {
            if self.node(i).data.is_element() {
                return true;
            }
            s = self.node(i).prev_sibling;
        }
        false
    }

    /// Steps 1-5 shared by pre-insertion and replace validity.
    fn common_validity(&self, node: u32, parent: u32, child: Option<u32>) -> DomResult<()> {
        let pdata = &self.node(parent).data;
        if !matches!(
            pdata,
            NodeData::Document | NodeData::DocumentFragment | NodeData::Element(_)
        ) {
            return Err(DomError::HierarchyRequest);
        }
        if self.is_host_including_inclusive_ancestor(node, parent) {
            return Err(DomError::HierarchyRequest);
        }
        if let Some(c) = child
            && self.node(c).parent != Some(parent)
        {
            return Err(DomError::NotFound);
        }
        let parent_is_doc = matches!(pdata, NodeData::Document);
        match &self.node(node).data {
            NodeData::Document => Err(DomError::HierarchyRequest),
            NodeData::Text(_) if parent_is_doc => Err(DomError::HierarchyRequest),
            NodeData::Doctype { .. } if !parent_is_doc => Err(DomError::HierarchyRequest),
            _ => Ok(()),
        }
    }

    /// <https://dom.spec.whatwg.org/#concept-node-ensure-pre-insertion-validity>
    fn ensure_pre_insertion_validity(
        &self,
        node: u32,
        parent: u32,
        child: Option<u32>,
    ) -> DomResult<()> {
        self.common_validity(node, parent, child)?;
        if !matches!(self.node(parent).data, NodeData::Document) {
            return Ok(());
        }
        let child_is_doctype = child.is_some_and(|c| self.node(c).data.is_doctype());
        let doctype_after = child.is_some_and(|c| self.doctype_following(c));
        let bad = match &self.node(node).data {
            NodeData::DocumentFragment => {
                let kids = self.child_indices(node);
                let elements = kids
                    .iter()
                    .filter(|&&k| self.node(k).data.is_element())
                    .count();
                let has_text = kids.iter().any(|&k| self.node(k).data.is_text());
                elements > 1
                    || has_text
                    || (elements == 1
                        && (self.has_element_child(parent, None)
                            || child_is_doctype
                            || doctype_after))
            }
            NodeData::Element(_) => {
                self.has_element_child(parent, None) || child_is_doctype || doctype_after
            }
            NodeData::Doctype { .. } => {
                self.has_doctype_child(parent, None)
                    || child.is_some_and(|c| self.element_preceding(c))
                    || (child.is_none() && self.has_element_child(parent, None))
            }
            _ => false,
        };
        if bad {
            Err(DomError::HierarchyRequest)
        } else {
            Ok(())
        }
    }

    /// <https://dom.spec.whatwg.org/#concept-node-replace> steps 1-6.
    fn ensure_replace_validity(&self, node: u32, parent: u32, child: u32) -> DomResult<()> {
        self.common_validity(node, parent, Some(child))?;
        if !matches!(self.node(parent).data, NodeData::Document) {
            return Ok(());
        }
        let doctype_after = self.doctype_following(child);
        let bad = match &self.node(node).data {
            NodeData::DocumentFragment => {
                let kids = self.child_indices(node);
                let elements = kids
                    .iter()
                    .filter(|&&k| self.node(k).data.is_element())
                    .count();
                let has_text = kids.iter().any(|&k| self.node(k).data.is_text());
                elements > 1
                    || has_text
                    || (elements == 1
                        && (self.has_element_child(parent, Some(child)) || doctype_after))
            }
            NodeData::Element(_) => self.has_element_child(parent, Some(child)) || doctype_after,
            NodeData::Doctype { .. } => {
                self.has_doctype_child(parent, Some(child)) || self.element_preceding(child)
            }
            _ => false,
        };
        if bad {
            Err(DomError::HierarchyRequest)
        } else {
            Ok(())
        }
    }

    /// <https://dom.spec.whatwg.org/#concept-node-remove>
    fn remove_node(&mut self, node: u32, suppress_observers: bool) {
        let Some(parent) = self.node(node).parent else {
            return;
        };
        let prev = self.node(node).prev_sibling;
        let next = self.node(node).next_sibling;
        if self.is_connected_index(parent) {
            self.index_subtree(node, false);
        }
        self.unlink(node);
        if !suppress_observers {
            self.queue(|d| {
                MutationRecord::child_list(
                    d.id_of(parent),
                    Vec::new(),
                    vec![d.id_of(node)],
                    d.opt_id(prev),
                    d.opt_id(next),
                )
            });
        }
    }

    /// <https://dom.spec.whatwg.org/#concept-node-insert>
    ///
    /// `record_fragment` controls the record queued on a fragment whose
    /// children are moved out; internal fragments (from `set_inner_html`)
    /// skip it since nothing can observe them.
    fn insert(
        &mut self,
        node: u32,
        parent: u32,
        child: Option<u32>,
        suppress_observers: bool,
        record_fragment: bool,
    ) {
        let is_fragment = matches!(self.node(node).data, NodeData::DocumentFragment);
        let nodes = if is_fragment {
            self.child_indices(node)
        } else {
            vec![node]
        };
        if nodes.is_empty() {
            return;
        }
        if is_fragment {
            for &n in &nodes {
                self.remove_node(n, true);
            }
            if record_fragment {
                self.queue(|d| {
                    MutationRecord::child_list(
                        d.id_of(node),
                        Vec::new(),
                        nodes.iter().map(|&n| d.id_of(n)).collect(),
                        None,
                        None,
                    )
                });
            }
        }
        let previous_sibling = match child {
            Some(c) => self.node(c).prev_sibling,
            None => self.node(parent).last_child,
        };
        let connected = self.is_connected_index(parent);
        for &n in &nodes {
            // "Adopt": a node with a parent is removed from it (observably).
            self.remove_node(n, false);
            self.link_before(parent, n, child);
            if connected {
                self.index_subtree(n, true);
            }
        }
        if !suppress_observers {
            self.queue(|d| {
                MutationRecord::child_list(
                    d.id_of(parent),
                    nodes.iter().map(|&n| d.id_of(n)).collect(),
                    Vec::new(),
                    d.opt_id(previous_sibling),
                    d.opt_id(child),
                )
            });
        }
    }

    /// <https://dom.spec.whatwg.org/#concept-node-replace-all>
    pub(crate) fn replace_all(&mut self, node: Option<u32>, parent: u32, record_fragment: bool) {
        let removed = self.child_indices(parent);
        let added = match node {
            Some(n) if matches!(self.node(n).data, NodeData::DocumentFragment) => {
                self.child_indices(n)
            }
            Some(n) => vec![n],
            None => Vec::new(),
        };
        for &r in &removed {
            self.remove_node(r, true);
        }
        if let Some(n) = node {
            self.insert(n, parent, None, true, record_fragment);
        }
        if !added.is_empty() || !removed.is_empty() {
            self.queue(|d| {
                MutationRecord::child_list(
                    d.id_of(parent),
                    added.iter().map(|&n| d.id_of(n)).collect(),
                    removed.iter().map(|&n| d.id_of(n)).collect(),
                    None,
                    None,
                )
            });
        }
    }

    /// `parent.insertBefore(node, child)`. Returns `node`.
    pub fn insert_before(
        &mut self,
        parent: NodeId,
        node: NodeId,
        child: Option<NodeId>,
    ) -> DomResult<NodeId> {
        let p = self.check(parent)?;
        let n = self.check(node)?;
        let c = child.map(|c| self.check(c)).transpose()?;
        self.ensure_pre_insertion_validity(n, p, c)?;
        let reference = if c == Some(n) {
            self.node(n).next_sibling
        } else {
            c
        };
        self.insert(n, p, reference, false, true);
        Ok(node)
    }

    /// `parent.appendChild(node)`. Returns `node`.
    pub fn append_child(&mut self, parent: NodeId, node: NodeId) -> DomResult<NodeId> {
        self.insert_before(parent, node, None)
    }

    /// `parent.removeChild(child)`. Returns `child` (now detached, still
    /// valid).
    pub fn remove_child(&mut self, parent: NodeId, child: NodeId) -> DomResult<NodeId> {
        let p = self.check(parent)?;
        let c = self.check(child)?;
        if self.node(c).parent != Some(p) {
            return Err(DomError::NotFound);
        }
        self.remove_node(c, false);
        Ok(child)
    }

    /// `ChildNode.remove()`: detaches the node if it has a parent.
    pub fn remove(&mut self, node: NodeId) -> DomResult<()> {
        let n = self.check(node)?;
        self.remove_node(n, false);
        Ok(())
    }

    /// `parent.replaceChild(node, child)`. Returns `child`.
    pub fn replace_child(
        &mut self,
        parent: NodeId,
        node: NodeId,
        child: NodeId,
    ) -> DomResult<NodeId> {
        let p = self.check(parent)?;
        let n = self.check(node)?;
        let c = self.check(child)?;
        self.ensure_replace_validity(n, p, c)?;
        let mut reference = self.node(c).next_sibling;
        if reference == Some(n) {
            reference = self.node(n).next_sibling;
        }
        let mut previous_sibling = self.node(c).prev_sibling;
        if previous_sibling == Some(n) {
            previous_sibling = self.node(n).prev_sibling;
        }
        let is_fragment = matches!(self.node(n).data, NodeData::DocumentFragment);
        // Adopt: remove node from its old parent first (observably), unless
        // it is the child being replaced.
        if n != c && !is_fragment {
            self.remove_node(n, false);
        }
        let removed = if self.node(c).parent.is_some() {
            self.remove_node(c, true);
            vec![c]
        } else {
            Vec::new()
        };
        let nodes = if is_fragment {
            self.child_indices(n)
        } else {
            vec![n]
        };
        self.insert(n, p, reference, true, true);
        self.queue(|d| {
            MutationRecord::child_list(
                d.id_of(p),
                nodes.iter().map(|&x| d.id_of(x)).collect(),
                removed.iter().map(|&x| d.id_of(x)).collect(),
                d.opt_id(previous_sibling),
                d.opt_id(reference),
            )
        });
        Ok(child)
    }

    // ----- character data and text ----------------------------------------------

    /// Replaces the data of a text, comment or processing instruction node.
    pub fn set_data(&mut self, id: NodeId, data: &str) -> DomResult<()> {
        let i = self.check(id)?;
        let slot = match &mut self.node_mut(i).data {
            NodeData::Text(s) | NodeData::Comment(s) => s,
            NodeData::ProcessingInstruction { data, .. } => data,
            _ => return Err(DomError::InvalidNodeType),
        };
        let old = std::mem::replace(slot, data.to_string());
        self.queue(|d| MutationRecord::character_data(d.id_of(i), old));
        Ok(())
    }

    /// The `textContent` setter: replaces an element's or fragment's
    /// children with one text node (none for the empty string); sets the
    /// data of character data nodes; does nothing for the document and
    /// doctypes.
    pub fn set_text_content(&mut self, id: NodeId, text: &str) -> DomResult<()> {
        let i = self.check(id)?;
        match &self.node(i).data {
            NodeData::Element(_) | NodeData::DocumentFragment => {
                let node = (!text.is_empty()).then(|| self.alloc(NodeData::Text(text.into())));
                self.replace_all(node, i, true);
                Ok(())
            }
            NodeData::Text(_) | NodeData::Comment(_) | NodeData::ProcessingInstruction { .. } => {
                self.set_data(id, text)
            }
            NodeData::Document | NodeData::Doctype { .. } => Ok(()),
        }
    }

    // ----- attributes --------------------------------------------------------------

    fn html_lower<'a>(&self, element: u32, name: &'a str) -> std::borrow::Cow<'a, str> {
        if self.element(element).is_some_and(ElementData::is_html)
            && name.bytes().any(|b| b.is_ascii_uppercase())
        {
            name.to_ascii_lowercase().into()
        } else {
            name.into()
        }
    }

    fn attrs_mut(&mut self, element: u32) -> &mut Vec<Attr> {
        match &mut self.node_mut(element).data {
            NodeData::Element(e) => &mut e.attrs,
            _ => unreachable!("attrs_mut on a checked element"),
        }
    }

    /// Bookkeeping after an attribute change: mutation record and ID index.
    fn attribute_changed(
        &mut self,
        element: u32,
        name: &QualName,
        old: Option<String>,
        new: Option<&str>,
    ) {
        self.queue(|d| {
            MutationRecord::attribute(d.id_of(element), &name.local, &name.ns, old.clone())
        });
        if name.ns.is_empty() && &*name.local == "id" && self.is_connected_index(element) {
            if let Some(o) = &old {
                self.index_remove(o, element);
            }
            if let Some(n) = new {
                self.index_add(n, element);
            }
        }
    }

    /// The element's attributes, in order.
    pub fn attributes(&self, element: NodeId) -> DomResult<&[Attr]> {
        Ok(self.element_data(element)?.attrs())
    }

    /// `getAttribute(qualifiedName)` (ASCII-lowercased for HTML elements).
    pub fn get_attribute(&self, element: NodeId, name: &str) -> DomResult<Option<&str>> {
        let i = self.check_element(element)?;
        let name = self.html_lower(i, name);
        Ok(self
            .element(i)
            .and_then(|e| e.attrs.iter().find(|a| a.has_qualified_name(&name)))
            .map(|a| a.value.as_str()))
    }

    /// `getAttributeNS(namespace, localName)`.
    pub fn get_attribute_ns(
        &self,
        element: NodeId,
        namespace: Option<&str>,
        local_name: &str,
    ) -> DomResult<Option<&str>> {
        let e = self.element_data(element)?;
        let ns = namespace.unwrap_or("");
        Ok(e.attrs
            .iter()
            .find(|a| &*a.name.ns == ns && &*a.name.local == local_name)
            .map(|a| a.value.as_str()))
    }

    /// `hasAttribute(qualifiedName)`.
    pub fn has_attribute(&self, element: NodeId, name: &str) -> DomResult<bool> {
        Ok(self.get_attribute(element, name)?.is_some())
    }

    /// `setAttribute(qualifiedName, value)`.
    pub fn set_attribute(&mut self, element: NodeId, name: &str, value: &str) -> DomResult<()> {
        let i = self.check_element(element)?;
        if !names::is_valid_attribute_local_name(name) {
            return Err(DomError::InvalidCharacter);
        }
        let name = self.html_lower(i, name).into_owned();
        let attrs = self.attrs_mut(i);
        let (qname, old) = match attrs.iter_mut().find(|a| a.has_qualified_name(&name)) {
            Some(a) => (
                a.name.clone(),
                Some(std::mem::replace(&mut a.value, value.to_string())),
            ),
            None => {
                let qname = QualName::new(None, ns!(), LocalName::from(name.as_str()));
                attrs.push(Attr {
                    name: qname.clone(),
                    value: value.to_string(),
                });
                (qname, None)
            }
        };
        self.attribute_changed(i, &qname, old, Some(value));
        Ok(())
    }

    /// `setAttributeNS(namespace, qualifiedName, value)`.
    pub fn set_attribute_ns(
        &mut self,
        element: NodeId,
        namespace: Option<&str>,
        qualified_name: &str,
        value: &str,
    ) -> DomResult<()> {
        let i = self.check_element(element)?;
        let qname = names::validate_and_extract(namespace, qualified_name, NameContext::Attribute)?;
        let attrs = self.attrs_mut(i);
        let old = match attrs
            .iter_mut()
            .find(|a| a.name.ns == qname.ns && a.name.local == qname.local)
        {
            Some(a) => Some(std::mem::replace(&mut a.value, value.to_string())),
            None => {
                attrs.push(Attr {
                    name: qname.clone(),
                    value: value.to_string(),
                });
                None
            }
        };
        self.attribute_changed(i, &qname, old, Some(value));
        Ok(())
    }

    fn remove_attribute_at(&mut self, element: u32, pos: usize) {
        let attr = self.attrs_mut(element).remove(pos);
        self.attribute_changed(element, &attr.name, Some(attr.value), None);
    }

    /// `removeAttribute(qualifiedName)`. Returns whether one was removed.
    pub fn remove_attribute(&mut self, element: NodeId, name: &str) -> DomResult<bool> {
        let i = self.check_element(element)?;
        let name = self.html_lower(i, name).into_owned();
        let pos = self
            .element(i)
            .and_then(|e| e.attrs.iter().position(|a| a.has_qualified_name(&name)));
        if let Some(pos) = pos {
            self.remove_attribute_at(i, pos);
        }
        Ok(pos.is_some())
    }

    /// `removeAttributeNS(namespace, localName)`. Returns whether one was
    /// removed.
    pub fn remove_attribute_ns(
        &mut self,
        element: NodeId,
        namespace: Option<&str>,
        local_name: &str,
    ) -> DomResult<bool> {
        let i = self.check_element(element)?;
        let ns = Namespace::from(namespace.unwrap_or(""));
        let pos = self.element(i).and_then(|e| {
            e.attrs
                .iter()
                .position(|a| a.name.ns == ns && &*a.name.local == local_name)
        });
        if let Some(pos) = pos {
            self.remove_attribute_at(i, pos);
        }
        Ok(pos.is_some())
    }

    // ----- cloning and freeing ------------------------------------------------------

    /// Copies a node's own data for cloning (no links, no template contents).
    fn shallow_copy(data: &NodeData) -> NodeData {
        match data {
            NodeData::Element(e) => {
                NodeData::Element(ElementData::new(e.name.clone(), e.attrs.clone()))
            }
            other => other.clone(),
        }
    }

    /// Copies node `src` of `from` (which may be `self`) and, if `deep`, its
    /// descendants and template contents, into `self` as a detached subtree.
    pub(crate) fn copy_subtree(&mut self, from: Option<&Document>, src: u32, deep: bool) -> u32 {
        let read = |d: &Document, i: u32| Document::shallow_copy(&d.node(i).data);
        let root_data = read(from.unwrap_or(self), src);
        let root = self.alloc_copy(root_data);
        // Work list: (source node, destination node) whose children (and
        // template contents) still need copying.
        let mut work = vec![(src, root)];
        while let Some((s, d)) = work.pop() {
            let (kids, contents) = {
                let sd = from.unwrap_or(self);
                (
                    if deep {
                        sd.child_indices(s)
                    } else {
                        Vec::new()
                    },
                    sd.element(s).and_then(|e| e.template_contents),
                )
            };
            if deep
                && let (Some(sc), Some(dc)) =
                    (contents, self.element(d).and_then(|e| e.template_contents))
            {
                work.push((sc, dc));
            }
            for k in kids {
                let data = read(from.unwrap_or(self), k);
                let copy = self.alloc_copy(data);
                self.link_before(d, copy, None);
                work.push((k, copy));
            }
        }
        root
    }

    fn alloc_copy(&mut self, data: NodeData) -> u32 {
        match data {
            NodeData::Element(e) => self.alloc_element(e.name, e.attrs),
            other => self.alloc(other),
        }
    }

    /// `cloneNode(deep)`: a detached copy. Cloning the document node is not
    /// supported (one document per arena).
    pub fn clone_node(&mut self, id: NodeId, deep: bool) -> DomResult<NodeId> {
        let i = self.check(id)?;
        if matches!(self.node(i).data, NodeData::Document) {
            return Err(DomError::NotSupported);
        }
        let copy = self.copy_subtree(None, i, deep);
        Ok(self.id_of(copy))
    }

    /// Frees a detached subtree: `root` (which must have no parent) and all
    /// its descendants and template contents. Their ids become stale.
    ///
    /// The binding calls this when no JS wrapper can reach any node of the
    /// subtree. Errors: [`DomError::HierarchyRequest`] if `root` has a
    /// parent; [`DomError::NotSupported`] for the document node or a
    /// template contents fragment (freed with its template).
    pub fn free_subtree(&mut self, root: NodeId) -> DomResult<()> {
        let r = self.check(root)?;
        let node = self.node(r);
        if r == self.root || node.host.is_some() {
            return Err(DomError::NotSupported);
        }
        if node.parent.is_some() {
            return Err(DomError::HierarchyRequest);
        }
        self.free_detached(r);
        Ok(())
    }

    pub(crate) fn free_detached(&mut self, r: u32) {
        let mut stack = vec![r];
        while let Some(top) = stack.pop() {
            for n in self.inclusive_subtree(top) {
                if let Some(c) = self.element(n).and_then(|e| e.template_contents) {
                    stack.push(c);
                }
                let slot = &mut self.slots[n as usize];
                slot.node = None;
                slot.generation = slot.generation.wrapping_add(1);
                // A slot whose generation wrapped is retired, never reused.
                if slot.generation != 0 {
                    self.free.push(n);
                }
            }
        }
    }

    /// Number of live nodes in the arena (attached or not).
    pub fn live_node_count(&self) -> usize {
        self.slots.iter().filter(|s| s.node.is_some()).count()
    }

    // ----- mutation records -----------------------------------------------------------

    /// Turns mutation recording on or off. Off by default.
    pub fn set_mutation_recording(&mut self, enabled: bool) {
        self.recording = enabled;
    }

    /// Whether mutation recording is on.
    pub fn is_recording_mutations(&self) -> bool {
        self.recording
    }

    /// Drains the recorded mutations, oldest first.
    pub fn take_mutation_records(&mut self) -> Vec<MutationRecord> {
        std::mem::take(&mut self.records)
    }
}

/// Iterator over a node's children. See [`Document::children`].
#[derive(Debug, Clone)]
pub struct Children<'a> {
    doc: &'a Document,
    next: Option<u32>,
}

impl Iterator for Children<'_> {
    type Item = NodeId;
    fn next(&mut self) -> Option<NodeId> {
        let i = self.next?;
        self.next = self.doc.node(i).next_sibling;
        Some(self.doc.id_of(i))
    }
}

/// Iterator over a node's descendants in tree order. See
/// [`Document::descendants`].
#[derive(Debug, Clone)]
pub struct Descendants<'a> {
    doc: &'a Document,
    root: u32,
    next: Option<u32>,
}

impl Iterator for Descendants<'_> {
    type Item = NodeId;
    fn next(&mut self) -> Option<NodeId> {
        let i = self.next?;
        self.next = self.doc.next_in_subtree(i, self.root);
        Some(self.doc.id_of(i))
    }
}

/// Iterator over a node's ancestors, nearest first. See
/// [`Document::ancestors`].
#[derive(Debug, Clone)]
pub struct Ancestors<'a> {
    doc: &'a Document,
    next: Option<u32>,
}

impl Iterator for Ancestors<'_> {
    type Item = NodeId;
    fn next(&mut self) -> Option<NodeId> {
        let i = self.next?;
        self.next = self.doc.node(i).parent;
        Some(self.doc.id_of(i))
    }
}
