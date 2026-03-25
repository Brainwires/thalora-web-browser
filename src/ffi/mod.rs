//! FFI (Foreign Function Interface) module for Thalora browser engine.
//!
//! Exposes C-compatible functions for use via P/Invoke from .NET,
//! or any other language that supports C FFI.

mod instance;
mod interaction;
mod navigation;

pub use instance::*;
pub use interaction::*;
pub use navigation::*;
