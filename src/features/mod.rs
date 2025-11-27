// Advanced browser features

// Native-only: fingerprinting uses reqwest
#[cfg(feature = "native")]
pub mod fingerprinting;
#[cfg(feature = "wasm")]
pub mod fingerprinting_wasm;
#[cfg(feature = "wasm")]
pub use fingerprinting_wasm as fingerprinting;

pub mod webgl;

// Native-only: ai_memory uses dirs crate for filesystem paths
#[cfg(feature = "native")]
pub mod ai_memory;
#[cfg(feature = "wasm")]
pub mod ai_memory_wasm;
#[cfg(feature = "wasm")]
pub use ai_memory_wasm as ai_memory;

// webassembly is now natively implemented in Boa engine
pub mod readability;

// Re-exports for clean API
pub use fingerprinting::{BrowserFingerprint, FingerprintManager, BrowserType};
pub use webgl::WebGLManager;
// webassembly types are now handled by Boa engine
pub use ai_memory::{AiMemoryHeap, MemoryData, ResearchEntry, CredentialEntry, SessionData, BookmarkEntry, NoteEntry, MemorySearchCriteria, MemorySortBy, SessionStatus, NotePriority, MemoryStatistics};
pub use readability::{ReadabilityEngine, ReadabilityConfig, QualityMetrics, ExtractionResult, ExtractionOptions, OutputFormat};