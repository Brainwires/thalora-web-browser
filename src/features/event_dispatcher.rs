//! Event dispatcher for simulating trusted browser events
//!
//! Provides functionality to dispatch trusted mouse events that appear to come
//! from the browser (is_trusted: true) rather than JavaScript.

use super::mouse_simulation::{MousePath, MousePoint, ClickSequence};
use std::time::Duration;

/// Coordinates for event dispatching
#[derive(Debug, Clone, Copy)]
pub struct EventCoords {
    /// Client X (relative to viewport)
    pub client_x: f64,
    /// Client Y (relative to viewport)
    pub client_y: f64,
    /// Screen X (relative to screen)
    pub screen_x: f64,
    /// Screen Y (relative to screen)
    pub screen_y: f64,
    /// Page X (relative to document)
    pub page_x: f64,
    /// Page Y (relative to document)
    pub page_y: f64,
    /// Offset X (relative to target element)
    pub offset_x: f64,
    /// Offset Y (relative to target element)
    pub offset_y: f64,
}

impl EventCoords {
    /// Create event coordinates from a point (assumes same coords for all)
    pub fn from_point(x: f64, y: f64) -> Self {
        Self {
            client_x: x,
            client_y: y,
            screen_x: x,
            screen_y: y,
            page_x: x,
            page_y: y,
            offset_x: x,
            offset_y: y,
        }
    }

    /// Create with scroll offset
    pub fn with_scroll(x: f64, y: f64, scroll_x: f64, scroll_y: f64) -> Self {
        Self {
            client_x: x,
            client_y: y,
            screen_x: x,
            screen_y: y,
            page_x: x + scroll_x,
            page_y: y + scroll_y,
            offset_x: x,
            offset_y: y,
        }
    }
}

/// Mouse event types that can be dispatched
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventType {
    MouseMove,
    MouseDown,
    MouseUp,
    Click,
    DblClick,
    MouseEnter,
    MouseLeave,
    MouseOver,
    MouseOut,
}

impl MouseEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MouseMove => "mousemove",
            Self::MouseDown => "mousedown",
            Self::MouseUp => "mouseup",
            Self::Click => "click",
            Self::DblClick => "dblclick",
            Self::MouseEnter => "mouseenter",
            Self::MouseLeave => "mouseleave",
            Self::MouseOver => "mouseover",
            Self::MouseOut => "mouseout",
        }
    }

    pub fn bubbles(&self) -> bool {
        match self {
            Self::MouseEnter | Self::MouseLeave => false,
            _ => true,
        }
    }

    pub fn cancelable(&self) -> bool {
        match self {
            Self::MouseEnter | Self::MouseLeave => false,
            _ => true,
        }
    }
}

/// Mouse button constants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Primary = 0,    // Left button
    Auxiliary = 1,  // Middle button (wheel)
    Secondary = 2,  // Right button
    Fourth = 3,     // Back button
    Fifth = 4,      // Forward button
}

/// A single event in a sequence to be dispatched
#[derive(Debug, Clone)]
pub struct EventAction {
    /// Type of mouse event
    pub event_type: MouseEventType,
    /// Coordinates for the event
    pub coords: EventCoords,
    /// Movement delta from previous position
    pub movement: (f64, f64),
    /// Mouse button pressed (for button events)
    pub button: MouseButton,
    /// Buttons currently pressed (bitmask)
    pub buttons: u16,
    /// Delay before dispatching this event
    pub delay: Duration,
    /// Whether this is a trusted event
    pub is_trusted: bool,
}

impl EventAction {
    /// Create a mouse move event
    pub fn mouse_move(coords: EventCoords, movement: (f64, f64), delay: Duration) -> Self {
        Self {
            event_type: MouseEventType::MouseMove,
            coords,
            movement,
            button: MouseButton::Primary,
            buttons: 0,
            delay,
            is_trusted: true,
        }
    }

    /// Create a mouse down event
    pub fn mouse_down(coords: EventCoords, button: MouseButton, delay: Duration) -> Self {
        Self {
            event_type: MouseEventType::MouseDown,
            coords,
            movement: (0.0, 0.0),
            button,
            buttons: 1 << (button as u16),
            delay,
            is_trusted: true,
        }
    }

    /// Create a mouse up event
    pub fn mouse_up(coords: EventCoords, button: MouseButton, delay: Duration) -> Self {
        Self {
            event_type: MouseEventType::MouseUp,
            coords,
            movement: (0.0, 0.0),
            button,
            buttons: 0,
            delay,
            is_trusted: true,
        }
    }

    /// Create a click event
    pub fn click(coords: EventCoords, button: MouseButton, delay: Duration) -> Self {
        Self {
            event_type: MouseEventType::Click,
            coords,
            movement: (0.0, 0.0),
            button,
            buttons: 0,
            delay,
            is_trusted: true,
        }
    }
}

/// Generates a complete click event sequence
///
/// A proper click sequence consists of:
/// 1. Multiple mousemove events along a Bezier curve path
/// 2. mousedown event
/// 3. Short delay (human reaction time)
/// 4. mouseup event
/// 5. click event (synthetic, follows mouseup)
#[derive(Debug, Clone)]
pub struct EventSequence {
    /// All events in order
    pub events: Vec<EventAction>,
}

impl EventSequence {
    /// Create a click sequence from current position to target
    pub fn click_at(current_pos: (f64, f64), target_pos: (f64, f64)) -> Self {
        let click = ClickSequence::generate(current_pos, target_pos);
        Self::from_click_sequence(click)
    }

    /// Create a double-click sequence
    pub fn double_click_at(current_pos: (f64, f64), target_pos: (f64, f64)) -> Self {
        let click = ClickSequence::generate_double_click(current_pos, target_pos);
        Self::from_click_sequence(click)
    }

    /// Convert a ClickSequence to an EventSequence
    fn from_click_sequence(click: ClickSequence) -> Self {
        let mut events = Vec::new();

        // Add all mousemove events from the path
        for point in &click.movement_path.points {
            let coords = EventCoords::from_point(point.x, point.y);
            events.push(EventAction::mouse_move(
                coords,
                (point.movement_x, point.movement_y),
                point.delay,
            ));
        }

        // Get final position
        let final_coords = click.movement_path.points.last()
            .map(|p| EventCoords::from_point(p.x, p.y))
            .unwrap_or_else(|| EventCoords::from_point(0.0, 0.0));

        // Add mousedown
        events.push(EventAction::mouse_down(
            final_coords.clone(),
            MouseButton::Primary,
            Duration::from_millis(0),
        ));

        // Add mouseup after click duration
        events.push(EventAction::mouse_up(
            final_coords.clone(),
            MouseButton::Primary,
            Duration::from_millis(click.click_duration_ms),
        ));

        // Add click event
        events.push(EventAction::click(
            final_coords.clone(),
            MouseButton::Primary,
            Duration::from_millis(0),
        ));

        // If double-click, add second click sequence
        if click.is_double_click {
            events.push(EventAction::mouse_down(
                final_coords.clone(),
                MouseButton::Primary,
                Duration::from_millis(click.double_click_delay_ms),
            ));

            events.push(EventAction::mouse_up(
                final_coords.clone(),
                MouseButton::Primary,
                Duration::from_millis(click.click_duration_ms),
            ));

            events.push(EventAction::click(
                final_coords.clone(),
                MouseButton::Primary,
                Duration::from_millis(0),
            ));

            // Add dblclick event
            events.push(EventAction {
                event_type: MouseEventType::DblClick,
                coords: final_coords,
                movement: (0.0, 0.0),
                button: MouseButton::Primary,
                buttons: 0,
                delay: Duration::from_millis(0),
                is_trusted: true,
            });
        }

        Self { events }
    }

    /// Create just a mouse move sequence (no click)
    pub fn move_to(current_pos: (f64, f64), target_pos: (f64, f64), duration_ms: u64) -> Self {
        let path = MousePath::generate(current_pos, target_pos, duration_ms);

        let events = path.points.iter().map(|point| {
            let coords = EventCoords::from_point(point.x, point.y);
            EventAction::mouse_move(coords, (point.movement_x, point.movement_y), point.delay)
        }).collect();

        Self { events }
    }

    /// Get total duration of the sequence
    pub fn total_duration(&self) -> Duration {
        self.events.iter().map(|e| e.delay).sum()
    }

    /// Get the final position after all events
    pub fn final_position(&self) -> Option<(f64, f64)> {
        self.events.last().map(|e| (e.coords.client_x, e.coords.client_y))
    }
}

/// Generate JavaScript code to dispatch a trusted mouse event
///
/// Note: In a real browser, trusted events can only be dispatched by the browser itself.
/// This generates code that creates events with properties matching trusted events,
/// but the isTrusted property cannot be set to true from JavaScript.
///
/// For actual trusted event simulation, this would need to be integrated with
/// the browser's native event system.
pub fn generate_mouse_event_js(action: &EventAction, target_selector: &str) -> String {
    let event_type = action.event_type.as_str();
    let bubbles = action.event_type.bubbles();
    let cancelable = action.event_type.cancelable();

    format!(
        r#"(function() {{
    var target = document.querySelector('{}');
    if (!target) return false;

    var event = new MouseEvent('{}', {{
        bubbles: {},
        cancelable: {},
        view: window,
        detail: 1,
        screenX: {},
        screenY: {},
        clientX: {},
        clientY: {},
        pageX: {},
        pageY: {},
        offsetX: {},
        offsetY: {},
        movementX: {},
        movementY: {},
        button: {},
        buttons: {},
        ctrlKey: false,
        shiftKey: false,
        altKey: false,
        metaKey: false,
        relatedTarget: null
    }});

    // Note: isTrusted cannot be set from JavaScript
    // Object.defineProperty(event, 'isTrusted', {{ value: true }});

    return target.dispatchEvent(event);
}})()"#,
        target_selector,
        event_type,
        bubbles,
        cancelable,
        action.coords.screen_x,
        action.coords.screen_y,
        action.coords.client_x,
        action.coords.client_y,
        action.coords.page_x,
        action.coords.page_y,
        action.coords.offset_x,
        action.coords.offset_y,
        action.movement.0,
        action.movement.1,
        action.button as i16,
        action.buttons,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_click_sequence_generation() {
        let sequence = EventSequence::click_at((0.0, 0.0), (100.0, 100.0));

        // Should have mousemove events + mousedown + mouseup + click
        assert!(sequence.events.len() > 3);

        // Last events should be mousedown, mouseup, click
        let len = sequence.events.len();
        assert_eq!(sequence.events[len - 3].event_type, MouseEventType::MouseDown);
        assert_eq!(sequence.events[len - 2].event_type, MouseEventType::MouseUp);
        assert_eq!(sequence.events[len - 1].event_type, MouseEventType::Click);

        // All events should be trusted
        assert!(sequence.events.iter().all(|e| e.is_trusted));
    }

    #[test]
    fn test_double_click_sequence() {
        let sequence = EventSequence::double_click_at((0.0, 0.0), (100.0, 100.0));

        // Should have dblclick at the end
        let last = sequence.events.last().unwrap();
        assert_eq!(last.event_type, MouseEventType::DblClick);

        // Should have two click events
        let click_count = sequence.events.iter()
            .filter(|e| e.event_type == MouseEventType::Click)
            .count();
        assert_eq!(click_count, 2);
    }

    #[test]
    fn test_event_js_generation() {
        let action = EventAction::click(
            EventCoords::from_point(100.0, 200.0),
            MouseButton::Primary,
            Duration::from_millis(0),
        );

        let js = generate_mouse_event_js(&action, "#button");

        assert!(js.contains("MouseEvent"));
        assert!(js.contains("click"));
        assert!(js.contains("#button"));
        assert!(js.contains("clientX: 100"));
        assert!(js.contains("clientY: 200"));
    }
}
