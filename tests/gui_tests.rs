// GUI Component Tests
// Tests for browser UI, input handling, and browser actions

#![cfg(feature = "gui")]

// BrowserAction enum tests
mod browser_action_tests {
    include!("gui/browser_action_test.rs");
}

// InputHandler tests
mod input_handler_tests {
    include!("gui/input_handler_test.rs");
}
