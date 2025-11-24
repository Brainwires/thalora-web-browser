//! Video APIs
//!
//! This module provides video playback capabilities.
//! https://html.spec.whatwg.org/multipage/media.html#htmlvideoelement

pub mod html_video_element;

#[cfg(test)]
mod tests;

// Re-exports
pub use html_video_element::HTMLVideoElement;
