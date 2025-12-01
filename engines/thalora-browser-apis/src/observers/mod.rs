//! Observer APIs

pub mod mutation_observer;
pub mod intersection_observer;
pub mod resize_observer;
pub mod performance_observer;

pub use performance_observer::{PerformanceObserver, PerformanceEntry, PerformanceObserverEntryList};

#[cfg(test)]
mod tests;
