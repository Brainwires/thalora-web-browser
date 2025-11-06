//! Browser objects - Window, Navigator, History, Performance, Location

pub mod frame_selection;
pub mod history;
pub mod location;
pub mod navigator;
pub mod performance;
pub mod selection;
// TODO: Implement web_locks module
// pub mod web_locks;
pub mod window;

#[cfg(test)]
mod tests;
