//! `thalora-dom`: Thalora's live, mutable, engine-neutral DOM.
//!
//! An arena-backed node tree ([`Document`]) built by html5ever, mutated with
//! DOM Standard semantics (pre-insertion validity, `DOMException`-shaped
//! errors), serialized per the HTML fragment serialization algorithm and
//! queried with the `selectors` crate. It holds no JS-engine types: the
//! V8 binding (`deno_web_page` in the Brainwires deno fork) sits on top and
//! talks to it through [`NodeId`]s, [`Document::serial`] numbers and the
//! mutation record log ([`Document::take_mutation_records`]).
//!
//!
//! # Model
//!
//! * One document per arena. Nodes are addressed by generation-checked
//!   [`NodeId`]s; detached nodes stay valid until the binding frees them
//!   with [`Document::free_subtree`].
//! * Template contents are a separate `DocumentFragment` (not children of
//!   the `<template>`), reachable via [`Document::template_contents`].
//! * The document is always an HTML document parsed with scripting enabled.
//!
//! # Deliberate shortcuts (for now)
//!
//! * No shadow DOM, live ranges, node iterators or `normalize()`.
//! * Selector interaction pseudo-classes (`:hover`, `:focus`, ...) parse but
//!   never match; form pseudo-classes read content attributes, not live
//!   form state.
//! * `cloneNode` of the document node is `NotSupportedError`.
//! * `previousSibling` of an insertion record is computed before the
//!   inserted node is detached from its old position, exactly as the DOM
//!   Standard words it.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod document;
mod error;
mod mutation;
mod names;
mod node;
mod parse;
mod serialize;

pub use document::{Ancestors, Children, Descendants, Document};
pub use error::{DomError, DomResult};
pub use mutation::{MutationKind, MutationRecord};
pub use node::{Attr, ElementData, NodeData, NodeId, NodeType, QuirksMode};

/// Re-exported name types (namespaces, local names, qualified names) used
/// in the public API.
pub use html5ever::{LocalName, Namespace, Prefix, QualName};
