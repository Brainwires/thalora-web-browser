//! DOM element rendering functions

use egui::{Ui, RichText};
use scraper::node::Node;
use super::types::*;
use crate::gui::{FontDescriptor, FontSize};

impl super::BrowserUI {
    /// Collect text from element, excluding style tags only (scripts should be executed, not displayed)
    pub(super) fn collect_visible_text(&self, element: scraper::ElementRef) -> String {
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
    pub(super) fn render_dom_element(&mut self, ui: &mut Ui, element: scraper::ElementRef) {
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
                let href = element.value().attr("href").unwrap_or("#").to_string();
                if !text.trim().is_empty() {
                    let link = egui::RichText::new(text.trim())
                        .size(14.0)
                        .color(egui::Color32::from_rgb(0, 102, 204))
                        .underline();
                    if ui.add(egui::Label::new(link).sense(egui::Sense::click())).clicked() {
                        tracing::info!("Link clicked: {}", href);
                        // Set pending navigation to the link href
                        if !href.is_empty() && href != "#" {
                            self.pending_navigation = Some(href);
                        }
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
                let name = element.value().attr("name").unwrap_or("");

                // Collect options from the select element
                let mut options: Vec<(String, String)> = Vec::new();
                let mut selected_value = String::new();

                for child in element.children() {
                    if let Some(option_elem) = scraper::ElementRef::wrap(child) {
                        if option_elem.value().name() == "option" {
                            let value = option_elem.value().attr("value")
                                .unwrap_or("")
                                .to_string();
                            let text = self.collect_visible_text(option_elem);
                            let is_selected = option_elem.value().attr("selected").is_some();

                            if is_selected || (selected_value.is_empty() && !value.is_empty()) {
                                selected_value = value.clone();
                            }

                            options.push((value, text.trim().to_string()));
                        }
                    }
                }

                // Show dropdown using egui's ComboBox
                if !options.is_empty() {
                    ui.horizontal(|ui| {
                        if !name.is_empty() {
                            ui.label(format!("{}:", name));
                        }

                        let display_text = options.iter()
                            .find(|(v, _)| v == &selected_value)
                            .map(|(_, t)| t.as_str())
                            .unwrap_or("Select...");

                        egui::ComboBox::from_id_salt(format!("select_{}", name))
                            .selected_text(display_text)
                            .show_ui(ui, |ui| {
                                for (value, text) in &options {
                                    ui.selectable_value(&mut selected_value, value.clone(), text);
                                }
                            });
                    });
                } else {
                    ui.label("Dropdown (no options)");
                }
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
                let _width = element.value().attr("width");
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
    pub(super) fn render_structured_element(&mut self, ui: &mut Ui, element: &DomElement, depth: usize) {
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
    pub(super) fn render_inline_structured_element(&mut self, ui: &mut Ui, element: &DomElement, font_size: f32, parent_color: Option<egui::Color32>) {
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
    pub(super) fn collect_all_text(&self, element: &DomElement) -> String {
        let mut text = element.text.clone();
        for child in &element.children {
            text.push_str(&self.collect_all_text(child));
        }
        text
    }

    /// Render inline elements (like <strong>, <em>, etc.)
    pub(super) fn render_inline_element(&mut self, ui: &mut Ui, element: scraper::ElementRef) {
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
                let href = element.value().attr("href").unwrap_or("#").to_string();
                if ui.link(text.trim()).clicked() {
                    tracing::info!("Link clicked: {}", href);
                    // Set pending navigation to the link href
                    if !href.is_empty() && href != "#" {
                        self.pending_navigation = Some(href);
                    }
                }
            }
            _ => {
                ui.label(text.trim());
            }
        }
    }
}
