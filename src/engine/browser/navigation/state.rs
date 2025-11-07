use anyhow::{Result, anyhow};

impl super::super::HeadlessWebBrowser {
    /// Go back in navigation history
    pub async fn go_back(&mut self) -> Result<Option<String>> {
        // Simplified implementation - return None since we don't maintain history
        Ok(None)
    }

    /// Go forward in navigation history
    pub async fn go_forward(&mut self) -> Result<Option<String>> {
        // Simplified implementation - return None since we don't maintain history
        Ok(None)
    }

    /// Get current page title
    pub fn get_current_title(&self) -> Option<String> {
        self.extract_title(&self.current_content)
    }
}
