//! Audio APIs
//!
//! This module provides audio playback capabilities.
//! - Native builds use rodio for audio playback
//! - WASM builds delegate to the browser's native Web Audio API
//! https://html.spec.whatwg.org/multipage/media.html
//! https://webaudio.github.io/web-audio-api/

// Native implementation using rodio
#[cfg(feature = "native")]
pub mod html_audio_element;
#[cfg(feature = "native")]
pub mod audio_context;

// WASM stubs - actual audio is used through web-sys
#[cfg(feature = "wasm")]
pub mod html_audio_element_wasm;
#[cfg(feature = "wasm")]
pub mod audio_context_wasm;

// Re-export WASM stubs with native module names for uniform API
#[cfg(feature = "wasm")]
pub use html_audio_element_wasm as html_audio_element;
#[cfg(feature = "wasm")]
pub use audio_context_wasm as audio_context;

#[cfg(test)]
mod tests;

// Re-exports at module level
pub use html_audio_element::HTMLAudioElement;
pub use audio_context::AudioContext;
