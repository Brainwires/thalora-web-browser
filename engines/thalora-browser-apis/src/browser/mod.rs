//! Browser objects - Window, Navigator, History, Performance, Location

pub mod clipboard;
pub mod frame_selection;
pub mod history;
pub mod location;
pub mod navigator;
pub mod notification;
pub mod performance;
pub mod permissions;
pub mod selection;
pub mod vibration;
pub mod window;

#[cfg(test)]
mod tests;
