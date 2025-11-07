//! Browser UI components using egui
//!
//! This module provides the browser's user interface including address bar, navigation buttons,
//! tabs, and other browser chrome elements.

use egui::{Context, TopBottomPanel, CentralPanel, SidePanel, Button, TextEdit, Ui, FontId, FontFamily, RichText};
use crate::gui::{TabManager, FontManager, FontDescriptor, FontWeight, FontStyle, FontSize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DOM element structure from JavaScript query
#[derive(Debug, Clone, Deserialize, Serialize)]
struct DomElement {
    tag: String,
    text: String,
    attrs: HashMap<String, String>,
    style: ElementStyle,
    children: Vec<DomElement>,
}

/// Computed style from getComputedStyle
#[derive(Debug, Clone, Deserialize, Serialize)]
struct ElementStyle {
    color: String,
    #[serde(rename = "backgroundColor")]
    background_color: String,
    #[serde(rename = "fontSize")]
    font_size: String,
    #[serde(rename = "fontWeight")]
    font_weight: String,
    #[serde(rename = "fontFamily")]
    font_family: String,
    #[serde(rename = "textDecoration")]
    text_decoration: String,
    display: String,
    #[serde(rename = "marginTop")]
    margin_top: String,
    #[serde(rename = "marginBottom")]
    margin_bottom: String,
    #[serde(rename = "paddingTop")]
    padding_top: String,
    #[serde(rename = "paddingBottom")]
    padding_bottom: String,
}

/// CSS spacing (for padding/margin)
#[derive(Debug, Clone, Default)]
struct CssSpacing {
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
}

/// Text alignment
#[derive(Debug, Clone)]
enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

/// Display type
#[derive(Debug, Clone)]
enum DisplayType {
    Block,
    Inline,
    Flex,
    None,
}

/// Parsed CSS style properties
#[derive(Debug, Clone, Default)]
struct CssStyle {
    text_color: Option<egui::Color32>,
    bg_color: Option<egui::Color32>,
    font_size: Option<f32>,
    font_weight: Option<bool>, // true = bold
    font_family: Option<String>,
    text_align: Option<TextAlign>,
    padding: CssSpacing,
    margin: CssSpacing,
    border_width: Option<f32>,
    border_color: Option<egui::Color32>,
    border_radius: Option<f32>,
    display: Option<DisplayType>,
    width: Option<f32>,
    height: Option<f32>,
}

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
            font_manager: FontManager::new(),
            base_font_size: 16.0,
        }
    }

    /// Initialize fonts for the context
    pub fn init_fonts(&self, ctx: &Context) {
        self.font_manager.install_fonts(ctx);
    }

    /// Create RichText with CSS styling applied
    fn create_styled_text(&self, text: &str, css: &CssStyle, default_size: f32) -> RichText {
        let font_size = css.font_size.unwrap_or(default_size);
        let mut rich_text = RichText::new(text).size(font_size);

        // Apply color
        if let Some(color) = css.text_color {
            rich_text = rich_text.color(color);
        }

        // Apply background color
        if let Some(bg_color) = css.bg_color {
            rich_text = rich_text.background_color(bg_color);
        }

        // Apply bold
        if css.font_weight.unwrap_or(false) {
            rich_text = rich_text.strong();
        }

        // Apply font family
        if let Some(family) = &css.font_family {
            let descriptor = FontDescriptor::new(family.clone(), font_size);
            rich_text = rich_text.font(descriptor.to_egui_font_id());
        }

        rich_text
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
                    // TODO: Create initial tab
                    tracing::debug!("Creating initial tab");
                }
            });
        }
    }

    /// Collect text from element, excluding style tags only (scripts should be executed, not displayed)
    fn collect_visible_text(&self, element: scraper::ElementRef) -> String {
        use scraper::node::Node;

        let mut text = String::new();
        for child in element.children() {
            match child.value() {
                Node::Text(t) => text.push_str(t.text.as_ref()),
                Node::Element(_) => {
                    if let Some(child_elem) = scraper::ElementRef::wrap(child) {
                        let tag = child_elem.value().name();
                        // Only skip style and meta tags from text display
                        // Scripts are handled separately by execution engine
                        if !matches!(tag, "style" | "head" | "meta" | "link" | "title") {
                            text.push_str(&self.collect_visible_text(child_elem));
                        }
                    }
                }
                _ => {}
            }
        }
        text
    }

    /// Render a DOM element from Boa's HTML
    fn render_dom_element(&self, ui: &mut Ui, element: scraper::ElementRef) {
        use scraper::node::Node;

        let tag_name = element.value().name();

        match tag_name {
            "h1" => {
                let text = self.collect_visible_text(element);
                if !text.trim().is_empty() {
                    let style_attr = element.value().attr("style").unwrap_or("");
                    let css = self.parse_inline_style(style_attr);

                    ui.add_space(css.margin.top.max(12.0));
                    let font_size = css.font_size.unwrap_or_else(|| FontSize::heading_size(1));
                    let mut rich_text = RichText::new(text.trim())
                        .size(font_size)
                        .strong();

                    if let Some(color) = css.text_color {
                        rich_text = rich_text.color(color);
                    }

                    // Apply font family if specified
                    if let Some(family) = &css.font_family {
                        let descriptor = FontDescriptor::new(family.clone(), font_size);
                        rich_text = rich_text.font(descriptor.to_egui_font_id());
                    }

                    ui.label(rich_text);
                    ui.add_space(css.margin.bottom.max(12.0));
                }
            }
            "h2" => {
                let text = self.collect_visible_text(element);
                if !text.trim().is_empty() {
                    let style_attr = element.value().attr("style").unwrap_or("");
                    let css = self.parse_inline_style(style_attr);

                    ui.add_space(css.margin.top.max(10.0));
                    let font_size = css.font_size.unwrap_or_else(|| FontSize::heading_size(2));
                    let mut rich_text = RichText::new(text.trim())
                        .size(font_size)
                        .strong();

                    if let Some(color) = css.text_color {
                        rich_text = rich_text.color(color);
                    }

                    if let Some(family) = &css.font_family {
                        let descriptor = FontDescriptor::new(family.clone(), font_size);
                        rich_text = rich_text.font(descriptor.to_egui_font_id());
                    }

                    ui.label(rich_text);
                    ui.add_space(css.margin.bottom.max(10.0));
                }
            }
            "h3" => {
                let text = self.collect_visible_text(element);
                if !text.trim().is_empty() {
                    let style_attr = element.value().attr("style").unwrap_or("");
                    let css = self.parse_inline_style(style_attr);

                    ui.add_space(css.margin.top.max(8.0));
                    let font_size = css.font_size.unwrap_or_else(|| FontSize::heading_size(3));
                    let mut rich_text = RichText::new(text.trim())
                        .size(font_size)
                        .strong();

                    if let Some(color) = css.text_color {
                        rich_text = rich_text.color(color);
                    }

                    if let Some(family) = &css.font_family {
                        let descriptor = FontDescriptor::new(family.clone(), font_size);
                        rich_text = rich_text.font(descriptor.to_egui_font_id());
                    }

                    ui.label(rich_text);
                    ui.add_space(css.margin.bottom.max(8.0));
                }
            }
            "h4" | "h5" | "h6" => {
                let text = self.collect_visible_text(element);
                if !text.trim().is_empty() {
                    let style_attr = element.value().attr("style").unwrap_or("");
                    let css = self.parse_inline_style(style_attr);

                    let level = match tag_name {
                        "h4" => 4,
                        "h5" => 5,
                        "h6" => 6,
                        _ => 4,
                    };

                    ui.add_space(css.margin.top.max(6.0));
                    let font_size = css.font_size.unwrap_or_else(|| FontSize::heading_size(level));
                    let mut rich_text = RichText::new(text.trim())
                        .size(font_size)
                        .strong();

                    if let Some(color) = css.text_color {
                        rich_text = rich_text.color(color);
                    }

                    if let Some(family) = &css.font_family {
                        let descriptor = FontDescriptor::new(family.clone(), font_size);
                        rich_text = rich_text.font(descriptor.to_egui_font_id());
                    }

                    ui.label(rich_text);
                    ui.add_space(css.margin.bottom.max(6.0));
                }
            }
            "p" => {
                let style_attr = element.value().attr("style").unwrap_or("");
                let css = self.parse_inline_style(style_attr);

                ui.add_space(css.margin.top);

                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;

                    // Apply text color if specified
                    if let Some(color) = css.text_color {
                        ui.style_mut().visuals.override_text_color = Some(color);
                    }

                    for child in element.children() {
                        match child.value() {
                            Node::Text(text) => {
                                if !text.trim().is_empty() {
                                    let mut rich_text = egui::RichText::new(text.text.as_ref())
                                        .size(css.font_size.unwrap_or(14.0));

                                    if css.font_weight.unwrap_or(false) {
                                        rich_text = rich_text.strong();
                                    }

                                    ui.label(rich_text);
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
                ui.add_space(css.margin.bottom.max(10.0));
            }
            "a" => {
                let text = self.collect_visible_text(element);
                let href = element.value().attr("href").unwrap_or("#");
                if !text.trim().is_empty() {
                    let link = egui::RichText::new(text.trim())
                        .size(14.0)
                        .color(egui::Color32::from_rgb(0, 102, 204))
                        .underline();
                    if ui.add(egui::Label::new(link).sense(egui::Sense::click())).clicked() {
                        tracing::info!("Link clicked: {}", href);
                        // TODO: Navigate to link
                    }
                }
            }
            "ul" | "ol" => {
                ui.add_space(4.0);
                ui.indent("list", |ui| {
                    for child in element.children() {
                        if let Some(child_element) = scraper::ElementRef::wrap(child) {
                            if child_element.value().name() == "li" {
                                let text = self.collect_visible_text(child_element);
                                if !text.trim().is_empty() {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("•").size(14.0));
                                        ui.label(egui::RichText::new(text.trim()).size(14.0));
                                    });
                                    ui.add_space(2.0);
                                }
                            }
                        }
                    }
                });
                ui.add_space(8.0);
            }
            "input" => {
                let input_type = element.value().attr("type").unwrap_or("text");
                let placeholder = element.value().attr("placeholder").unwrap_or("");
                let value = element.value().attr("value").unwrap_or("");
                let name = element.value().attr("name").unwrap_or("");

                match input_type {
                    "text" | "email" | "password" | "search" | "url" => {
                        ui.horizontal(|ui| {
                            if let Some(label_text) = element.value().attr("aria-label").or(element.value().attr("placeholder")) {
                                ui.label(format!("{}:", label_text));
                            }
                            let mut text = value.to_string();
                            ui.add(egui::TextEdit::singleline(&mut text)
                                .hint_text(placeholder)
                                .desired_width(200.0));
                        });
                        ui.add_space(8.0);
                    }
                    "submit" | "button" => {
                        let button_text = value.to_string();
                        if ui.button(&button_text).clicked() {
                            tracing::info!("Button clicked: {}", button_text);
                        }
                        ui.add_space(4.0);
                    }
                    "checkbox" => {
                        let mut checked = element.value().attr("checked").is_some();
                        ui.checkbox(&mut checked, name);
                        ui.add_space(4.0);
                    }
                    "radio" => {
                        let mut selected = element.value().attr("checked").is_some();
                        ui.radio_value(&mut selected, true, name);
                        ui.add_space(4.0);
                    }
                    _ => {}
                }
            }
            "button" => {
                let text = self.collect_visible_text(element);
                if !text.trim().is_empty() {
                    if ui.button(text.trim()).clicked() {
                        tracing::info!("Button clicked: {}", text.trim());
                    }
                    ui.add_space(4.0);
                }
            }
            "textarea" => {
                let placeholder = element.value().attr("placeholder").unwrap_or("");
                let mut text = self.collect_visible_text(element);
                ui.add(egui::TextEdit::multiline(&mut text)
                    .hint_text(placeholder)
                    .desired_width(f32::INFINITY)
                    .desired_rows(4));
                ui.add_space(8.0);
            }
            "select" => {
                ui.label("Dropdown:");
                // TODO: Implement proper dropdown with options
                ui.add_space(4.0);
            }
            "form" => {
                egui::Frame::none()
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200)))
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        for child in element.children() {
                            if let Some(child_element) = scraper::ElementRef::wrap(child) {
                                self.render_dom_element(ui, child_element);
                            }
                        }
                    });
                ui.add_space(10.0);
            }
            "div" | "section" | "article" | "main" | "aside" => {
                // Parse inline styles
                let style_attr = element.value().attr("style").unwrap_or("");
                let css = self.parse_inline_style(style_attr);

                // Skip display: none elements
                if matches!(css.display, Some(DisplayType::None)) {
                    return;
                }

                // Apply margin
                ui.add_space(css.margin.top);

                // Create frame with styling
                let has_styling = css.bg_color.is_some() || css.border_width.is_some() || css.padding.top > 0.0;

                if has_styling {
                    let mut frame = egui::Frame::none();

                    // Apply padding
                    frame = frame.inner_margin(egui::Margin {
                        left: css.padding.left,
                        right: css.padding.right,
                        top: css.padding.top,
                        bottom: css.padding.bottom,
                    });

                    // Apply background color
                    if let Some(bg) = css.bg_color {
                        frame = frame.fill(bg);
                    }

                    // Apply border
                    if let Some(border_width) = css.border_width {
                        let border_color = css.border_color.unwrap_or(egui::Color32::from_rgb(200, 200, 200));
                        frame = frame.stroke(egui::Stroke::new(border_width, border_color));
                    }

                    // Apply border radius if specified
                    if let Some(radius) = css.border_radius {
                        frame = frame.rounding(egui::Rounding::same(radius));
                    }

                    frame.show(ui, |ui| {
                        // Apply text color if specified
                        if let Some(color) = css.text_color {
                            ui.style_mut().visuals.override_text_color = Some(color);
                        }

                        for child in element.children() {
                            if let Some(child_element) = scraper::ElementRef::wrap(child) {
                                self.render_dom_element(ui, child_element);
                            }
                        }
                    });
                } else {
                    for child in element.children() {
                        if let Some(child_element) = scraper::ElementRef::wrap(child) {
                            self.render_dom_element(ui, child_element);
                        }
                    }
                }

                // Apply bottom margin
                ui.add_space(css.margin.bottom);
            }
            "header" | "footer" | "nav" => {
                ui.add_space(8.0);
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(248, 248, 248))
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        for child in element.children() {
                            if let Some(child_element) = scraper::ElementRef::wrap(child) {
                                self.render_dom_element(ui, child_element);
                            }
                        }
                    });
                ui.add_space(8.0);
            }
            "br" => {
                ui.label("");
            }
            "hr" => {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);
            }
            "pre" => {
                let text = self.collect_visible_text(element);
                if !text.trim().is_empty() {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(245, 245, 245))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 220, 220)))
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.add(egui::Label::new(egui::RichText::new(text.as_str())
                                .font(egui::FontId::monospace(12.0))
                                .color(egui::Color32::from_rgb(50, 50, 50))));
                        });
                    ui.add_space(10.0);
                }
            }
            "code" => {
                let text = self.collect_visible_text(element);
                if !text.trim().is_empty() {
                    ui.label(egui::RichText::new(text.trim())
                        .font(egui::FontId::monospace(13.0))
                        .background_color(egui::Color32::from_rgb(245, 245, 245))
                        .color(egui::Color32::from_rgb(214, 51, 132)));
                }
            }
            "img" => {
                let alt = element.value().attr("alt").unwrap_or("Image");
                let width = element.value().attr("width");
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(240, 240, 240))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200)))
                    .inner_margin(egui::Margin::same(20.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(format!("🖼 {}", alt))
                            .italics()
                            .color(egui::Color32::GRAY));
                    });
                ui.add_space(8.0);
            }
            "span" => {
                // Inline element with CSS styling
                let style_attr = element.value().attr("style").unwrap_or("");
                let css = self.parse_inline_style(style_attr);

                // Render text content with styling
                let text = self.collect_visible_text(element);
                if !text.trim().is_empty() {
                    let mut rich_text = egui::RichText::new(text.trim())
                        .size(css.font_size.unwrap_or(14.0));

                    if css.font_weight.unwrap_or(false) {
                        rich_text = rich_text.strong();
                    }

                    if let Some(color) = css.text_color {
                        rich_text = rich_text.color(color);
                    }

                    if let Some(bg_color) = css.bg_color {
                        rich_text = rich_text.background_color(bg_color);
                    }

                    ui.label(rich_text);
                }
            }
            "script" | "style" | "head" | "meta" | "link" | "title" | "noscript" => {
                // Skip non-visual elements
            }
            _ => {
                // For unknown elements, check if they have inline styles
                let style_attr = element.value().attr("style").unwrap_or("");
                let css = self.parse_inline_style(style_attr);

                // Skip display: none elements
                if matches!(css.display, Some(DisplayType::None)) {
                    return;
                }

                // Render children or text
                for child in element.children() {
                    match child.value() {
                        Node::Text(text) => {
                            let content = text.trim();
                            if !content.is_empty() {
                                let mut rich_text = egui::RichText::new(content)
                                    .size(css.font_size.unwrap_or(14.0));

                                if let Some(color) = css.text_color {
                                    rich_text = rich_text.color(color);
                                }

                                if css.font_weight.unwrap_or(false) {
                                    rich_text = rich_text.strong();
                                }

                                ui.label(rich_text);
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

    /// Render structured DOM element with proper styling from computed styles
    fn render_structured_element(&mut self, ui: &mut Ui, element: &DomElement, depth: usize) {
        // Skip script, style, head elements
        if matches!(element.tag.as_str(), "script" | "style" | "head" | "meta" | "link") {
            return;
        }

        // Parse font size
        let font_size = self.parse_px(&element.style.font_size).unwrap_or(14.0);

        // Parse color (simple RGB parser)
        let text_color = self.parse_color(&element.style.color);

        // Apply margins
        let margin_top = self.parse_px(&element.style.margin_top).unwrap_or(0.0);
        let margin_bottom = self.parse_px(&element.style.margin_bottom).unwrap_or(0.0);

        if margin_top > 0.0 {
            ui.add_space(margin_top);
        }

        // Render based on tag
        match element.tag.as_str() {
            "h1" => {
                let text = self.collect_all_text(element);
                if !text.trim().is_empty() {
                    let mut rich_text = egui::RichText::new(text.trim()).size(32.0).strong();
                    if let Some(color) = text_color {
                        rich_text = rich_text.color(color);
                    }
                    ui.label(rich_text);
                }
            }
            "h2" => {
                let text = self.collect_all_text(element);
                if !text.trim().is_empty() {
                    let mut rich_text = egui::RichText::new(text.trim()).size(24.0).strong();
                    if let Some(color) = text_color {
                        rich_text = rich_text.color(color);
                    }
                    ui.label(rich_text);
                }
            }
            "h3" => {
                let text = self.collect_all_text(element);
                if !text.trim().is_empty() {
                    let mut rich_text = egui::RichText::new(text.trim()).size(20.0).strong();
                    if let Some(color) = text_color {
                        rich_text = rich_text.color(color);
                    }
                    ui.label(rich_text);
                }
            }
            "h4" | "h5" | "h6" => {
                let text = self.collect_all_text(element);
                if !text.trim().is_empty() {
                    let mut rich_text = egui::RichText::new(text.trim()).size(16.0).strong();
                    if let Some(color) = text_color {
                        rich_text = rich_text.color(color);
                    }
                    ui.label(rich_text);
                }
            }
            "p" => {
                if !element.text.trim().is_empty() || !element.children.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        if !element.text.trim().is_empty() {
                            let mut rich_text = egui::RichText::new(element.text.trim()).size(font_size);
                            if let Some(color) = text_color {
                                rich_text = rich_text.color(color);
                            }
                            ui.label(rich_text);
                        }
                        for child in &element.children {
                            self.render_inline_structured_element(ui, child, font_size, text_color);
                        }
                    });
                }
            }
            "a" => {
                let text = self.collect_all_text(element);
                if !text.trim().is_empty() {
                    let link_text = egui::RichText::new(text.trim())
                        .size(font_size)
                        .color(egui::Color32::from_rgb(0, 0, 238))
                        .underline();

                    if ui.add(egui::Label::new(link_text).sense(egui::Sense::click())).clicked() {
                        if let Some(href) = element.attrs.get("href") {
                            tracing::info!("Link clicked: {}", href);
                            // TODO: Navigate to link
                            self.pending_navigation = Some(href.clone());
                        }
                    }
                }
            }
            "ul" | "ol" => {
                for child in &element.children {
                    if child.tag == "li" {
                        ui.horizontal(|ui| {
                            ui.label("•");
                            let text = self.collect_all_text(child);
                            if !text.trim().is_empty() {
                                let mut rich_text = egui::RichText::new(text.trim()).size(font_size);
                                if let Some(color) = text_color {
                                    rich_text = rich_text.color(color);
                                }
                                ui.label(rich_text);
                            }
                        });
                    }
                }
            }
            "strong" | "b" => {
                let text = self.collect_all_text(element);
                if !text.trim().is_empty() {
                    let mut rich_text = egui::RichText::new(text.trim()).size(font_size).strong();
                    if let Some(color) = text_color {
                        rich_text = rich_text.color(color);
                    }
                    ui.label(rich_text);
                }
            }
            "em" | "i" => {
                let text = self.collect_all_text(element);
                if !text.trim().is_empty() {
                    let mut rich_text = egui::RichText::new(text.trim()).size(font_size).italics();
                    if let Some(color) = text_color {
                        rich_text = rich_text.color(color);
                    }
                    ui.label(rich_text);
                }
            }
            "code" | "pre" => {
                let text = self.collect_all_text(element);
                if !text.trim().is_empty() {
                    ui.code(text.trim());
                }
            }
            "br" => {
                ui.add_space(4.0);
            }
            "hr" => {
                ui.separator();
            }
            // Container elements - render children
            "div" | "section" | "article" | "main" | "aside" | "header" | "footer" | "nav" | "body" | "html" => {
                if !element.text.trim().is_empty() {
                    let mut rich_text = egui::RichText::new(element.text.trim()).size(font_size);
                    if let Some(color) = text_color {
                        rich_text = rich_text.color(color);
                    }
                    ui.label(rich_text);
                }
                for child in &element.children {
                    self.render_structured_element(ui, child, depth + 1);
                }
            }
            _ => {
                // Unknown element - render text and children
                if !element.text.trim().is_empty() {
                    let mut rich_text = egui::RichText::new(element.text.trim()).size(font_size);
                    if let Some(color) = text_color {
                        rich_text = rich_text.color(color);
                    }
                    ui.label(rich_text);
                }
                for child in &element.children {
                    self.render_structured_element(ui, child, depth + 1);
                }
            }
        }

        if margin_bottom > 0.0 {
            ui.add_space(margin_bottom);
        }
    }

    /// Render inline structured element
    fn render_inline_structured_element(&mut self, ui: &mut Ui, element: &DomElement, font_size: f32, parent_color: Option<egui::Color32>) {
        match element.tag.as_str() {
            "a" => {
                let text = self.collect_all_text(element);
                if !text.trim().is_empty() {
                    let link_text = egui::RichText::new(text.trim())
                        .size(font_size)
                        .color(egui::Color32::from_rgb(0, 0, 238))
                        .underline();

                    if ui.add(egui::Label::new(link_text).sense(egui::Sense::click())).clicked() {
                        if let Some(href) = element.attrs.get("href") {
                            self.pending_navigation = Some(href.clone());
                        }
                    }
                }
            }
            "strong" | "b" => {
                let text = self.collect_all_text(element);
                if !text.trim().is_empty() {
                    let mut rich_text = egui::RichText::new(text.trim()).size(font_size).strong();
                    if let Some(color) = parent_color {
                        rich_text = rich_text.color(color);
                    }
                    ui.label(rich_text);
                }
            }
            "em" | "i" => {
                let text = self.collect_all_text(element);
                if !text.trim().is_empty() {
                    let mut rich_text = egui::RichText::new(text.trim()).size(font_size).italics();
                    if let Some(color) = parent_color {
                        rich_text = rich_text.color(color);
                    }
                    ui.label(rich_text);
                }
            }
            "code" => {
                let text = self.collect_all_text(element);
                if !text.trim().is_empty() {
                    ui.code(text.trim());
                }
            }
            _ => {
                if !element.text.trim().is_empty() {
                    let mut rich_text = egui::RichText::new(element.text.trim()).size(font_size);
                    if let Some(color) = parent_color {
                        rich_text = rich_text.color(color);
                    }
                    ui.label(rich_text);
                }
                for child in &element.children {
                    self.render_inline_structured_element(ui, child, font_size, parent_color);
                }
            }
        }
    }

    /// Collect all text from element and children recursively
    fn collect_all_text(&self, element: &DomElement) -> String {
        let mut text = element.text.clone();
        for child in &element.children {
            text.push_str(&self.collect_all_text(child));
        }
        text
    }

    /// Parse CSS pixel value (e.g., "16px" -> 16.0)
    fn parse_px(&self, value: &str) -> Option<f32> {
        value.trim_end_matches("px").parse().ok()
    }

    /// Parse CSS color (basic RGB parser)
    fn parse_color(&self, color: &str) -> Option<egui::Color32> {
        if color.starts_with("rgb(") && color.ends_with(')') {
            let rgb_str = &color[4..color.len()-1];
            let parts: Vec<&str> = rgb_str.split(',').collect();
            if parts.len() == 3 {
                let r: u8 = parts[0].trim().parse().ok()?;
                let g: u8 = parts[1].trim().parse().ok()?;
                let b: u8 = parts[2].trim().parse().ok()?;
                return Some(egui::Color32::from_rgb(r, g, b));
            }
        }
        None
    }

    /// Parse inline style attribute for all CSS properties
    fn parse_inline_style(&self, style: &str) -> CssStyle {
        let mut css_style = CssStyle::default();

        for rule in style.split(';') {
            let parts: Vec<&str> = rule.split(':').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                match parts[0] {
                    "color" => {
                        css_style.text_color = self.parse_css_color(parts[1]);
                    }
                    "background-color" | "background" => {
                        css_style.bg_color = self.parse_css_color(parts[1]);
                    }
                    "font-size" => {
                        let size = FontSize::parse_css(parts[1], self.base_font_size);
                        css_style.font_size = Some(size);
                    }
                    "font-weight" => {
                        let weight = FontWeight::from_css(parts[1]);
                        css_style.font_weight = Some(weight.is_bold());
                    }
                    "font-family" => {
                        let family = FontDescriptor::parse_family(parts[1]);
                        css_style.font_family = Some(family);
                    }
                    "text-align" => {
                        css_style.text_align = match parts[1].trim() {
                            "left" => Some(TextAlign::Left),
                            "center" => Some(TextAlign::Center),
                            "right" => Some(TextAlign::Right),
                            "justify" => Some(TextAlign::Justify),
                            _ => None,
                        };
                    }
                    "padding" => {
                        css_style.padding = self.parse_css_spacing(parts[1]);
                    }
                    "padding-top" => {
                        if let Some(val) = self.parse_css_size(parts[1]) {
                            css_style.padding.top = val;
                        }
                    }
                    "padding-bottom" => {
                        if let Some(val) = self.parse_css_size(parts[1]) {
                            css_style.padding.bottom = val;
                        }
                    }
                    "padding-left" => {
                        if let Some(val) = self.parse_css_size(parts[1]) {
                            css_style.padding.left = val;
                        }
                    }
                    "padding-right" => {
                        if let Some(val) = self.parse_css_size(parts[1]) {
                            css_style.padding.right = val;
                        }
                    }
                    "margin" => {
                        css_style.margin = self.parse_css_spacing(parts[1]);
                    }
                    "margin-top" => {
                        if let Some(val) = self.parse_css_size(parts[1]) {
                            css_style.margin.top = val;
                        }
                    }
                    "margin-bottom" => {
                        if let Some(val) = self.parse_css_size(parts[1]) {
                            css_style.margin.bottom = val;
                        }
                    }
                    "margin-left" => {
                        if let Some(val) = self.parse_css_size(parts[1]) {
                            css_style.margin.left = val;
                        }
                    }
                    "margin-right" => {
                        if let Some(val) = self.parse_css_size(parts[1]) {
                            css_style.margin.right = val;
                        }
                    }
                    "border" => {
                        // Parse shorthand border: "1px solid black"
                        let border_parts: Vec<&str> = parts[1].split_whitespace().collect();
                        if border_parts.len() >= 1 {
                            css_style.border_width = self.parse_css_size(border_parts[0]);
                        }
                        if border_parts.len() >= 3 {
                            css_style.border_color = self.parse_css_color(border_parts[2]);
                        }
                    }
                    "border-width" => {
                        css_style.border_width = self.parse_css_size(parts[1]);
                    }
                    "border-color" => {
                        css_style.border_color = self.parse_css_color(parts[1]);
                    }
                    "border-radius" => {
                        css_style.border_radius = self.parse_css_size(parts[1]);
                    }
                    "display" => {
                        css_style.display = match parts[1].trim() {
                            "block" => Some(DisplayType::Block),
                            "inline" => Some(DisplayType::Inline),
                            "flex" => Some(DisplayType::Flex),
                            "none" => Some(DisplayType::None),
                            _ => None,
                        };
                    }
                    "width" => {
                        css_style.width = self.parse_css_size(parts[1]);
                    }
                    "height" => {
                        css_style.height = self.parse_css_size(parts[1]);
                    }
                    _ => {}
                }
            }
        }

        css_style
    }

    /// Parse CSS size values (px, pt, em, rem, %)
    fn parse_css_size(&self, size: &str) -> Option<f32> {
        let size = size.trim();

        if size.ends_with("px") {
            size.trim_end_matches("px").parse::<f32>().ok()
        } else if size.ends_with("pt") {
            // Convert pt to px (1pt = 1.333px)
            size.trim_end_matches("pt").parse::<f32>().ok().map(|v| v * 1.333)
        } else if size.ends_with("em") || size.ends_with("rem") {
            // Convert em/rem to px (assume 14px base)
            size.trim_end_matches("em").trim_end_matches("rem").parse::<f32>().ok().map(|v| v * 14.0)
        } else if size.ends_with("%") {
            // For percentage, just strip the % and parse
            size.trim_end_matches("%").parse::<f32>().ok()
        } else {
            // Try parsing as raw number (assume px)
            size.parse::<f32>().ok()
        }
    }

    /// Parse CSS spacing values (can be 1-4 values: "10px" or "10px 20px" or "10px 20px 30px 40px")
    fn parse_css_spacing(&self, spacing: &str) -> CssSpacing {
        let parts: Vec<&str> = spacing.split_whitespace().collect();
        let mut result = CssSpacing::default();

        match parts.len() {
            1 => {
                // All sides same
                if let Some(val) = self.parse_css_size(parts[0]) {
                    result.top = val;
                    result.bottom = val;
                    result.left = val;
                    result.right = val;
                }
            }
            2 => {
                // top/bottom, left/right
                if let Some(tb) = self.parse_css_size(parts[0]) {
                    result.top = tb;
                    result.bottom = tb;
                }
                if let Some(lr) = self.parse_css_size(parts[1]) {
                    result.left = lr;
                    result.right = lr;
                }
            }
            3 => {
                // top, left/right, bottom
                if let Some(t) = self.parse_css_size(parts[0]) {
                    result.top = t;
                }
                if let Some(lr) = self.parse_css_size(parts[1]) {
                    result.left = lr;
                    result.right = lr;
                }
                if let Some(b) = self.parse_css_size(parts[2]) {
                    result.bottom = b;
                }
            }
            4 => {
                // top, right, bottom, left
                if let Some(t) = self.parse_css_size(parts[0]) {
                    result.top = t;
                }
                if let Some(r) = self.parse_css_size(parts[1]) {
                    result.right = r;
                }
                if let Some(b) = self.parse_css_size(parts[2]) {
                    result.bottom = b;
                }
                if let Some(l) = self.parse_css_size(parts[3]) {
                    result.left = l;
                }
            }
            _ => {}
        }

        result
    }

    /// Parse CSS color values (hex, rgb, named colors)
    fn parse_css_color(&self, color: &str) -> Option<egui::Color32> {
        let color = color.trim();

        // RGB/RGBA
        if color.starts_with("rgb") {
            return self.parse_color(color);
        }

        // Hex colors
        if color.starts_with('#') {
            let hex = &color[1..];
            if hex.len() == 6 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex[0..2], 16),
                    u8::from_str_radix(&hex[2..4], 16),
                    u8::from_str_radix(&hex[4..6], 16),
                ) {
                    return Some(egui::Color32::from_rgb(r, g, b));
                }
            } else if hex.len() == 3 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex[0..1].repeat(2), 16),
                    u8::from_str_radix(&hex[1..2].repeat(2), 16),
                    u8::from_str_radix(&hex[2..3].repeat(2), 16),
                ) {
                    return Some(egui::Color32::from_rgb(r, g, b));
                }
            }
        }

        // Named colors
        match color.to_lowercase().as_str() {
            "red" => Some(egui::Color32::from_rgb(255, 0, 0)),
            "green" => Some(egui::Color32::from_rgb(0, 128, 0)),
            "blue" => Some(egui::Color32::from_rgb(0, 0, 255)),
            "black" => Some(egui::Color32::BLACK),
            "white" => Some(egui::Color32::WHITE),
            "gray" | "grey" => Some(egui::Color32::GRAY),
            "yellow" => Some(egui::Color32::from_rgb(255, 255, 0)),
            "orange" => Some(egui::Color32::from_rgb(255, 165, 0)),
            "purple" => Some(egui::Color32::from_rgb(128, 0, 128)),
            _ => None,
        }
    }

    /// Render inline elements (like <strong>, <em>, etc.)
    fn render_inline_element(&self, ui: &mut Ui, element: scraper::ElementRef) {
        let tag_name = element.value().name();
        let text = self.collect_visible_text(element);

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