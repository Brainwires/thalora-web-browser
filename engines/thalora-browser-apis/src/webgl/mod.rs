//! WebGL APIs
//!
//! This module provides WebGL rendering context implementation using wgpu.
//! https://www.khronos.org/webgl/

pub mod webgl_rendering_context;
pub mod webgl2_rendering_context;
pub mod shader;
pub mod buffer;
pub mod texture;
pub mod state;

// Re-exports
pub use webgl_rendering_context::WebGLRenderingContext;
pub use webgl2_rendering_context::WebGL2RenderingContext;
