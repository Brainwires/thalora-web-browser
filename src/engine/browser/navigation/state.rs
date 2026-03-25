use anyhow::{Result, anyhow};

impl super::super::HeadlessWebBrowser {
    /// Go back in navigation history
    /// Returns the URL navigated to, or None if at the beginning of history
    pub async fn go_back(&mut self) -> Result<Option<String>> {
        if !self.can_go_back() {
            eprintln!("🔍 DEBUG: go_back - cannot go back, at beginning of history");
            return Ok(None);
        }

        // Decrement history index
        self.history.current_index -= 1;

        // Get the URL from history
        let entry = &self.history.entries[self.history.current_index];
        let url = entry.url.clone();

        eprintln!(
            "🔍 DEBUG: go_back - navigating to history entry {}: {}",
            self.history.current_index, url
        );

        // Navigate without adding to history
        self.navigate_internal(&url).await?;

        Ok(Some(url))
    }

    /// Go forward in navigation history
    /// Returns the URL navigated to, or None if at the end of history
    pub async fn go_forward(&mut self) -> Result<Option<String>> {
        if !self.can_go_forward() {
            eprintln!("🔍 DEBUG: go_forward - cannot go forward, at end of history");
            return Ok(None);
        }

        // Increment history index
        self.history.current_index += 1;

        // Get the URL from history
        let entry = &self.history.entries[self.history.current_index];
        let url = entry.url.clone();

        eprintln!(
            "🔍 DEBUG: go_forward - navigating to history entry {}: {}",
            self.history.current_index, url
        );

        // Navigate without adding to history
        self.navigate_internal(&url).await?;

        Ok(Some(url))
    }

    /// Get current page title
    pub fn get_current_title(&self) -> Option<String> {
        self.extract_title(&self.current_content)
    }

    /// Get the current position in history
    pub fn get_history_position(&self) -> (usize, usize) {
        (self.history.current_index, self.history.entries.len())
    }

    /// Get history entries as a list of (url, title) tuples
    pub fn get_history_entries(&self) -> Vec<(String, String)> {
        self.history
            .entries
            .iter()
            .map(|e| (e.url.clone(), e.title.clone()))
            .collect()
    }
}
