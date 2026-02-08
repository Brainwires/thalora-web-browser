//! Document Web API implementations
//!
//! Contains the Document interface, DocumentFragment interface,
//! and parseHTMLUnsafe implementation.
//! https://dom.spec.whatwg.org/#interface-document

mod types;
mod intrinsic;
mod properties;
mod creation;
mod query;
mod events;
mod canvas;
mod collections;
pub mod dom_tree;

pub mod document_fragment;
pub mod document_parse;

pub use types::*;
pub use intrinsic::*;
pub use document_fragment::*;
pub use document_parse::*;

#[cfg(test)]
mod tests;
