//! Keyboard Event Dispatcher
//!
//! Provides realistic keyboard event simulation for form automation.
//! Generates proper keydown, keypress, input, and keyup event sequences.

use boa_engine::{
    object::JsObject,
    Context, JsResult,
};

use crate::dom::element::with_element_data;
use crate::events::event::EventData;
use crate::events::propagation::dispatch_event_with_propagation;
use crate::events::ui_events::{KeyboardEventData, UIEventData, InputEventData, key_location};

/// A keyboard action in a typing sequence
#[derive(Debug, Clone)]
pub struct KeyboardAction {
    /// Event type: "keydown", "keyup", "input"
    pub event_type: String,
    /// The key value (e.g., "a", "Enter", "Backspace")
    pub key: String,
    /// The physical key code (e.g., "KeyA", "Enter", "Backspace")
    pub code: String,
    /// Legacy key code
    pub key_code: u32,
    /// Whether Shift is pressed
    pub shift_key: bool,
    /// Whether Ctrl is pressed
    pub ctrl_key: bool,
    /// Whether Alt is pressed
    pub alt_key: bool,
    /// Whether Meta (Cmd/Win) is pressed
    pub meta_key: bool,
}

impl KeyboardAction {
    /// Create a new keyboard action for a regular character key
    pub fn from_char(c: char, shift: bool) -> Self {
        let key = c.to_string();
        let code = char_to_code(c);
        let key_code = char_to_keycode(c);

        Self {
            event_type: "keydown".to_string(),
            key,
            code,
            key_code,
            shift_key: shift,
            ctrl_key: false,
            alt_key: false,
            meta_key: false,
        }
    }

    /// Create a keydown event
    pub fn keydown(key: &str, code: &str, key_code: u32) -> Self {
        Self {
            event_type: "keydown".to_string(),
            key: key.to_string(),
            code: code.to_string(),
            key_code,
            shift_key: false,
            ctrl_key: false,
            alt_key: false,
            meta_key: false,
        }
    }

    /// Create a keyup event
    pub fn keyup(key: &str, code: &str, key_code: u32) -> Self {
        Self {
            event_type: "keyup".to_string(),
            key: key.to_string(),
            code: code.to_string(),
            key_code,
            shift_key: false,
            ctrl_key: false,
            alt_key: false,
            meta_key: false,
        }
    }
}

/// A sequence of keyboard events for typing text
pub struct KeyboardSequence {
    events: Vec<KeyboardAction>,
}

impl KeyboardSequence {
    /// Create a keyboard sequence from a text string
    /// Generates keydown/input/keyup for each character
    pub fn from_text(text: &str) -> Self {
        let mut events = Vec::new();

        for c in text.chars() {
            let is_upper = c.is_uppercase();
            let key_char = if is_upper { c.to_lowercase().next().unwrap_or(c) } else { c };

            // keydown
            events.push(KeyboardAction::from_char(key_char, is_upper));

            // input event (for text input)
            events.push(KeyboardAction {
                event_type: "input".to_string(),
                key: c.to_string(),
                code: char_to_code(key_char),
                key_code: char_to_keycode(key_char),
                shift_key: is_upper,
                ctrl_key: false,
                alt_key: false,
                meta_key: false,
            });

            // keyup
            let mut keyup = KeyboardAction::from_char(key_char, is_upper);
            keyup.event_type = "keyup".to_string();
            events.push(keyup);
        }

        Self { events }
    }

    /// Dispatch the keyboard sequence to an element
    pub fn dispatch_to(&self, element: &JsObject, context: &mut Context) -> JsResult<()> {
        for action in &self.events {
            dispatch_keyboard_action(element, action, context)?;
        }
        Ok(())
    }
}

/// Dispatch a single keyboard action to an element
fn dispatch_keyboard_action(
    element: &JsObject,
    action: &KeyboardAction,
    context: &mut Context,
) -> JsResult<()> {
    match action.event_type.as_str() {
        "keydown" | "keyup" => {
            dispatch_keyboard_event(element, action, context)?;
        }
        "input" => {
            dispatch_input_event(element, action, context)?;
            // Also update element value if it's an input element
            update_input_value(element, &action.key)?;
        }
        _ => {}
    }
    Ok(())
}

/// Dispatch a keydown or keyup event
fn dispatch_keyboard_event(
    element: &JsObject,
    action: &KeyboardAction,
    context: &mut Context,
) -> JsResult<bool> {
    let is_keydown = action.event_type == "keydown";

    // Create KeyboardEvent
    let mut event_base = EventData::new_trusted(
        action.event_type.clone(),
        true,  // bubbles
        true,  // cancelable
    );
    event_base.set_target(Some(element.clone()));

    let mut ui_event = UIEventData::new(action.event_type.clone(), true, true);
    ui_event.event = event_base;

    let keyboard_event_data = KeyboardEventData {
        ui_event,
        key: action.key.clone(),
        code: action.code.clone(),
        location: key_location::DOM_KEY_LOCATION_STANDARD,
        ctrl_key: action.ctrl_key,
        shift_key: action.shift_key,
        alt_key: action.alt_key,
        meta_key: action.meta_key,
        repeat: false,
        is_composing: false,
        key_code: action.key_code,
        char_code: if is_keydown { 0 } else { 0 },
        which: action.key_code,
    };

    let keyboard_event_proto = context.intrinsics().constructors().keyboard_event().prototype();
    let keyboard_event = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        keyboard_event_proto,
        keyboard_event_data,
    );

    dispatch_event_with_propagation(&keyboard_event.upcast(), element, context)
}

/// Dispatch an input event
fn dispatch_input_event(
    element: &JsObject,
    action: &KeyboardAction,
    context: &mut Context,
) -> JsResult<bool> {
    // Create InputEvent
    let mut event_base = EventData::new_trusted("input".to_string(), true, false); // input events are not cancelable
    event_base.set_target(Some(element.clone()));

    let mut ui_event = UIEventData::new("input".to_string(), true, false);
    ui_event.event = event_base;

    let input_event_data = InputEventData {
        ui_event,
        data: Some(action.key.clone()),
        input_type: "insertText".to_string(),
        is_composing: false,
        data_transfer: None,
    };

    let input_event_proto = context.intrinsics().constructors().input_event().prototype();
    let input_event = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        input_event_proto,
        input_event_data,
    );

    dispatch_event_with_propagation(&input_event.upcast(), element, context)
}

/// Update the value property of an input element
fn update_input_value(element: &JsObject, text_to_add: &str) -> JsResult<()> {
    if let Ok(()) = with_element_data(element, |element_data| {
        let tag = element_data.get_tag_name().to_uppercase();
        if tag == "INPUT" || tag == "TEXTAREA" {
            // Append to current value
            let current_value = element_data.get_attribute("value").unwrap_or_default();
            let new_value = format!("{}{}", current_value, text_to_add);
            element_data.set_attribute("value".to_string(), new_value);
        }
    }, "not element") {}
    Ok(())
}

/// Map a character to its key code
fn char_to_keycode(c: char) -> u32 {
    match c {
        'a'..='z' => (c as u32) - ('a' as u32) + 65,  // A-Z
        'A'..='Z' => (c as u32) - ('A' as u32) + 65,  // A-Z (same as lowercase)
        '0'..='9' => (c as u32) - ('0' as u32) + 48,  // 0-9
        ' ' => 32,                                      // Space
        '\n' | '\r' => 13,                             // Enter
        '\t' => 9,                                      // Tab
        _ => 0,
    }
}

/// Map a character to its code (physical key location)
fn char_to_code(c: char) -> String {
    match c {
        'a'..='z' | 'A'..='Z' => format!("Key{}", c.to_uppercase()),
        '0'..='9' => format!("Digit{}", c),
        ' ' => "Space".to_string(),
        '\n' | '\r' => "Enter".to_string(),
        '\t' => "Tab".to_string(),
        '-' => "Minus".to_string(),
        '=' => "Equal".to_string(),
        '[' => "BracketLeft".to_string(),
        ']' => "BracketRight".to_string(),
        '\\' => "Backslash".to_string(),
        ';' => "Semicolon".to_string(),
        '\'' => "Quote".to_string(),
        '`' => "Backquote".to_string(),
        ',' => "Comma".to_string(),
        '.' => "Period".to_string(),
        '/' => "Slash".to_string(),
        _ => "Unidentified".to_string(),
    }
}

/// Type text into an element with realistic keyboard events
pub fn type_text(element: &JsObject, text: &str, context: &mut Context) -> JsResult<()> {
    // First, focus the element
    crate::browser::focus_manager::focus_element(element, context)?;

    // Then dispatch keyboard events
    let sequence = KeyboardSequence::from_text(text);
    sequence.dispatch_to(element, context)
}

/// Press a special key (Enter, Tab, Escape, etc.)
pub fn press_key(element: &JsObject, key: &str, context: &mut Context) -> JsResult<()> {
    let (code, key_code) = match key {
        "Enter" => ("Enter", 13u32),
        "Tab" => ("Tab", 9),
        "Escape" | "Esc" => ("Escape", 27),
        "Backspace" => ("Backspace", 8),
        "Delete" => ("Delete", 46),
        "ArrowUp" => ("ArrowUp", 38),
        "ArrowDown" => ("ArrowDown", 40),
        "ArrowLeft" => ("ArrowLeft", 37),
        "ArrowRight" => ("ArrowRight", 39),
        "Home" => ("Home", 36),
        "End" => ("End", 35),
        "PageUp" => ("PageUp", 33),
        "PageDown" => ("PageDown", 34),
        _ => (key, 0),
    };

    // keydown
    let keydown = KeyboardAction::keydown(key, code, key_code);
    dispatch_keyboard_event(element, &keydown, context)?;

    // keyup
    let keyup = KeyboardAction::keyup(key, code, key_code);
    dispatch_keyboard_event(element, &keyup, context)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_to_keycode() {
        assert_eq!(char_to_keycode('a'), 65);
        assert_eq!(char_to_keycode('z'), 90);
        assert_eq!(char_to_keycode('0'), 48);
        assert_eq!(char_to_keycode(' '), 32);
    }

    #[test]
    fn test_char_to_code() {
        assert_eq!(char_to_code('a'), "KeyA");
        assert_eq!(char_to_code('Z'), "KeyZ");
        assert_eq!(char_to_code('5'), "Digit5");
        assert_eq!(char_to_code(' '), "Space");
    }

    #[test]
    fn test_keyboard_sequence_from_text() {
        let seq = KeyboardSequence::from_text("Hi");
        // Should have 3 events per character: keydown, input, keyup
        assert_eq!(seq.events.len(), 6);
    }
}
