//! Audio APIs
//!
//! This module provides audio playback capabilities.
//! - Native builds use rodio for audio playback
//!
//! <https://html.spec.whatwg.org/multipage/media.html>
//! <https://webaudio.github.io/web-audio-api/>

// Native implementation using rodio
#[cfg(feature = "native")]
pub mod audio_context;
#[cfg(feature = "native")]
pub mod html_audio_element;

#[cfg(test)]
mod tests;

// Re-exports at module level
pub use audio_context::AudioContext;
pub use html_audio_element::HTMLAudioElement;
