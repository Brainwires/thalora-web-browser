//! WebGL APIs
//!
//! This module provides WebGL rendering context implementation using wgpu.
//! https://www.khronos.org/webgl/

// Core types
pub mod buffer;
pub mod shader;
pub mod state;
pub mod texture;

// WebGL1 context (refactored into smaller files)
pub mod context;
pub mod constants;
pub mod methods_shader;
pub mod methods_buffer;
pub mod methods_texture;
pub mod methods_uniform;
pub mod methods_draw;

// WebGL2 context (refactored into smaller files)
pub mod context2;
pub mod constants2;
pub mod methods2_vao;
pub mod methods2_query;
pub mod methods2_transform;

#[cfg(test)]
mod tests;

// Re-exports
pub use context::WebGLRenderingContext;
pub use context2::WebGL2RenderingContext;
