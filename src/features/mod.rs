// Advanced browser features
pub mod fingerprinting;
pub mod webgl;
pub mod ai_memory;
// webassembly is now natively implemented in Boa engine
pub mod readability;

// Re-exports for clean API
pub use fingerprinting::{BrowserFingerprint, FingerprintManager, BrowserType};
pub use webgl::WebGLManager;
// webassembly types are now handled by Boa engine
pub use ai_memory::{AiMemoryHeap, MemoryData, ResearchEntry, CredentialEntry, SessionData, BookmarkEntry, NoteEntry, MemorySearchCriteria, MemorySortBy, SessionStatus, NotePriority, MemoryStatistics};
pub use readability::{ReadabilityEngine, ReadabilityConfig, QualityMetrics, ExtractionResult, ExtractionOptions, OutputFormat};