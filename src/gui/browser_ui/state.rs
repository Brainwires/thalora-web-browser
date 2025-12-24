//! State management methods

impl super::BrowserUI {
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
    pub fn navigation_state(&self) -> &super::types::NavigationState {
        &self.navigation_state
    }

    /// Take pending navigation URL if one exists
    pub fn take_pending_navigation(&mut self) -> Option<String> {
        self.pending_navigation.take()
    }

    /// Set a pending browser action
    pub fn set_pending_action(&mut self, action: super::types::BrowserAction) {
        self.pending_action = Some(action);
    }

    /// Take pending browser action if one exists
    pub fn take_pending_action(&mut self) -> Option<super::types::BrowserAction> {
        self.pending_action.take()
    }
}
