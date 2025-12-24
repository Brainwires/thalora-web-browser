//! Browser UI components using egui
//!
//! This module provides the browser's user interface including address bar, navigation buttons,
//! tabs, and other browser chrome elements.

use egui::{Context, TopBottomPanel, CentralPanel, SidePanel, Ui};
use crate::gui::{TabManager, FontManager};

// Module declarations
mod types;
mod chrome;
mod dom_rendering;
mod styles;
mod state;

// Re-export public types
pub use types::{NavigationState, BrowserAction};

// Internal imports
use types::*;

/// Main browser UI manager
pub struct BrowserUI {
    navigation_state: NavigationState,
    address_bar_text: String,
    is_editing_address: bool,
    debug_mode: bool,
    show_dev_tools: bool,
    search_text: String,
    pending_navigation: Option<String>,
    pending_action: Option<BrowserAction>,
    font_manager: FontManager,
    base_font_size: f32,
}

impl BrowserUI {
    /// Create a new browser UI
    pub fn new(debug_mode: bool) -> Self {
        Self {
            navigation_state: NavigationState::default(),
            address_bar_text: String::new(),
            is_editing_address: false,
            debug_mode,
            show_dev_tools: false,
            search_text: String::new(),
            pending_navigation: None,
            pending_action: None,
            font_manager: FontManager::new(),
            base_font_size: 16.0,
        }
    }

    /// Initialize fonts for the context
    pub fn init_fonts(&self, ctx: &Context) {
        self.font_manager.install_fonts(ctx);
    }

    /// Show the main browser UI
    pub fn show(&mut self, ctx: &Context, tab_manager: &TabManager) {
        // Top panel with navigation and address bar
        TopBottomPanel::top("browser_top_panel").show(ctx, |ui| {
            self.show_navigation_bar(ui, tab_manager);
        });

        // Side panel for developer tools (if enabled)
        if self.show_dev_tools {
            SidePanel::right("dev_tools_panel")
                .default_width(300.0)
                .show(ctx, |ui| {
                    self.show_dev_tools_panel(ui);
                });
        }

        // Central panel for web content
        CentralPanel::default().show(ctx, |ui| {
            self.show_web_content_area(ui, tab_manager);
        });

        // Handle keyboard shortcuts
        self.handle_shortcuts(ctx);
    }

    /// Show the web content area
    fn show_web_content_area(&mut self, ui: &mut Ui, tab_manager: &TabManager) {
        if let Some(tab_id) = tab_manager.current_tab_id() {
            if let Some(tab) = tab_manager.get_tab(tab_id) {
                // Show loading indicator if needed
                if self.navigation_state.is_loading {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.spinner();
                        ui.label("Loading...");
                    });
                } else {
                    // Display the rendered HTML content with proper styling
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            // Get raw HTML content directly
                            let html_content = pollster::block_on(tab.get_content()).unwrap_or_default();

                            if html_content.trim().is_empty() || html_content == "about:blank" {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(50.0);
                                    ui.heading(tab.title());
                                    ui.label("No content loaded");
                                });
                            } else {
                                // Add padding around the content like a real browser
                                egui::Frame::none()
                                    .inner_margin(egui::Margin::symmetric(40.0, 20.0))
                                    .fill(egui::Color32::WHITE)
                                    .show(ui, |ui| {
                                        // Parse and render HTML directly with scraper
                                        use scraper::{Html, Selector};
                                        let document = Html::parse_document(&html_content);

                                        ui.spacing_mut().item_spacing.y = 8.0;
                                        ui.style_mut().visuals.override_text_color = Some(egui::Color32::from_rgb(33, 33, 33));

                                        // Render body content
                                        if let Ok(body_selector) = Selector::parse("body") {
                                            if let Some(body) = document.select(&body_selector).next() {
                                                for child in body.children() {
                                                    if let Some(element) = scraper::ElementRef::wrap(child) {
                                                        self.render_dom_element(ui, element);
                                                    }
                                                }
                                            } else {
                                                // No body, render document root children
                                                for node in document.root_element().children() {
                                                    if let Some(element) = scraper::ElementRef::wrap(node) {
                                                        if element.value().name() != "head" {
                                                            self.render_dom_element(ui, element);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    });
                            }
                        });
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("No Tab Selected");
                });
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("Welcome to Thalora Browser");
                ui.label("Create a new tab to start browsing");

                if ui.button("Create New Tab").clicked() {
                    tracing::debug!("Creating initial tab");
                    self.set_pending_action(BrowserAction::NewTab);
                }
            });
        }
    }

    /// Show developer tools panel
    fn show_dev_tools_panel(&mut self, ui: &mut Ui) {
        ui.heading("Developer Tools");
        ui.separator();

        // Console section
        ui.collapsing("Console", |ui| {
            ui.label("JavaScript console output will appear here");

            ui.horizontal(|ui| {
                let response = ui.text_edit_singleline(&mut self.search_text);
                let execute_clicked = ui.button("Execute").clicked();
                let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                if (execute_clicked || enter_pressed) && !self.search_text.is_empty() {
                    tracing::debug!("Executing JS: {}", self.search_text);
                    let code = self.search_text.clone();
                    self.search_text.clear();
                    self.set_pending_action(BrowserAction::ExecuteJavaScript(code));
                }
            });
        });

        // Network section
        ui.collapsing("Network", |ui| {
            ui.label("Network requests will be shown here");
        });

        // Elements section
        ui.collapsing("Elements", |ui| {
            ui.label("DOM tree will be displayed here");
        });

        // Performance section
        ui.collapsing("Performance", |ui| {
            ui.label("Performance metrics and profiling");
        });
    }

    /// Handle keyboard shortcuts
    fn handle_shortcuts(&mut self, ctx: &Context) {
        ctx.input(|i| {
            // Ctrl+T - New tab
            if i.modifiers.ctrl && i.key_pressed(egui::Key::T) {
                tracing::debug!("Keyboard shortcut: New tab");
                self.set_pending_action(BrowserAction::NewTab);
            }

            // Ctrl+W - Close tab
            if i.modifiers.ctrl && i.key_pressed(egui::Key::W) {
                tracing::debug!("Keyboard shortcut: Close tab");
                // CloseTab with 0 means close current tab (handled in event loop)
                self.set_pending_action(BrowserAction::CloseTab(0));
            }

            // Ctrl+R or F5 - Reload
            if (i.modifiers.ctrl && i.key_pressed(egui::Key::R)) || i.key_pressed(egui::Key::F5) {
                tracing::debug!("Keyboard shortcut: Reload");
                self.set_pending_action(BrowserAction::Reload);
            }

            // F12 - Toggle dev tools
            if i.key_pressed(egui::Key::F12) {
                self.show_dev_tools = !self.show_dev_tools;
                tracing::debug!("Toggled dev tools: {}", self.show_dev_tools);
            }

            // Ctrl+L - Focus address bar
            if i.modifiers.ctrl && i.key_pressed(egui::Key::L) {
                tracing::debug!("Keyboard shortcut: Focus address bar");
                self.set_pending_action(BrowserAction::FocusAddressBar);
            }

            // Alt+Left - Go back
            if i.modifiers.alt && i.key_pressed(egui::Key::ArrowLeft) && self.navigation_state.can_go_back {
                tracing::debug!("Keyboard shortcut: Go back");
                self.set_pending_action(BrowserAction::GoBack);
            }

            // Alt+Right - Go forward
            if i.modifiers.alt && i.key_pressed(egui::Key::ArrowRight) && self.navigation_state.can_go_forward {
                tracing::debug!("Keyboard shortcut: Go forward");
                self.set_pending_action(BrowserAction::GoForward);
            }

            // Escape - Stop loading
            if i.key_pressed(egui::Key::Escape) && self.navigation_state.is_loading {
                tracing::debug!("Keyboard shortcut: Stop loading");
                self.set_pending_action(BrowserAction::StopLoading);
            }
        });
    }

    /// Navigate to the URL in the address bar
    fn navigate_to_address(&mut self) {
        let url = self.address_bar_text.trim();
        if !url.is_empty() {
            tracing::info!("Navigating to: {}", url);

            // Add protocol if missing
            let full_url = if url.starts_with("http://") || url.starts_with("https://") {
                url.to_string()
            } else if url.contains('.') && !url.contains(' ') {
                format!("https://{}", url)
            } else {
                // Treat as search query
                format!("https://www.google.com/search?q={}", urlencoding::encode(url))
            };

            // Set pending navigation for event loop to process
            self.pending_navigation = Some(full_url.clone());
            self.set_current_url(&full_url);
        }
    }
}
