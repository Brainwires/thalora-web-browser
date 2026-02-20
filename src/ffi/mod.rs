//! FFI (Foreign Function Interface) module for Thalora browser engine.
//!
//! Exposes C-compatible functions for use via P/Invoke from .NET,
//! or any other language that supports C FFI.

mod instance;
mod navigation;
mod interaction;

pub use instance::*;
pub use navigation::*;
pub use interaction::*;
