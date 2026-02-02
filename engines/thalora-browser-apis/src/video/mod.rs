//! Video APIs
//!
//! This module provides video playback capabilities.
//! https://html.spec.whatwg.org/multipage/media.html#htmlvideoelement

// Native implementation using rquest
#[cfg(feature = "native")]
pub mod html_video_element;

// WASM stub - browser's native HTMLVideoElement is used directly
#[cfg(feature = "wasm")]
pub mod html_video_element_wasm;
#[cfg(feature = "wasm")]
pub use html_video_element_wasm as html_video_element;

#[cfg(test)]
mod tests;

// Re-exports
pub use html_video_element::HTMLVideoElement;
