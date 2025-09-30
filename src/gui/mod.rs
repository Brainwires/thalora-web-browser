//! GUI module for Thalora graphical web browser
//! 
//! This module provides a traditional web browser interface using winit for window management
//! and wgpu for rendering. It integrates with the existing HeadlessWebBrowser engine to provide
//! visual web browsing capabilities.

pub mod window;
pub mod renderer;
pub mod browser_ui;
pub mod tab_manager;
pub mod input_handler;

// Re-export main types
pub use window::WindowManager;
pub use renderer::{WebRenderer, RenderState};
pub use browser_ui::{BrowserUI, NavigationState};
pub use tab_manager::{TabManager, Tab, TabId};
pub use input_handler::InputHandler;

use crate::engine::EngineConfig;
use anyhow::Result;

/// Main graphical browser application
pub struct GraphicalBrowser {
    width: u32,
    height: u32,
    fullscreen: bool,
    debug: bool,
    engine_config: EngineConfig,
}

impl GraphicalBrowser {
    /// Create a new graphical browser instance
    pub fn new(
        width: u32,
        height: u32,
        fullscreen: bool,
        debug: bool,
        engine_config: EngineConfig,
    ) -> Result<Self> {
        tracing::info!("Initializing graphical browser with {}x{} window", width, height);

        Ok(Self {
            width,
            height,
            fullscreen,
            debug,
            engine_config,
        })
    }

    /// Navigate to a URL in the current tab
    pub async fn navigate_to(&mut self, url: &str) -> Result<()> {
        tracing::info!("GUI browser would navigate to: {}", url);
        // TODO: Implement actual navigation when window/renderer integration is complete
        Ok(())
    }

    /// Run the main browser event loop
    pub async fn run(self) -> Result<()> {
        tracing::info!("Starting GUI browser event loop");
        tracing::info!("GUI browser configuration:");
        tracing::info!("  Size: {}x{}", self.width, self.height);
        tracing::info!("  Fullscreen: {}", self.fullscreen);
        tracing::info!("  Debug: {}", self.debug);
        tracing::info!("  Engine: {:?}", self.engine_config.engine_type);

        // Initialize components
        let mut window_manager = WindowManager::new(self.width, self.height, self.fullscreen, self.debug);
        let (window, event_loop) = window_manager.create_window().await?;
        
        let mut renderer = WebRenderer::new(&window).await?;
        let mut browser_ui = BrowserUI::new(self.debug);
        let mut tab_manager = TabManager::new(self.engine_config.clone()).await?;
        let mut input_handler = InputHandler::new();

        // Create initial tab with default page
        let initial_tab_id = tab_manager.create_tab("about:blank".to_string()).await?;
        tab_manager.set_active_tab(initial_tab_id)?;

        println!("Thalora GUI Browser - Running");
        println!("Press Ctrl+Q to quit, Ctrl+T for new tab, Ctrl+W to close tab");

        // Run the event loop
        use winit::event::{Event, WindowEvent};

        event_loop.run(move |event, window_target| {
            match event {
                Event::WindowEvent { event, .. } => {
                    // Handle egui events first
                    if renderer.handle_event(&event) {
                        return;
                    }

                    // Handle input events
                    if let Some(action) = input_handler.handle_event(&event) {
                        match action {
                            crate::gui::input_handler::BrowserAction::Quit => {
                                window_target.exit();
                            }
                            crate::gui::input_handler::BrowserAction::NewTab => {
                                if let Ok(tab_id) = pollster::block_on(tab_manager.create_tab("about:blank".to_string())) {
                                    let _ = tab_manager.set_active_tab(tab_id);
                                }
                            }
                            crate::gui::input_handler::BrowserAction::CloseTab => {
                                if let Some(active_id) = tab_manager.get_active_tab_id() {
                                    let _ = tab_manager.close_tab(active_id);
                                }
                            }
                            crate::gui::input_handler::BrowserAction::Navigate(url) => {
                                if let Some(active_id) = tab_manager.get_active_tab_id() {
                                    let _ = pollster::block_on(tab_manager.navigate_tab(active_id, &url));
                                }
                            }
                            _ => {
                                // Handle other actions
                            }
                        }
                    }

                    match event {
                                                WindowEvent::CloseRequested => {
                            window_target.exit();
                        }
                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { .. } => {
                            // Handle scale factor change
                        }
                        _ => {}
                    }
                }
                Event::AboutToWait => {
                    // Check for pending navigation from UI
                    if let Some(url) = browser_ui.take_pending_navigation() {
                        if let Some(active_id) = tab_manager.get_active_tab_id() {
                            tracing::info!("Processing pending navigation to: {}", url);
                            if let Err(e) = pollster::block_on(tab_manager.navigate_tab(active_id, &url)) {
                                tracing::error!("Navigation failed: {}", e);
                            }
                        }
                    }

                    // Update browser UI state
                    if let Some(active_tab) = tab_manager.get_active_tab() {
                        browser_ui.update_from_tab(active_tab);
                    }

                    // Render frame
                    if let Err(e) = renderer.render(&mut browser_ui, &tab_manager) {
                        tracing::error!("Render error: {}", e);
                    }

                    // Request redraw
                    window.request_redraw();
                }
                _ => {}
            }
        }).map_err(|e| anyhow::anyhow!("Event loop error: {}", e))?;

        Ok(())
    }
}