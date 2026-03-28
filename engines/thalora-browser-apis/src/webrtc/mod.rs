//! WebRTC APIs
//!
//! - Native builds use the webrtc crate for real P2P networking
//! - WASM builds delegate to the browser's native WebRTC API

// Native implementation using webrtc crate
#[cfg(feature = "native")]
pub mod rtc_data_channel;
#[cfg(feature = "native")]
pub mod rtc_ice_candidate;
#[cfg(feature = "native")]
pub mod rtc_peer_connection;
#[cfg(feature = "native")]
pub mod rtc_session_description;

// WASM stubs - actual WebRTC is used through web-sys
#[cfg(feature = "wasm")]
pub mod rtc_data_channel_wasm;
#[cfg(feature = "wasm")]
pub mod rtc_ice_candidate_wasm;
#[cfg(feature = "wasm")]
pub mod rtc_peer_connection_wasm;
#[cfg(feature = "wasm")]
pub mod rtc_session_description_wasm;

// Re-exports
#[cfg(feature = "native")]
pub use rtc_data_channel::RTCDataChannelBuiltin as RTCDataChannel;
#[cfg(feature = "native")]
pub use rtc_ice_candidate::RTCIceCandidateBuiltin as RTCIceCandidate;
#[cfg(feature = "native")]
pub use rtc_peer_connection::RTCPeerConnectionBuiltin as RTCPeerConnection;
#[cfg(feature = "native")]
pub use rtc_session_description::RTCSessionDescriptionBuiltin as RTCSessionDescription;

#[cfg(feature = "wasm")]
pub use rtc_data_channel_wasm::RTCDataChannel;
#[cfg(feature = "wasm")]
pub use rtc_ice_candidate_wasm::RTCIceCandidate;
#[cfg(feature = "wasm")]
pub use rtc_peer_connection_wasm::RTCPeerConnection;
#[cfg(feature = "wasm")]
pub use rtc_session_description_wasm::RTCSessionDescription;

#[cfg(test)]
mod webrtc_tests;
