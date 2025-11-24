//! Audio APIs
//!
//! This module provides audio playback capabilities using rodio.
//! https://html.spec.whatwg.org/multipage/media.html
//! https://webaudio.github.io/web-audio-api/

pub mod html_audio_element;
pub mod audio_context;

// Re-exports
pub use html_audio_element::HTMLAudioElement;
pub use audio_context::AudioContext;
