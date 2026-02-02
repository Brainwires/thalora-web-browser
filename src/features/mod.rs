// Advanced browser features

// Native-only: fingerprinting uses reqwest
#[cfg(any(feature = "native", feature = "web-search"))]
pub mod fingerprinting;
#[cfg(feature = "wasm")]
pub mod fingerprinting_wasm;
#[cfg(feature = "wasm")]
pub use fingerprinting_wasm as fingerprinting;

pub mod webgl;

// Mouse simulation and event dispatching for realistic human-like interactions
pub mod mouse_simulation;
pub mod event_dispatcher;

// Challenge solver for handling Cloudflare/Turnstile through proper browser behavior
pub mod solver;

// Native-only: ai_memory uses dirs crate for filesystem paths
#[cfg(any(feature = "native", feature = "web-search"))]
pub mod ai_memory;
#[cfg(feature = "wasm")]
pub mod ai_memory_wasm;
#[cfg(feature = "wasm")]
pub use ai_memory_wasm as ai_memory;

// webassembly is now natively implemented in Boa engine
pub mod readability;

// Re-exports for clean API
#[cfg(any(feature = "native", feature = "web-search"))]
pub use fingerprinting::{BrowserFingerprint, FingerprintManager, BrowserType};
pub use webgl::WebGLManager;
// webassembly types are now handled by Boa engine
#[cfg(any(feature = "native", feature = "web-search"))]
pub use ai_memory::{AiMemoryHeap, MemoryData, ResearchEntry, CredentialEntry, SessionData, BookmarkEntry, NoteEntry, MemorySearchCriteria, MemorySortBy, SessionStatus, NotePriority, MemoryStatistics};
pub use readability::{ReadabilityEngine, ReadabilityConfig, QualityMetrics, ExtractionResult, ExtractionOptions, OutputFormat};

// Mouse simulation exports
pub use mouse_simulation::{MousePath, MousePoint, MousePathConfig, ClickSequence};
pub use event_dispatcher::{EventSequence, EventAction, EventCoords, MouseEventType, MouseButton};

// Challenge solver exports
pub use solver::{ChallengeSolver, ChallengeDetector, ChallengeType, DetectedChallenge, BehavioralSimulator, BehavioralConfig};