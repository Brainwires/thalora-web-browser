//! WebRTC APIs
//!
//! - Native builds use the webrtc crate for real P2P networking

// Native implementation using webrtc crate
#[cfg(feature = "native")]
pub mod rtc_data_channel;
#[cfg(feature = "native")]
pub mod rtc_ice_candidate;
#[cfg(feature = "native")]
pub mod rtc_peer_connection;
#[cfg(feature = "native")]
pub mod rtc_session_description;

// Re-exports
#[cfg(feature = "native")]
pub use rtc_data_channel::RTCDataChannelBuiltin as RTCDataChannel;
#[cfg(feature = "native")]
pub use rtc_ice_candidate::RTCIceCandidateBuiltin as RTCIceCandidate;
#[cfg(feature = "native")]
pub use rtc_peer_connection::RTCPeerConnectionBuiltin as RTCPeerConnection;
#[cfg(feature = "native")]
pub use rtc_session_description::RTCSessionDescriptionBuiltin as RTCSessionDescription;

#[cfg(test)]
mod webrtc_tests;
