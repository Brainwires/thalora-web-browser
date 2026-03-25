// Advanced browser features

// Native-only: fingerprinting uses reqwest
#[cfg(feature = "core")]
pub mod fingerprinting;
#[cfg(feature = "wasm")]
pub mod fingerprinting_wasm;
#[cfg(feature = "wasm")]
pub use fingerprinting_wasm as fingerprinting;

pub mod webgl;

// Native-only: ai_memory uses dirs crate for filesystem paths
#[cfg(feature = "core")]
pub mod ai_memory;
#[cfg(feature = "wasm")]
pub mod ai_memory_wasm;
#[cfg(feature = "wasm")]
pub use ai_memory_wasm as ai_memory;

// webassembly is now natively implemented in Boa engine
pub mod readability;

// Re-exports for clean API
#[cfg(feature = "core")]
pub use fingerprinting::{BrowserFingerprint, BrowserType, FingerprintManager};
pub use webgl::WebGLManager;
// webassembly types are now handled by Boa engine
#[cfg(feature = "core")]
pub use ai_memory::{
    AiMemoryHeap, BookmarkEntry, CredentialEntry, MemoryData, MemorySearchCriteria, MemorySortBy,
    MemoryStatistics, NoteEntry, NotePriority, ResearchEntry, SessionData, SessionStatus,
};
pub use readability::{
    ExtractionOptions, ExtractionResult, OutputFormat, QualityMetrics, ReadabilityConfig,
    ReadabilityEngine,
};
