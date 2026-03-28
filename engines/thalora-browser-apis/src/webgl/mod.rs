//! WebGL APIs
//!
//! This module provides WebGL rendering context implementation.
//! - Native builds use wgpu for GPU access
//! - WASM builds delegate to the browser's native WebGL
//! https://www.khronos.org/webgl/

// Shared constants (no native deps)
pub mod constants;
pub mod constants2;
pub mod state;

// Native implementation using wgpu
#[cfg(feature = "native")]
pub mod buffer;
#[cfg(feature = "native")]
pub mod context;
#[cfg(feature = "native")]
pub mod context2;
#[cfg(feature = "native")]
pub mod methods2_query;
#[cfg(feature = "native")]
pub mod methods2_transform;
#[cfg(feature = "native")]
pub mod methods2_vao;
#[cfg(feature = "native")]
pub mod methods_buffer;
#[cfg(feature = "native")]
pub mod methods_draw;
#[cfg(feature = "native")]
pub mod methods_shader;
#[cfg(feature = "native")]
pub mod methods_texture;
#[cfg(feature = "native")]
pub mod methods_uniform;
#[cfg(feature = "native")]
pub mod shader;
#[cfg(feature = "native")]
pub mod texture;

// WASM stubs - actual WebGL is used through web-sys in JavaScript
#[cfg(feature = "wasm")]
pub mod context2_wasm;
#[cfg(feature = "wasm")]
pub mod context_wasm;

// Re-export WASM stubs with native module names for uniform API
#[cfg(feature = "wasm")]
pub use context_wasm as context;
#[cfg(feature = "wasm")]
pub use context2_wasm as context2;

#[cfg(test)]
mod tests;

// Re-exports at module level
pub use context::WebGLRenderingContext;
pub use context2::WebGL2RenderingContext;
