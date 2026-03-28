//! DOM APIs - Document Object Model

pub mod attr;
pub mod character_data;
pub mod document;
pub mod document_fragment;
pub mod document_parse;
pub mod dom_parser;
pub mod domtokenlist;
pub mod element;
pub mod html_element;
pub mod html_script_element;

// Native-only implementations using reqwest
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
pub mod nodelist;
pub mod range;
pub mod selection;
pub mod shadow;
pub mod text;
pub mod treewalker;

#[cfg(test)]
mod document_tests;

#[cfg(test)]
mod element_tests;

#[cfg(test)]
mod dom_additional_tests;
