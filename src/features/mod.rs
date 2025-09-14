// Advanced browser features
pub mod fingerprinting;
// pub mod webgl; // Temporarily disabled due to Boa API compatibility issues
pub mod ai_memory;
pub mod react_processor;
pub mod solver;

// Re-exports for clean API
pub use fingerprinting::{BrowserFingerprint, FingerprintManager, BrowserType};
// pub use webgl::WebGLManager;
pub use ai_memory::{AiMemoryHeap, MemoryData, ResearchEntry, CredentialEntry, SessionData, BookmarkEntry, NoteEntry, MemorySearchCriteria, MemorySortBy, SessionStatus, NotePriority, MemoryStatistics};
pub use react_processor::{ReactProcessor, ReactElement, ProcessedReactData};
pub use solver::{ChallengeSolver, ChallengeType, ChallengeResult, SolverConfig, ChallengePatterns};