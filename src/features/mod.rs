// Advanced browser features
pub mod fingerprinting;
pub mod webgl;
pub mod ai_memory;
pub mod webassembly;
pub mod readability;

// Re-exports for clean API
pub use fingerprinting::{BrowserFingerprint, FingerprintManager, BrowserType};
pub use webgl::WebGLManager;
pub use webassembly::{AdvancedWebAssemblyEngine, WebAssemblyEngine, ValidationResult, OptimizationResult};
pub use ai_memory::{AiMemoryHeap, MemoryData, ResearchEntry, CredentialEntry, SessionData, BookmarkEntry, NoteEntry, MemorySearchCriteria, MemorySortBy, SessionStatus, NotePriority, MemoryStatistics};
pub use readability::{ReadabilityEngine, ReadabilityConfig, QualityMetrics, ExtractionResult, ExtractionOptions, OutputFormat};