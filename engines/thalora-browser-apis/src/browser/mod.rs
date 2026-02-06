//! Browser objects - Window, Navigator, History, Performance, Location

pub mod clipboard;
pub mod cssom;
pub mod focus_manager;
pub mod frame_selection;
pub mod history;
pub mod keyboard_dispatcher;
pub mod location;
pub mod navigation_bridge;
pub mod navigator;
pub mod notification;
pub mod performance;
pub mod permissions;
pub mod selection;
pub mod vibration;
pub mod window;
pub mod window_registry;

#[cfg(test)]
mod tests;
