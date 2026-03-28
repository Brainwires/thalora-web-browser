//! Implementation of the Web Locks API.
//!
//! The Web Locks API provides a mechanism for coordinating access to shared resources
//! using named locks. It allows code in one tab to asynchronously acquire a lock,
//! hold it while work is performed, then release it.
//!
//! More information:
//! - [W3C Specification](https://www.w3.org/TR/web-locks/)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/Web_Locks_API)

pub mod lock;
pub mod lock_info;
pub mod lock_manager;

pub use lock::Lock;
pub use lock_info::{LockInfo, LockManagerSnapshot};
pub use lock_manager::LockManager;

#[cfg(test)]
mod tests;
