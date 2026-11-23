//! MCP tools for the encrypted credential vault.
//!
//! Secrets go in (`vault_store_credential`) and are only ever used by
//! `browser_fill_credential`; no tool returns them to the model.

pub(crate) mod handlers;

pub(crate) use handlers::normalize_origin;
pub use handlers::{handle_delete_credential, handle_list_credentials, handle_store_credential};
