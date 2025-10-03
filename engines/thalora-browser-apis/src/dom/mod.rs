//! DOM APIs - Document Object Model

pub mod attr;
pub mod character_data;
pub mod document;
pub mod document_fragment;
pub mod document_parse;
pub mod domtokenlist;
pub mod element;
pub mod node;
pub mod nodelist;
pub mod range;
pub mod shadow;
pub mod text;

#[cfg(test)]
mod document_tests;

#[cfg(test)]
mod element_tests;
