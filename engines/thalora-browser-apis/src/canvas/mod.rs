//! Canvas 2D rendering API implementation
//!
//! This module provides real 2D rendering using tiny-skia as the backend.
//! https://html.spec.whatwg.org/multipage/canvas.html

pub mod rendering_context_2d;
pub mod canvas_state;
pub mod path;
pub mod html_canvas_element;
pub mod offscreen_canvas;

// Re-exports
pub use rendering_context_2d::CanvasRenderingContext2D;
pub use canvas_state::CanvasState;
pub use path::Path2D;
pub use html_canvas_element::HTMLCanvasElement;
pub use offscreen_canvas::OffscreenCanvas;
