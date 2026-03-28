//! Observer APIs

pub mod intersection_observer;
pub mod mutation_observer;
pub mod performance_observer;
pub mod resize_observer;

pub use performance_observer::{
    PerformanceEntry, PerformanceObserver, PerformanceObserverEntryList,
};

#[cfg(test)]
mod tests;
