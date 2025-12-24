// Tests for InputHandler functionality

use thalora::gui::InputHandler;
use thalora::gui::input_handler::BrowserAction;

/// Test creating a new InputHandler
#[test]
fn test_create_input_handler() {
    let handler = InputHandler::new();
    // Should not panic
    assert!(!handler.is_mouse_pressed());
    assert!(!handler.is_ctrl_pressed());
    assert!(!handler.is_shift_pressed());
    assert!(!handler.is_alt_pressed());
}

/// Test InputHandler default trait
#[test]
fn test_input_handler_default() {
    let handler = InputHandler::default();
    assert_eq!(handler.mouse_position(), (0.0, 0.0));
}

/// Test mouse position getter
#[test]
fn test_mouse_position() {
    let handler = InputHandler::new();
    let (x, y) = handler.mouse_position();
    assert_eq!(x, 0.0);
    assert_eq!(y, 0.0);
}

/// Test modifier key states
#[test]
fn test_modifier_keys_initial_state() {
    let handler = InputHandler::new();

    assert!(!handler.is_ctrl_pressed());
    assert!(!handler.is_shift_pressed());
    assert!(!handler.is_alt_pressed());
}

/// Test scroll delta
#[test]
fn test_scroll_delta() {
    let handler = InputHandler::new();
    let (dx, dy) = handler.scroll_delta();
    assert_eq!(dx, 0.0);
    assert_eq!(dy, 0.0);
}

/// Test take_scroll_delta resets values
#[test]
fn test_take_scroll_delta_resets() {
    let mut handler = InputHandler::new();

    // Initial value
    let (dx1, dy1) = handler.take_scroll_delta();
    assert_eq!(dx1, 0.0);
    assert_eq!(dy1, 0.0);

    // After take, still zero
    let (dx2, dy2) = handler.scroll_delta();
    assert_eq!(dx2, 0.0);
    assert_eq!(dy2, 0.0);
}

/// Test right mouse button state
#[test]
fn test_right_mouse_button() {
    let handler = InputHandler::new();
    assert!(!handler.is_right_mouse_pressed());
}

/// Test middle mouse button state
#[test]
fn test_middle_mouse_button() {
    let handler = InputHandler::new();
    assert!(!handler.is_middle_mouse_pressed());
}

/// Test BrowserAction enum from input_handler module
#[test]
fn test_input_browser_action_variants() {
    // Test all variants exist and can be created
    let actions = vec![
        BrowserAction::Quit,
        BrowserAction::NewTab,
        BrowserAction::CloseTab,
        BrowserAction::Navigate("https://example.com".to_string()),
        BrowserAction::Reload,
        BrowserAction::GoBack,
        BrowserAction::GoForward,
        BrowserAction::ZoomIn,
        BrowserAction::ZoomOut,
        BrowserAction::ResetZoom,
        BrowserAction::ToggleDevTools,
        BrowserAction::FocusAddressBar,
        BrowserAction::SwitchTab(1),
        BrowserAction::ShowContextMenu(100.0, 200.0),
        BrowserAction::OpenLinkInNewTab("https://test.com".to_string()),
        BrowserAction::Scroll(10.0, -20.0),
        BrowserAction::None,
    ];

    assert_eq!(actions.len(), 17);
}

/// Test BrowserAction Debug trait
#[test]
fn test_input_browser_action_debug() {
    let action = BrowserAction::Navigate("https://example.com".to_string());
    let debug_str = format!("{:?}", action);
    assert!(debug_str.contains("Navigate"));
    assert!(debug_str.contains("example.com"));
}

/// Test BrowserAction Clone trait
#[test]
fn test_input_browser_action_clone() {
    let action = BrowserAction::ShowContextMenu(50.5, 75.25);
    let cloned = action.clone();

    match cloned {
        BrowserAction::ShowContextMenu(x, y) => {
            assert_eq!(x, 50.5);
            assert_eq!(y, 75.25);
        }
        _ => panic!("Clone failed"),
    }
}

/// Test Scroll action with various deltas
#[test]
fn test_scroll_action_values() {
    let test_cases = [
        (0.0, 0.0),
        (10.0, -10.0),
        (-40.0, 40.0),
        (100.5, 200.75),
        (-0.5, 0.5),
    ];

    for (dx, dy) in test_cases {
        let action = BrowserAction::Scroll(dx, dy);
        match action {
            BrowserAction::Scroll(scroll_x, scroll_y) => {
                assert_eq!(scroll_x, dx);
                assert_eq!(scroll_y, dy);
            }
            _ => panic!("Wrong variant"),
        }
    }
}

/// Test ShowContextMenu with various coordinates
#[test]
fn test_context_menu_action_coordinates() {
    let test_cases = [
        (0.0, 0.0),
        (100.0, 200.0),
        (1920.0, 1080.0),
        (0.5, 0.5),
    ];

    for (x, y) in test_cases {
        let action = BrowserAction::ShowContextMenu(x, y);
        match action {
            BrowserAction::ShowContextMenu(menu_x, menu_y) => {
                assert_eq!(menu_x, x);
                assert_eq!(menu_y, y);
            }
            _ => panic!("Wrong variant"),
        }
    }
}

/// Test SwitchTab with various tab indices
#[test]
fn test_switch_tab_indices() {
    for idx in [0, 1, 5, 10, 100] {
        let action = BrowserAction::SwitchTab(idx);
        match action {
            BrowserAction::SwitchTab(tab_idx) => {
                assert_eq!(tab_idx, idx);
            }
            _ => panic!("Wrong variant"),
        }
    }
}

/// Test multiple InputHandler instances
#[test]
fn test_multiple_input_handlers() {
    let handler1 = InputHandler::new();
    let handler2 = InputHandler::new();

    // Both should be independent
    assert_eq!(handler1.mouse_position(), (0.0, 0.0));
    assert_eq!(handler2.mouse_position(), (0.0, 0.0));
}
