//! Window management for the graphical browser
//! 
//! This module handles window creation, event loop management, and basic window operations
//! using winit.

use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
    dpi::PhysicalSize,
};
use anyhow::{Result, Context};
use std::sync::Arc;

/// Window manager for the browser application
pub struct WindowManager {
    width: u32,
    height: u32,
    fullscreen: bool,
    debug: bool,
}

impl WindowManager {
    /// Create a new window manager
    pub fn new(width: u32, height: u32, fullscreen: bool, debug_enabled: bool) -> Self {
        tracing::info!("Initializing window manager: {}x{}, fullscreen: {}, debug: {}", 
                      width, height, fullscreen, debug_enabled);
        
        Self {
            width,
            height,
            fullscreen,
            debug: debug_enabled,
        }
    }    /// Create the window and event loop
    pub async fn create_window(self) -> Result<(Arc<Window>, EventLoop<()>)> {
        tracing::info!("Creating window: {}x{}, fullscreen: {}", self.width, self.height, self.fullscreen);

        // Create event loop
        let event_loop = EventLoop::new()
            .context("Failed to create event loop")?;

        // Create window builder
        let mut window_builder = WindowBuilder::new()
            .with_title("Thalora Web Browser")
            .with_inner_size(PhysicalSize::new(self.width, self.height))
            .with_resizable(true);

        // Apply fullscreen if requested
        if self.fullscreen {
            window_builder = window_builder.with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        }

        // Create the actual window
        let window = window_builder
            .build(&event_loop)
            .context("Failed to create window")?;

        tracing::info!("Window created successfully");

        Ok((Arc::new(window), event_loop))
    }
}

/// Utility functions for window operations
impl WindowManager {
    /// Set window title
    pub fn set_title(window: &Window, title: &str) {
        window.set_title(title);
    }

    /// Toggle fullscreen mode
    pub fn toggle_fullscreen(window: &Window) {
        let current_fullscreen = window.fullscreen();
        if current_fullscreen.is_some() {
            window.set_fullscreen(None);
        } else {
            window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        }
    }

    /// Get the window scale factor for DPI awareness
    pub fn scale_factor(window: &Window) -> f64 {
        window.scale_factor()
    }

    /// Convert logical coordinates to physical coordinates
    pub fn logical_to_physical(window: &Window, logical: (f32, f32)) -> (u32, u32) {
        let scale = window.scale_factor() as f32;
        ((logical.0 * scale) as u32, (logical.1 * scale) as u32)
    }

    /// Convert physical coordinates to logical coordinates
    pub fn physical_to_logical(window: &Window, physical: (u32, u32)) -> (f32, f32) {
        let scale = window.scale_factor() as f32;
        (physical.0 as f32 / scale, physical.1 as f32 / scale)
    }
}