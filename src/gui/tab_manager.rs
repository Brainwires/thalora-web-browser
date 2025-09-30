//! Tab management for the graphical browser
//! 
//! This module handles multiple browser tabs, each with its own HeadlessWebBrowser instance
//! and navigation state.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::{Result, Context};
use crate::engine::{HeadlessWebBrowser, EngineConfig};

/// Unique identifier for browser tabs
pub type TabId = u32;

/// Individual browser tab
pub struct Tab {
    id: TabId,
    title: String,
    url: String,
    browser: Arc<Mutex<HeadlessWebBrowser>>,
    is_loading: bool,
    can_go_back: bool,
    can_go_forward: bool,
}

impl Tab {
    /// Create a new tab
    fn new(id: TabId, initial_url: String, _engine_config: EngineConfig) -> Result<Self> {
        let browser = HeadlessWebBrowser::new();
        
        Ok(Self {
            id,
            title: "New Tab".to_string(),
            url: initial_url,
            browser,
            is_loading: false,
            can_go_back: false,
            can_go_forward: false,
        })
    }

    /// Get tab ID
    pub fn id(&self) -> TabId {
        self.id
    }

    /// Get tab title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get tab URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Check if tab is loading
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    /// Check if tab can go back
    pub fn can_go_back(&self) -> bool {
        self.can_go_back
    }

    /// Check if tab can go forward
    pub fn can_go_forward(&self) -> bool {
        self.can_go_forward
    }

    /// Navigate to a URL
    pub async fn navigate_to(&mut self, url: &str) -> Result<()> {
        tracing::info!("Tab {}: Navigating to {}", self.id, url);
        
        self.is_loading = true;
        self.url = url.to_string();

        // Navigate using the browser engine
        let mut browser = self.browser.lock().unwrap();
        match browser.navigate_to(url).await {
            Ok(_) => {
                // Get content and update title separately to avoid borrowing conflicts
                let content = browser.get_current_content();
                drop(browser); // Release the mutex before calling mutable self method
                self.update_title_from_content_string(&content);
                self.is_loading = false;
                self.can_go_back = true; // Simplified - in real implementation track history
                tracing::info!("Tab {}: Navigation completed", self.id);
                Ok(())
            }
            Err(e) => {
                self.is_loading = false;
                self.title = "Failed to load".to_string();
                Err(e)
            }
        }
    }

    /// Execute JavaScript in this tab
    pub async fn execute_javascript(&mut self, code: &str) -> Result<String> {
        let mut browser = self.browser.lock().unwrap();
        browser.execute_javascript(code).await
            .context("Failed to execute JavaScript in tab")
    }

    /// Get page content
    pub async fn get_content(&self) -> Result<String> {
        let browser = self.browser.lock().unwrap();
        Ok(browser.get_current_content())
    }

    /// Reload the current page
    pub async fn reload(&mut self) -> Result<()> {
        let current_url = self.url.clone();
        self.navigate_to(&current_url).await
    }

    /// Go back in history (simplified implementation)
    pub async fn go_back(&mut self) -> Result<()> {
        if self.can_go_back {
            // TODO: Implement proper history management
            tracing::debug!("Tab {}: Going back (not implemented)", self.id);
        }
        Ok(())
    }

    /// Go forward in history (simplified implementation)
    pub async fn go_forward(&mut self) -> Result<()> {
        if self.can_go_forward {
            // TODO: Implement proper history management
            tracing::debug!("Tab {}: Going forward (not implemented)", self.id);
        }
        Ok(())
    }

    /// Update title from page content
    fn update_title_from_content(&mut self, browser: &HeadlessWebBrowser) {
        let content = browser.get_current_content();
        self.update_title_from_content_string(&content);
    }

    /// Update title from page content string
    fn update_title_from_content_string(&mut self, content: &str) {
        // Try to extract title from page content
        if let Some(title_start) = content.find("<title>") {
            if let Some(title_end) = content[title_start + 7..].find("</title>") {
                let title = &content[title_start + 7..title_start + 7 + title_end];
                if !title.trim().is_empty() {
                    self.title = title.trim().to_string();
                    return;
                }
            }
        }

        // Fallback to URL-based title
        if let Ok(parsed_url) = url::Url::parse(&self.url) {
            if let Some(host) = parsed_url.host_str() {
                self.title = host.to_string();
            } else {
                self.title = self.url.clone();
            }
        } else {
            self.title = self.url.clone();
        }
    }
}

/// Tab manager for handling multiple browser tabs
pub struct TabManager {
    tabs: HashMap<TabId, Tab>,
    current_tab: Option<TabId>,
    next_tab_id: TabId,
    engine_config: EngineConfig,
}

impl TabManager {
    /// Create a new tab manager
    pub fn new() -> Self {
        Self {
            tabs: HashMap::new(),
            current_tab: None,
            next_tab_id: 1,
            engine_config: EngineConfig::new(false).unwrap_or_else(|_| {
                // Fallback engine config if creation fails
                EngineConfig {
                    engine_type: crate::engine::EngineType::Boa,
                }
            }),
        }
    }

    /// Create a new tab
    pub fn create_tab(&mut self, initial_url: String, engine_config: EngineConfig) -> Result<TabId> {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        tracing::info!("Creating new tab {} with URL: {}", tab_id, initial_url);

        let tab = Tab::new(tab_id, initial_url, engine_config.clone())?;
        self.tabs.insert(tab_id, tab);
        self.current_tab = Some(tab_id);
        self.engine_config = engine_config;

        Ok(tab_id)
    }

    /// Close a tab
    pub fn close_tab(&mut self, tab_id: TabId) -> Result<()> {
        if let Some(_tab) = self.tabs.remove(&tab_id) {
            tracing::info!("Closed tab {}", tab_id);

            // If this was the current tab, switch to another
            if self.current_tab == Some(tab_id) {
                self.current_tab = self.tabs.keys().next().copied();
            }
        }

        Ok(())
    }

    /// Switch to a specific tab
    pub fn switch_to_tab(&mut self, tab_id: TabId) -> Result<()> {
        if self.tabs.contains_key(&tab_id) {
            tracing::debug!("Switching to tab {}", tab_id);
            self.current_tab = Some(tab_id);
            Ok(())
        } else {
            anyhow::bail!("Tab {} does not exist", tab_id)
        }
    }

    /// Get current tab ID
    pub fn current_tab_id(&self) -> Option<TabId> {
        self.current_tab
    }

    /// Get a reference to a specific tab
    pub fn get_tab(&self, tab_id: TabId) -> Option<&Tab> {
        self.tabs.get(&tab_id)
    }

    /// Get a mutable reference to a specific tab
    pub fn get_tab_mut(&mut self, tab_id: TabId) -> Option<&mut Tab> {
        self.tabs.get_mut(&tab_id)
    }

    /// Get all tabs
    pub fn tabs(&self) -> impl Iterator<Item = &Tab> {
        self.tabs.values()
    }

    /// Navigate the specified tab to a URL
    pub async fn navigate_tab(&mut self, tab_id: TabId, url: &str) -> Result<()> {
        if let Some(tab) = self.tabs.get_mut(&tab_id) {
            tab.navigate_to(url).await
        } else {
            anyhow::bail!("Tab {} does not exist", tab_id)
        }
    }

    /// Get content from the specified tab
    pub async fn get_tab_content(&self, tab_id: TabId) -> Result<String> {
        if let Some(tab) = self.tabs.get(&tab_id) {
            tab.get_content().await
        } else {
            anyhow::bail!("Tab {} does not exist", tab_id)
        }
    }

    /// Execute JavaScript in the specified tab
    pub async fn execute_javascript_in_tab(&mut self, tab_id: TabId, code: &str) -> Result<String> {
        if let Some(tab) = self.tabs.get_mut(&tab_id) {
            tab.execute_javascript(code).await
        } else {
            anyhow::bail!("Tab {} does not exist", tab_id)
        }
    }

    /// Reload the specified tab
    pub async fn reload_tab(&mut self, tab_id: TabId) -> Result<()> {
        if let Some(tab) = self.tabs.get_mut(&tab_id) {
            tab.reload().await
        } else {
            anyhow::bail!("Tab {} does not exist", tab_id)
        }
    }

    /// Get the number of open tabs
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Check if there are any tabs open
    pub fn has_tabs(&self) -> bool {
        !self.tabs.is_empty()
    }

    /// Get tab IDs in order
    pub fn tab_ids(&self) -> Vec<TabId> {
        let mut ids: Vec<TabId> = self.tabs.keys().copied().collect();
        ids.sort();
        ids
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}