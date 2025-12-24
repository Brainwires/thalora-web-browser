//! Type definitions for browser UI

use egui::Color32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DOM element structure from JavaScript query
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DomElement {
    pub tag: String,
    pub text: String,
    pub attrs: HashMap<String, String>,
    pub style: ElementStyle,
    pub children: Vec<DomElement>,
}

/// Computed style from getComputedStyle
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ElementStyle {
    pub color: String,
    #[serde(rename = "backgroundColor")]
    pub background_color: String,
    #[serde(rename = "fontSize")]
    pub font_size: String,
    #[serde(rename = "fontWeight")]
    pub font_weight: String,
    #[serde(rename = "fontFamily")]
    pub font_family: String,
    #[serde(rename = "textDecoration")]
    pub text_decoration: String,
    pub display: String,
    #[serde(rename = "marginTop")]
    pub margin_top: String,
    #[serde(rename = "marginBottom")]
    pub margin_bottom: String,
    #[serde(rename = "paddingTop")]
    pub padding_top: String,
    #[serde(rename = "paddingBottom")]
    pub padding_bottom: String,
}

/// CSS spacing (for padding/margin)
#[derive(Debug, Clone, Default)]
pub struct CssSpacing {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

/// Text alignment
#[derive(Debug, Clone)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

/// Display type
#[derive(Debug, Clone)]
pub enum DisplayType {
    Block,
    Inline,
    Flex,
    None,
}

/// Parsed CSS style properties
#[derive(Debug, Clone, Default)]
pub struct CssStyle {
    pub text_color: Option<Color32>,
    pub bg_color: Option<Color32>,
    pub font_size: Option<f32>,
    pub font_weight: Option<bool>, // true = bold
    pub font_family: Option<String>,
    pub text_align: Option<TextAlign>,
    pub padding: CssSpacing,
    pub margin: CssSpacing,
    pub border_width: Option<f32>,
    pub border_color: Option<Color32>,
    pub border_radius: Option<f32>,
    pub display: Option<DisplayType>,
    pub width: Option<f32>,
    pub height: Option<f32>,
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

/// Browser actions that can be triggered by UI interactions
#[derive(Debug, Clone)]
pub enum BrowserAction {
    /// Navigate back in history
    GoBack,
    /// Navigate forward in history
    GoForward,
    /// Reload the current page
    Reload,
    /// Stop loading the current page
    StopLoading,
    /// Create a new tab
    NewTab,
    /// Close a specific tab
    CloseTab(crate::gui::TabId),
    /// Switch to a specific tab
    SwitchTab(crate::gui::TabId),
    /// Show the browser menu
    ShowMenu,
    /// Focus the address bar
    FocusAddressBar,
    /// Execute JavaScript in the current tab
    ExecuteJavaScript(String),
}
