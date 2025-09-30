//! Input handling for the graphical browser
//! 
//! This module handles keyboard and mouse input events, mapping them to browser actions
//! and web page interactions.

use winit::event::{WindowEvent, ElementState, MouseButton};
use crate::gui::{BrowserUI, TabManager};

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
    None,
}

/// Input handler for browser events
pub struct InputHandler {
    mouse_position: (f32, f32),
    mouse_pressed: bool,
    ctrl_pressed: bool,
    shift_pressed: bool,
    alt_pressed: bool,
}

impl InputHandler {
    /// Create a new input handler
    pub fn new() -> Self {
        Self {
            mouse_position: (0.0, 0.0),
            mouse_pressed: false,
            ctrl_pressed: false,
            shift_pressed: false,
            alt_pressed: false,
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
                        if *state == ElementState::Pressed {
                            // TODO: Show context menu
                            None
                        } else {
                            None
                        }
                    }
                    MouseButton::Middle => {
                        if *state == ElementState::Pressed {
                            // TODO: Handle middle click
                            None
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            }

            WindowEvent::MouseWheel { .. } => {
                // TODO: Handle scroll
                None
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
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}