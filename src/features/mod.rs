// Advanced browser features

// Native-only: fingerprinting uses reqwest
#[cfg(feature = "core")]
pub mod fingerprinting;

// Native-only: the vault uses the dirs crate for its default location
#[cfg(feature = "core")]
pub mod credential_vault;

pub mod readability;

// Re-exports for clean API
#[cfg(feature = "core")]
pub use credential_vault::{CredentialEntry, CredentialVault};
#[cfg(feature = "core")]
pub use fingerprinting::{BrowserFingerprint, BrowserType, FingerprintManager};
pub use readability::{
    ExtractionOptions, ExtractionResult, OutputFormat, QualityMetrics, ReadabilityConfig,
    ReadabilityEngine,
};
