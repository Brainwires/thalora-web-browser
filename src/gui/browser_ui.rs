//! Browser UI components using egui
//! 
//! This module provides the browser's user interface including address bar, navigation buttons,
//! tabs, and other browser chrome elements.

use egui::{Context, TopBottomPanel, CentralPanel, SidePanel, Button, TextEdit, Ui};
use crate::gui::TabManager;

/// Navigation state for the browser
#[derive(Default)]
pub struct NavigationState {
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_loading: bool,
    pub current_url: String,
    pub page_title: String,
}

/// Main browser UI manager
pub struct BrowserUI {
    navigation_state: NavigationState,
    address_bar_text: String,
    is_editing_address: bool,
    debug_mode: bool,
    show_dev_tools: bool,
    search_text: String,
    pending_navigation: Option<String>,
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
        }
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

    /// Show the navigation bar with back/forward buttons and address bar
    fn show_navigation_bar(&mut self, ui: &mut Ui, tab_manager: &TabManager) {
        ui.horizontal(|ui| {
            // Back button
            if ui.add_enabled(
                self.navigation_state.can_go_back,
                Button::new("◀")
            ).clicked() && self.navigation_state.can_go_back {
                // TODO: Implement back navigation
                tracing::debug!("Back button clicked");
            }

            // Forward button
            if ui.add_enabled(
                self.navigation_state.can_go_forward,
                Button::new("▶")
            ).clicked() && self.navigation_state.can_go_forward {
                // TODO: Implement forward navigation
                tracing::debug!("Forward button clicked");
            }

            // Reload button
            let reload_text = if self.navigation_state.is_loading { "⊗" } else { "⟲" };
            if ui.add(Button::new(reload_text)).clicked() {
                if self.navigation_state.is_loading {
                    // TODO: Stop loading
                    tracing::debug!("Stop loading clicked");
                } else {
                    // TODO: Reload page
                    tracing::debug!("Reload clicked");
                }
            }

            // Address bar
            ui.add_space(8.0);
            let address_response = ui.add(
                TextEdit::singleline(&mut self.address_bar_text)
                    .desired_width(f32::INFINITY)
                    .hint_text("Enter URL or search...")
            );

            // Track if user is actively editing
            if address_response.has_focus() {
                self.is_editing_address = true;
            }

            if address_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.navigate_to_address();
                self.is_editing_address = false;
            }

            // Also clear editing flag if focus lost without Enter
            if address_response.lost_focus() && !ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.is_editing_address = false;
            }

            // Menu button
            if ui.add(Button::new("≡")).clicked() {
                // TODO: Show browser menu
                tracing::debug!("Menu button clicked");
            }
        });

        // Tab bar
        ui.horizontal(|ui| {
            self.show_tab_bar(ui, tab_manager);
        });
    }

    /// Show the tab bar
    fn show_tab_bar(&mut self, ui: &mut Ui, tab_manager: &TabManager) {
        ui.horizontal(|ui| {
            // Render tabs
            for tab in tab_manager.tabs() {
                let is_active = Some(tab.id()) == tab_manager.current_tab_id();
                
                let tab_title = if tab.title().is_empty() {
                    "New Tab".to_string()
                } else {
                    tab.title().to_string()
                };

                let mut tab_button = Button::new(&tab_title);
                if is_active {
                    tab_button = tab_button.fill(egui::Color32::from_gray(100));
                }

                if ui.add(tab_button).clicked() {
                    // TODO: Switch to this tab
                    tracing::debug!("Switching to tab: {}", tab.id());
                }

                // Close button for tab
                if ui.add(Button::new("×").small()).clicked() {
                    // TODO: Close this tab
                    tracing::debug!("Closing tab: {}", tab.id());
                }
            }

            // New tab button
            if ui.add(Button::new("+")).clicked() {
                // TODO: Create new tab
                tracing::debug!("Creating new tab");
            }
        });
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
                    // Display the rendered DOM content
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            // Get rendered DOM content
                            let content = pollster::block_on(tab.get_rendered_dom()).unwrap_or_default();

                            // Debug: print content length
                            tracing::debug!("Content length: {} bytes", content.len());

                            // Only show "no content" if truly empty or just whitespace
                            let trimmed = content.trim();
                            if trimmed.is_empty() || trimmed == "about:blank" {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(50.0);
                                    ui.heading(tab.title());
                                    ui.label("No content loaded");
                                });
                            } else {
                                // Parse and render the HTML DOM using scraper
                                use scraper::{Html, Selector};

                                // For now, extract and display text content from HTML
                                // TODO: Implement proper HTML rendering with layout
                                let document = Html::parse_document(&content);

                                // Try to extract body text
                                if let Ok(body_selector) = Selector::parse("body") {
                                    if let Some(body) = document.select(&body_selector).next() {
                                        let text = body.text().collect::<Vec<_>>().join(" ");
                                        let trimmed_text = text.trim();

                                        if !trimmed_text.is_empty() {
                                            ui.label(trimmed_text);
                                        } else {
                                            ui.label("Content loaded but body is empty");
                                        }
                                    } else {
                                        // No body tag, show all text
                                        let text = document.root_element().text().collect::<Vec<_>>().join(" ");
                                        ui.label(text.trim());
                                    }
                                } else {
                                    // Selector failed, show all text
                                    let text = document.root_element().text().collect::<Vec<_>>().join(" ");
                                    ui.label(text.trim());
                                }
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
                    // TODO: Create initial tab
                    tracing::debug!("Creating initial tab");
                }
            });
        }
    }

    /// Render a DOM element from Boa's HTML
    fn render_dom_element(&self, ui: &mut Ui, element: scraper::ElementRef) {
        use scraper::node::Node;

        let tag_name = element.value().name();

        match tag_name {
            "h1" => {
                let text: String = element.text().collect();
                if !text.trim().is_empty() {
                    ui.heading(text.trim());
                    ui.add_space(6.0);
                }
            }
            "h2" => {
                let text: String = element.text().collect();
                if !text.trim().is_empty() {
                    ui.label(egui::RichText::new(text.trim()).heading().strong());
                    ui.add_space(5.0);
                }
            }
            "h3" | "h4" | "h5" | "h6" => {
                let text: String = element.text().collect();
                if !text.trim().is_empty() {
                    ui.label(egui::RichText::new(text.trim()).strong().size(16.0));
                    ui.add_space(4.0);
                }
            }
            "p" => {
                ui.horizontal_wrapped(|ui| {
                    for child in element.children() {
                        match child.value() {
                            Node::Text(text) => {
                                let content = text.trim();
                                if !content.is_empty() {
                                    ui.label(content);
                                }
                            }
                            Node::Element(_) => {
                                if let Some(child_elem) = scraper::ElementRef::wrap(child) {
                                    self.render_inline_element(ui, child_elem);
                                }
                            }
                            _ => {}
                        }
                    }
                });
                ui.add_space(8.0);
            }
            "a" => {
                let text: String = element.text().collect();
                let href = element.value().attr("href").unwrap_or("#");
                if !text.trim().is_empty() {
                    if ui.link(text.trim()).clicked() {
                        tracing::info!("Link clicked: {}", href);
                        // TODO: Navigate to link
                    }
                }
            }
            "ul" | "ol" => {
                for child in element.children() {
                    if let Some(child_element) = scraper::ElementRef::wrap(child) {
                        if child_element.value().name() == "li" {
                            let text: String = child_element.text().collect();
                            if !text.trim().is_empty() {
                                ui.horizontal(|ui| {
                                    ui.label("•");
                                    ui.label(text.trim());
                                });
                            }
                        }
                    }
                }
                ui.add_space(6.0);
            }
            "div" | "section" | "article" | "main" | "aside" | "header" | "footer" | "nav" => {
                // Render children recursively
                for child in element.children() {
                    if let Some(child_element) = scraper::ElementRef::wrap(child) {
                        self.render_dom_element(ui, child_element);
                    }
                }
            }
            "br" => {
                ui.add_space(4.0);
            }
            "hr" => {
                ui.separator();
                ui.add_space(6.0);
            }
            "pre" | "code" => {
                let text: String = element.text().collect();
                if !text.trim().is_empty() {
                    ui.code(text.trim());
                    ui.add_space(4.0);
                }
            }
            "script" | "style" | "head" | "meta" | "link" | "title" => {
                // Skip non-visual elements
            }
            _ => {
                // For unknown elements, render children or text
                for child in element.children() {
                    match child.value() {
                        Node::Text(text) => {
                            let content = text.trim();
                            if !content.is_empty() {
                                ui.label(content);
                            }
                        }
                        Node::Element(_) => {
                            if let Some(child_element) = scraper::ElementRef::wrap(child) {
                                self.render_dom_element(ui, child_element);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Render inline elements (like <strong>, <em>, etc.)
    fn render_inline_element(&self, ui: &mut Ui, element: scraper::ElementRef) {
        let tag_name = element.value().name();
        let text: String = element.text().collect();

        if text.trim().is_empty() {
            return;
        }

        match tag_name {
            "strong" | "b" => {
                ui.label(egui::RichText::new(text.trim()).strong());
            }
            "em" | "i" => {
                ui.label(egui::RichText::new(text.trim()).italics());
            }
            "code" => {
                ui.code(text.trim());
            }
            "a" => {
                let href = element.value().attr("href").unwrap_or("#");
                if ui.link(text.trim()).clicked() {
                    tracing::info!("Link clicked: {}", href);
                    // TODO: Navigate to link
                }
            }
            _ => {
                ui.label(text.trim());
            }
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
                ui.text_edit_singleline(&mut self.search_text);
                if ui.button("Execute").clicked() {
                    // TODO: Execute JavaScript in current tab
                    tracing::debug!("Executing JS: {}", self.search_text);
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
                // TODO: Create new tab
            }

            // Ctrl+W - Close tab
            if i.modifiers.ctrl && i.key_pressed(egui::Key::W) {
                tracing::debug!("Keyboard shortcut: Close tab");
                // TODO: Close current tab
            }

            // Ctrl+R - Reload
            if i.modifiers.ctrl && i.key_pressed(egui::Key::R) {
                tracing::debug!("Keyboard shortcut: Reload");
                // TODO: Reload current tab
            }

            // F12 - Toggle dev tools
            if i.key_pressed(egui::Key::F12) {
                self.show_dev_tools = !self.show_dev_tools;
                tracing::debug!("Toggled dev tools: {}", self.show_dev_tools);
            }

            // Ctrl+L - Focus address bar
            if i.modifiers.ctrl && i.key_pressed(egui::Key::L) {
                tracing::debug!("Keyboard shortcut: Focus address bar");
                // TODO: Focus address bar
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

    /// Set the current URL in the UI
    pub fn set_current_url(&mut self, url: &str) {
        self.navigation_state.current_url = url.to_string();
        self.address_bar_text = url.to_string();
    }

    /// Set the current page title
    pub fn set_page_title(&mut self, title: &str) {
        self.navigation_state.page_title = title.to_string();
    }

    /// Update UI state from the active tab
    pub fn update_from_tab(&mut self, tab: &crate::gui::Tab) {
        // Only update address bar if user is not currently editing it
        if !self.is_editing_address {
            self.set_current_url(tab.url());
        }
        self.set_page_title(tab.title());
        self.set_loading(tab.is_loading());
        self.set_navigation_state(tab.can_go_back(), tab.can_go_forward());
    }

    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.navigation_state.is_loading = loading;
    }

    /// Set navigation state
    pub fn set_navigation_state(&mut self, can_go_back: bool, can_go_forward: bool) {
        self.navigation_state.can_go_back = can_go_back;
        self.navigation_state.can_go_forward = can_go_forward;
    }

    /// Get current navigation state
    pub fn navigation_state(&self) -> &NavigationState {
        &self.navigation_state
    }

    /// Take pending navigation URL if one exists
    pub fn take_pending_navigation(&mut self) -> Option<String> {
        self.pending_navigation.take()
    }
}