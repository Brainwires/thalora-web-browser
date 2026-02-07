//! DOM APIs - Document Object Model

pub mod attr;
pub mod character_data;
pub mod comment;
pub mod document;

// Backward-compatible re-exports (document_fragment and document_parse now live inside document/)
pub use document::document_fragment;
pub use document::document_parse;
pub mod dom_parser;
pub mod domtokenlist;
pub mod element;
pub mod html_element;
pub mod html_script_element;
pub mod html_iframe_element;

// Native-only implementations using rquest
#[cfg(feature = "native")]
pub mod html_image_element;

// WASM stubs - browser's native APIs are used directly
#[cfg(feature = "wasm")]
pub mod html_image_element_wasm;
#[cfg(feature = "wasm")]
pub use html_image_element_wasm as html_image_element;

pub mod htmlcollection;
pub mod image_bitmap;
pub mod namednodemap;
pub mod node;
pub mod nodeiterator;
pub mod treewalker;
pub mod nodelist;
pub mod range;
pub mod selection;
pub mod shadow;
pub mod text;

#[cfg(test)]
mod element_tests;

#[cfg(test)]
mod dom_additional_tests;
