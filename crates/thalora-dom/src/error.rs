//! Errors returned by DOM operations.

use std::fmt;

use crate::NodeId;

/// An error from a DOM operation.
///
/// Each variant maps to a `DOMException` name (see [`DomError::name`]) so the
/// binding layer can throw the exception the DOM Standard requires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomError {
    /// The operation would yield an incorrect node tree
    /// (`HierarchyRequestError`).
    HierarchyRequest,
    /// The object can not be found here (`NotFoundError`), e.g. the reference
    /// child is not a child of the parent.
    NotFound,
    /// A name contains an invalid character (`InvalidCharacterError`).
    InvalidCharacter,
    /// A namespace / prefix combination is not allowed (`NamespaceError`).
    Namespace,
    /// A selector or markup string failed to parse (`SyntaxError`). The
    /// payload is a human-readable reason (not exposed to pages verbatim by
    /// default; the binding decides the message).
    Syntax(String),
    /// The operation is not supported (`NotSupportedError`), e.g. cloning
    /// the document node.
    NotSupported,
    /// The operation does not apply to this kind of node
    /// (`InvalidNodeTypeError`), e.g. reading attributes of a text node.
    InvalidNodeType,
    /// The node id refers to a node that was freed (or never existed in this
    /// document). Maps to `InvalidStateError`; a well-behaved binding never
    /// produces one.
    StaleNode(NodeId),
}

impl DomError {
    /// The `DOMException` name this error maps to.
    pub fn name(&self) -> &'static str {
        match self {
            DomError::HierarchyRequest => "HierarchyRequestError",
            DomError::NotFound => "NotFoundError",
            DomError::InvalidCharacter => "InvalidCharacterError",
            DomError::Namespace => "NamespaceError",
            DomError::Syntax(_) => "SyntaxError",
            DomError::NotSupported => "NotSupportedError",
            DomError::InvalidNodeType => "InvalidNodeTypeError",
            DomError::StaleNode(_) => "InvalidStateError",
        }
    }
}

impl fmt::Display for DomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomError::Syntax(reason) => write!(f, "SyntaxError: {reason}"),
            DomError::StaleNode(id) => write!(f, "InvalidStateError: stale node id {id:?}"),
            other => f.write_str(other.name()),
        }
    }
}

impl std::error::Error for DomError {}

/// Shorthand for results of DOM operations.
pub type DomResult<T> = Result<T, DomError>;
