//! Input handling for the graphical browser
//!
//! This module handles keyboard and mouse input events, mapping them to browser actions
//! and web page interactions.

use winit::event::{WindowEvent, ElementState, MouseButton, MouseScrollDelta};

/// Browser actions that can be triggered by input events
#[derive(Debug, Clone)]
pub enum BrowserAction {
    Quit,
    NewTab,
    CloseTab,
    Navigate(String),
    Reload,
    GoBack,
    GoForward,
    ZoomIn,
    ZoomOut,
    ResetZoom,
    ToggleDevTools,
    FocusAddressBar,
    SwitchTab(u32),
    ShowContextMenu(f32, f32),
    OpenLinkInNewTab(String),
    Scroll(f32, f32),
    None,
}

/// Input handler for browser events
pub struct InputHandler {
    mouse_position: (f32, f32),
    mouse_pressed: bool,
    right_mouse_pressed: bool,
    middle_mouse_pressed: bool,
    ctrl_pressed: bool,
    shift_pressed: bool,
    alt_pressed: bool,
    scroll_delta: (f32, f32),
}

impl InputHandler {
    /// Create a new input handler
    pub fn new() -> Self {
        Self {
            mouse_position: (0.0, 0.0),
            mouse_pressed: false,
            right_mouse_pressed: false,
            middle_mouse_pressed: false,
            ctrl_pressed: false,
            shift_pressed: false,
            alt_pressed: false,
            scroll_delta: (0.0, 0.0),
        }
    }

    /// Handle window events and return action to perform
    pub fn handle_event(&mut self, event: &WindowEvent) -> Option<BrowserAction> {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = (position.x as f32, position.y as f32);
                None
            }

            WindowEvent::MouseInput { state, button, .. } => {
                match button {
                    MouseButton::Left => {
                        self.mouse_pressed = *state == ElementState::Pressed;
                        None
                    }
                    MouseButton::Right => {
                        self.right_mouse_pressed = *state == ElementState::Pressed;
                        if *state == ElementState::Pressed {
                            // Show context menu at current mouse position
                            let (x, y) = self.mouse_position;
                            tracing::debug!("Right-click at ({}, {})", x, y);
                            Some(BrowserAction::ShowContextMenu(x, y))
                        } else {
                            None
                        }
                    }
                    MouseButton::Middle => {
                        self.middle_mouse_pressed = *state == ElementState::Pressed;
                        if *state == ElementState::Pressed {
                            // Middle click - could be used for auto-scroll or open link in new tab
                            tracing::debug!("Middle-click at {:?}", self.mouse_position);
                            // For now, middle-click acts as a scroll toggle
                            None
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                // Handle scroll events
                let (dx, dy) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        // Line-based scrolling (most mice)
                        // Multiply by a factor for smooth scrolling
                        (*x * 40.0, *y * 40.0)
                    }
                    MouseScrollDelta::PixelDelta(pos) => {
                        // Pixel-based scrolling (trackpads)
                        (pos.x as f32, pos.y as f32)
                    }
                };
                self.scroll_delta = (dx, dy);

                // Handle zoom with Ctrl+scroll
                if self.ctrl_pressed {
                    if dy > 0.0 {
                        return Some(BrowserAction::ZoomIn);
                    } else if dy < 0.0 {
                        return Some(BrowserAction::ZoomOut);
                    }
                }

                // Regular scroll - egui handles this internally, but we track it
                Some(BrowserAction::Scroll(dx, dy))
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if let winit::keyboard::PhysicalKey::Code(keycode) = event.physical_key {
                    let pressed = event.state == ElementState::Pressed;
                    
                    // Track modifier keys
                    match keycode {
                        winit::keyboard::KeyCode::ControlLeft | winit::keyboard::KeyCode::ControlRight => {
                            self.ctrl_pressed = pressed;
                            None
                        }
                        winit::keyboard::KeyCode::ShiftLeft | winit::keyboard::KeyCode::ShiftRight => {
                            self.shift_pressed = pressed;
                            None
                        }
                        winit::keyboard::KeyCode::AltLeft | winit::keyboard::KeyCode::AltRight => {
                            self.alt_pressed = pressed;
                            None
                        }
                        _ => {
                            if pressed {
                                self.handle_key_press(keycode)
                            } else {
                                None
                            }
                        }
                    }
                } else {
                    None
                }
            }

            _ => None
        }
    }

    /// Handle key press events
    fn handle_key_press(&self, key: winit::keyboard::KeyCode) -> Option<BrowserAction> {
        // Handle keyboard shortcuts
        if self.ctrl_pressed {
            match key {
                winit::keyboard::KeyCode::KeyQ => Some(BrowserAction::Quit),
                winit::keyboard::KeyCode::KeyT => Some(BrowserAction::NewTab),
                winit::keyboard::KeyCode::KeyW => Some(BrowserAction::CloseTab),
                winit::keyboard::KeyCode::KeyR => Some(BrowserAction::Reload),
                winit::keyboard::KeyCode::KeyL => Some(BrowserAction::FocusAddressBar),
                winit::keyboard::KeyCode::Equal => Some(BrowserAction::ZoomIn),
                winit::keyboard::KeyCode::Minus => Some(BrowserAction::ZoomOut),
                winit::keyboard::KeyCode::Digit0 => Some(BrowserAction::ResetZoom),
                _ => None
            }
        } else {
            match key {
                winit::keyboard::KeyCode::F5 => Some(BrowserAction::Reload),
                winit::keyboard::KeyCode::F12 => Some(BrowserAction::ToggleDevTools),
                _ => None
            }
        }
    }

    /// Get current mouse position
    pub fn mouse_position(&self) -> (f32, f32) {
        self.mouse_position
    }

    /// Check if mouse is currently pressed
    pub fn is_mouse_pressed(&self) -> bool {
        self.mouse_pressed
    }

    /// Check if Ctrl is currently pressed
    pub fn is_ctrl_pressed(&self) -> bool {
        self.ctrl_pressed
    }

    /// Check if Shift is currently pressed
    pub fn is_shift_pressed(&self) -> bool {
        self.shift_pressed
    }

    /// Check if Alt is currently pressed
    pub fn is_alt_pressed(&self) -> bool {
        self.alt_pressed
    }

    /// Check if right mouse button is pressed
    pub fn is_right_mouse_pressed(&self) -> bool {
        self.right_mouse_pressed
    }

    /// Check if middle mouse button is pressed
    pub fn is_middle_mouse_pressed(&self) -> bool {
        self.middle_mouse_pressed
    }

    /// Get the current scroll delta and reset it
    pub fn take_scroll_delta(&mut self) -> (f32, f32) {
        let delta = self.scroll_delta;
        self.scroll_delta = (0.0, 0.0);
        delta
    }

    /// Get scroll delta without resetting
    pub fn scroll_delta(&self) -> (f32, f32) {
        self.scroll_delta
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}