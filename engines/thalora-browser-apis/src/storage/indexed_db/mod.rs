//! IndexedDB API Implementation
//!
//! A low-level API for client-side storage of significant amounts of structured data,
//! including files/blobs. This API uses indexes to enable high-performance searches of this data.
//!
//! More information:
//! - [W3C Specification](https://w3c.github.io/IndexedDB/)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API)

pub mod backend;
pub mod cursor;
pub mod database;
pub mod factory;
pub mod index;
pub mod key;
pub mod key_range;
pub mod object_store;
pub mod request;
pub mod transaction;
pub mod version_change_event;

#[cfg(test)]
mod tests;
