//! Node handles and node data.

use html5ever::{LocalName, Namespace, Prefix, QualName};

/// A handle to a node in a [`Document`](crate::Document).
///
/// A `NodeId` is an arena index plus a generation. Freeing a node (see
/// [`Document::free_subtree`](crate::Document::free_subtree)) bumps the
/// generation of its slot, so an id that outlives its node is detected as
/// stale ([`DomError::StaleNode`](crate::DomError::StaleNode)) instead of
/// silently aliasing a newer node. Removing a node from the tree does *not*
/// free it: detached nodes stay valid until freed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

impl NodeId {
    /// The arena slot index of this node.
    pub fn index(self) -> u32 {
        self.index
    }

    /// The generation of the arena slot this id was issued for.
    pub fn generation(self) -> u32 {
        self.generation
    }

    /// Packs the id into a `u64` (generation in the high half) for passing
    /// across an FFI / op boundary.
    pub fn to_bits(self) -> u64 {
        (u64::from(self.generation) << 32) | u64::from(self.index)
    }

    /// Unpacks an id produced by [`NodeId::to_bits`]. Any `u64` is accepted;
    /// a value that does not name a live node is reported as stale when used.
    pub fn from_bits(bits: u64) -> NodeId {
        NodeId {
            index: bits as u32,
            generation: (bits >> 32) as u32,
        }
    }
}

/// The DOM `nodeType` of a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    /// `ELEMENT_NODE` (1).
    Element,
    /// `TEXT_NODE` (3).
    Text,
    /// `PROCESSING_INSTRUCTION_NODE` (7).
    ProcessingInstruction,
    /// `COMMENT_NODE` (8).
    Comment,
    /// `DOCUMENT_NODE` (9).
    Document,
    /// `DOCUMENT_TYPE_NODE` (10).
    DocumentType,
    /// `DOCUMENT_FRAGMENT_NODE` (11).
    DocumentFragment,
}

impl NodeType {
    /// The numeric `nodeType` value the DOM Standard assigns.
    pub fn code(self) -> u16 {
        match self {
            NodeType::Element => 1,
            NodeType::Text => 3,
            NodeType::ProcessingInstruction => 7,
            NodeType::Comment => 8,
            NodeType::Document => 9,
            NodeType::DocumentType => 10,
            NodeType::DocumentFragment => 11,
        }
    }
}

/// The document's quirks mode, as set by the parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuirksMode {
    /// Standards mode.
    #[default]
    NoQuirks,
    /// Limited-quirks ("almost standards") mode.
    LimitedQuirks,
    /// Quirks mode.
    Quirks,
}

/// An attribute of an element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attr {
    /// Namespace, prefix and local name.
    pub name: QualName,
    /// The attribute value.
    pub value: String,
}

impl Attr {
    /// Whether this attribute's qualified name (`prefix:local` or `local`)
    /// equals `qualified`.
    pub fn has_qualified_name(&self, qualified: &str) -> bool {
        qualified_name_eq(&self.name, qualified)
    }

    /// The attribute's qualified name (`prefix:local` or `local`).
    pub fn qualified_name(&self) -> String {
        qualified_name(&self.name)
    }
}

pub(crate) fn qualified_name(name: &QualName) -> String {
    match &name.prefix {
        Some(prefix) => format!("{}:{}", &**prefix, &*name.local),
        None => name.local.to_string(),
    }
}

pub(crate) fn qualified_name_eq(name: &QualName, qualified: &str) -> bool {
    match &name.prefix {
        None => &*name.local == qualified,
        Some(prefix) => {
            let (p, l) = (&**prefix, &*name.local);
            qualified.len() == p.len() + 1 + l.len()
                && qualified.starts_with(p)
                && qualified.as_bytes()[p.len()] == b':'
                && qualified.ends_with(l)
        }
    }
}

/// Data of an element node.
#[derive(Debug, Clone)]
pub struct ElementData {
    pub(crate) name: QualName,
    pub(crate) attrs: Vec<Attr>,
    /// For `<template>` (HTML namespace): the index of its contents fragment.
    pub(crate) template_contents: Option<u32>,
}

impl ElementData {
    pub(crate) fn new(name: QualName, attrs: Vec<Attr>) -> ElementData {
        ElementData {
            name,
            attrs,
            template_contents: None,
        }
    }

    /// The element's namespace, prefix and local name.
    pub fn name(&self) -> &QualName {
        &self.name
    }

    /// The element's local name.
    pub fn local_name(&self) -> &LocalName {
        &self.name.local
    }

    /// The element's namespace URL (empty for no namespace).
    pub fn namespace(&self) -> &Namespace {
        &self.name.ns
    }

    /// The element's namespace prefix, if any.
    pub fn prefix(&self) -> Option<&Prefix> {
        self.name.prefix.as_ref()
    }

    /// Whether the element is in the HTML namespace.
    pub fn is_html(&self) -> bool {
        self.name.ns == html5ever::ns!(html)
    }

    /// Whether this is an HTML element with the given local name.
    pub fn is_html_named(&self, local: &str) -> bool {
        self.is_html() && &*self.name.local == local
    }

    /// The attributes, in order.
    pub fn attrs(&self) -> &[Attr] {
        &self.attrs
    }

    /// The value of the attribute with no namespace and the given local
    /// name (exact match, no case folding).
    pub fn attr(&self, local: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|a| a.name.ns.is_empty() && &*a.name.local == local)
            .map(|a| a.value.as_str())
    }

    /// The element's ID (the `id` attribute in no namespace), if set.
    pub fn id(&self) -> Option<&str> {
        self.attr("id")
    }

    /// The qualified name in the form the DOM `tagName` reports it:
    /// ASCII-uppercased for HTML elements.
    pub fn tag_name(&self) -> String {
        let q = qualified_name(&self.name);
        if self.is_html() {
            q.to_ascii_uppercase()
        } else {
            q
        }
    }
}

/// What a node is, with its kind-specific data.
#[derive(Debug, Clone)]
pub enum NodeData {
    /// The document node.
    Document,
    /// A document type node.
    Doctype {
        /// The doctype name.
        name: String,
        /// The public identifier (empty if absent).
        public_id: String,
        /// The system identifier (empty if absent).
        system_id: String,
    },
    /// An element.
    Element(ElementData),
    /// A text node.
    Text(String),
    /// A comment.
    Comment(String),
    /// A processing instruction.
    ProcessingInstruction {
        /// The target.
        target: String,
        /// The data.
        data: String,
    },
    /// A document fragment (including template contents).
    DocumentFragment,
}

impl NodeData {
    /// The DOM `nodeType` of this data.
    pub fn node_type(&self) -> NodeType {
        match self {
            NodeData::Document => NodeType::Document,
            NodeData::Doctype { .. } => NodeType::DocumentType,
            NodeData::Element(_) => NodeType::Element,
            NodeData::Text(_) => NodeType::Text,
            NodeData::Comment(_) => NodeType::Comment,
            NodeData::ProcessingInstruction { .. } => NodeType::ProcessingInstruction,
            NodeData::DocumentFragment => NodeType::DocumentFragment,
        }
    }

    /// The element data, if this is an element.
    pub fn as_element(&self) -> Option<&ElementData> {
        match self {
            NodeData::Element(e) => Some(e),
            _ => None,
        }
    }

    /// The character data of a text, comment or processing instruction node.
    pub fn character_data(&self) -> Option<&str> {
        match self {
            NodeData::Text(s) | NodeData::Comment(s) => Some(s),
            NodeData::ProcessingInstruction { data, .. } => Some(data),
            _ => None,
        }
    }

    pub(crate) fn is_element(&self) -> bool {
        matches!(self, NodeData::Element(_))
    }

    pub(crate) fn is_text(&self) -> bool {
        matches!(self, NodeData::Text(_))
    }

    pub(crate) fn is_doctype(&self) -> bool {
        matches!(self, NodeData::Doctype { .. })
    }
}

/// Arena storage for one node: tree links plus data.
#[derive(Debug, Clone)]
pub(crate) struct Node {
    pub(crate) serial: u64,
    pub(crate) parent: Option<u32>,
    pub(crate) first_child: Option<u32>,
    pub(crate) last_child: Option<u32>,
    pub(crate) prev_sibling: Option<u32>,
    pub(crate) next_sibling: Option<u32>,
    /// For a template contents fragment: its template element.
    pub(crate) host: Option<u32>,
    pub(crate) data: NodeData,
}
