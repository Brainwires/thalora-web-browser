// Tests for BrowserAction enum and pending action system

use thalora::gui::{BrowserUI, BrowserAction};

/// Test BrowserAction enum variants exist
#[test]
fn test_browser_action_variants() {
    // Test all variants can be created
    let _action1 = BrowserAction::GoBack;
    let _action2 = BrowserAction::GoForward;
    let _action3 = BrowserAction::Reload;
    let _action4 = BrowserAction::StopLoading;
    let _action5 = BrowserAction::NewTab;
    let _action6 = BrowserAction::CloseTab(1);
    let _action7 = BrowserAction::SwitchTab(2);
    let _action8 = BrowserAction::ShowMenu;
    let _action9 = BrowserAction::FocusAddressBar;
    let _action10 = BrowserAction::ExecuteJavaScript("console.log('test')".to_string());
}

/// Test BrowserAction Debug trait
#[test]
fn test_browser_action_debug() {
    let action = BrowserAction::GoBack;
    let debug_str = format!("{:?}", action);
    assert!(debug_str.contains("GoBack"));

    let action = BrowserAction::CloseTab(42);
    let debug_str = format!("{:?}", action);
    assert!(debug_str.contains("CloseTab"));
    assert!(debug_str.contains("42"));
}

/// Test BrowserAction Clone trait
#[test]
fn test_browser_action_clone() {
    let action = BrowserAction::ExecuteJavaScript("test code".to_string());
    let cloned = action.clone();

    match cloned {
        BrowserAction::ExecuteJavaScript(code) => {
            assert_eq!(code, "test code");
        }
        _ => panic!("Clone failed to preserve variant"),
    }
}

/// Test pending action workflow
#[test]
fn test_pending_action_workflow() {
    let mut ui = BrowserUI::new(false);

    // Initially no pending action
    assert!(ui.take_pending_action().is_none());

    // Set a pending action
    ui.set_pending_action(BrowserAction::NewTab);

    // Take should return the action
    let action = ui.take_pending_action();
    assert!(action.is_some());
    match action.unwrap() {
        BrowserAction::NewTab => {} // Expected
        _ => panic!("Wrong action returned"),
    }

    // After take, should be None again
    assert!(ui.take_pending_action().is_none());
}

/// Test multiple pending actions (only last one is kept)
#[test]
fn test_pending_action_overwrites() {
    let mut ui = BrowserUI::new(false);

    // Set multiple actions
    ui.set_pending_action(BrowserAction::GoBack);
    ui.set_pending_action(BrowserAction::GoForward);
    ui.set_pending_action(BrowserAction::Reload);

    // Should only get the last one
    let action = ui.take_pending_action();
    assert!(action.is_some());
    match action.unwrap() {
        BrowserAction::Reload => {} // Expected - last action wins
        _ => panic!("Should have gotten Reload action"),
    }
}

/// Test CloseTab with tab ID 0 (current tab)
#[test]
fn test_close_current_tab_action() {
    let action = BrowserAction::CloseTab(0);
    match action {
        BrowserAction::CloseTab(id) => {
            assert_eq!(id, 0, "Tab ID 0 means current tab");
        }
        _ => panic!("Wrong variant"),
    }
}

/// Test SwitchTab with various tab IDs
#[test]
fn test_switch_tab_action() {
    for id in [1, 5, 100, u32::MAX] {
        let action = BrowserAction::SwitchTab(id);
        match action {
            BrowserAction::SwitchTab(tab_id) => {
                assert_eq!(tab_id, id);
            }
            _ => panic!("Wrong variant"),
        }
    }
}

/// Test ExecuteJavaScript with various code strings
#[test]
fn test_execute_javascript_action() {
    let test_cases = [
        "console.log('hello')",
        "document.title",
        "window.location.href",
        "",  // Empty code
        "alert('test'); return 42;",
    ];

    for code in test_cases {
        let action = BrowserAction::ExecuteJavaScript(code.to_string());
        match action {
            BrowserAction::ExecuteJavaScript(js_code) => {
                assert_eq!(js_code, code);
            }
            _ => panic!("Wrong variant"),
        }
    }
}

/// Test pending navigation separate from pending action
#[test]
fn test_pending_navigation_and_action_independent() {
    let mut ui = BrowserUI::new(false);

    // Both should be initially None
    assert!(ui.take_pending_navigation().is_none());
    assert!(ui.take_pending_action().is_none());

    // Set pending action
    ui.set_pending_action(BrowserAction::Reload);

    // Pending navigation should still be None
    assert!(ui.take_pending_navigation().is_none());

    // Pending action should have value
    assert!(ui.take_pending_action().is_some());
}
