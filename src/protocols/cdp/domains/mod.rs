// CDP domain implementations

pub mod runtime;
pub mod debugger;
pub mod dom;
pub mod network;
pub mod page;
pub mod console;
pub mod performance;
pub mod storage;

// Re-exports
pub use runtime::RuntimeDomain;
pub use debugger::{DebuggerDomain, BreakpointInfo};
pub use dom::DomDomain;
pub use network::NetworkDomain;
pub use page::PageDomain;
pub use console::ConsoleDomain;
pub use performance::PerformanceDomain;
pub use storage::StorageDomain;
