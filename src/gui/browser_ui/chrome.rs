//! Browser chrome components (navigation bar, tabs)

use egui::{Button, TextEdit, Ui};
use super::types::*;
use crate::gui::TabManager;

impl super::BrowserUI {
    /// Show the navigation bar with back/forward buttons and address bar
    pub(super) fn show_navigation_bar(&mut self, ui: &mut Ui, tab_manager: &TabManager) {
        ui.horizontal(|ui| {
            // Back button
            if ui.add_enabled(
                self.navigation_state.can_go_back,
                Button::new("◀").min_size(egui::vec2(28.0, 24.0))
            ).on_hover_text("Go back (Alt+Left)")
             .clicked() && self.navigation_state.can_go_back {
                tracing::debug!("Back button clicked");
                self.set_pending_action(BrowserAction::GoBack);
            }

            // Forward button
            if ui.add_enabled(
                self.navigation_state.can_go_forward,
                Button::new("▶").min_size(egui::vec2(28.0, 24.0))
            ).on_hover_text("Go forward (Alt+Right)")
             .clicked() && self.navigation_state.can_go_forward {
                tracing::debug!("Forward button clicked");
                self.set_pending_action(BrowserAction::GoForward);
            }

            // Reload/Stop button
            let (reload_text, tooltip) = if self.navigation_state.is_loading {
                ("⊗", "Stop loading (Esc)")
            } else {
                ("⟲", "Reload page (Ctrl+R)")
            };
            if ui.add(Button::new(reload_text).min_size(egui::vec2(28.0, 24.0)))
                .on_hover_text(tooltip)
                .clicked() {
                if self.navigation_state.is_loading {
                    tracing::debug!("Stop loading clicked");
                    self.set_pending_action(BrowserAction::StopLoading);
                } else {
                    tracing::debug!("Reload clicked");
                    self.set_pending_action(BrowserAction::Reload);
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
            if ui.add(Button::new("≡").min_size(egui::vec2(28.0, 24.0)))
                .on_hover_text("Menu")
                .clicked() {
                tracing::debug!("Menu button clicked");
                self.set_pending_action(BrowserAction::ShowMenu);
            }
        });

        // Tab bar
        ui.horizontal(|ui| {
            self.show_tab_bar(ui, tab_manager);
        });
    }

    /// Show the tab bar
    pub(super) fn show_tab_bar(&mut self, ui: &mut Ui, tab_manager: &TabManager) {
        ui.horizontal(|ui| {
            // Render tabs
            for tab in tab_manager.tabs() {
                let is_active = Some(tab.id()) == tab_manager.current_tab_id();
                let tab_id = tab.id();

                let tab_title = if tab.title().is_empty() {
                    "New Tab".to_string()
                } else {
                    // Truncate long titles
                    let title = tab.title().to_string();
                    if title.len() > 20 {
                        format!("{}...", &title[..17])
                    } else {
                        title
                    }
                };

                // Tab button with active styling
                let mut tab_button = Button::new(&tab_title);
                if is_active {
                    tab_button = tab_button.fill(egui::Color32::from_gray(100));
                }

                ui.horizontal(|ui| {
                    if ui.add(tab_button).on_hover_text(tab.url()).clicked() {
                        if !is_active {
                            tracing::debug!("Switching to tab: {}", tab_id);
                            self.set_pending_action(BrowserAction::SwitchTab(tab_id));
                        }
                    }

                    // Close button for tab
                    if ui.add(Button::new("×").small())
                        .on_hover_text("Close tab (Ctrl+W)")
                        .clicked() {
                        tracing::debug!("Closing tab: {}", tab_id);
                        self.set_pending_action(BrowserAction::CloseTab(tab_id));
                    }
                });
            }

            // New tab button
            if ui.add(Button::new("+"))
                .on_hover_text("New tab (Ctrl+T)")
                .clicked() {
                tracing::debug!("Creating new tab");
                self.set_pending_action(BrowserAction::NewTab);
            }
        });
    }
}
