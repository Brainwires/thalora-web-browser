// Advanced browser features

// Native-only: fingerprinting uses reqwest
#[cfg(feature = "core")]
pub mod fingerprinting;

pub mod webgl;

// Native-only: the vault uses the dirs crate for its default location
#[cfg(feature = "core")]
pub mod credential_vault;

// webassembly is now natively implemented in Boa engine
pub mod readability;

// Re-exports for clean API
#[cfg(feature = "core")]
pub use fingerprinting::{BrowserFingerprint, BrowserType, FingerprintManager};
pub use webgl::WebGLManager;
// webassembly types are now handled by Boa engine
#[cfg(feature = "core")]
pub use credential_vault::{CredentialEntry, CredentialVault};
pub use readability::{
    ExtractionOptions, ExtractionResult, OutputFormat, QualityMetrics, ReadabilityConfig,
    ReadabilityEngine,
};
