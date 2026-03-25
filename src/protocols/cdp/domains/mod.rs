// CDP domain implementations

pub mod console;
pub mod debugger;
pub mod dom;
pub mod network;
pub mod page;
pub mod performance;
pub mod runtime;
pub mod storage;

// Re-exports
pub use console::ConsoleDomain;
pub use debugger::{BreakpointInfo, DebuggerDomain};
pub use dom::DomDomain;
pub use network::NetworkDomain;
pub use page::PageDomain;
pub use performance::PerformanceDomain;
pub use runtime::RuntimeDomain;
pub use storage::StorageDomain;
