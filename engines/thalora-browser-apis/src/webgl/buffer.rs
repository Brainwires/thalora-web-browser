//! WebGL Buffer Operations
//!
//! Handles WebGL buffer creation, binding, and data operations.

use std::sync::atomic::{AtomicU32, Ordering};

use super::state::WebGLConstants;

static BUFFER_ID_COUNTER: AtomicU32 = AtomicU32::new(1);
static FRAMEBUFFER_ID_COUNTER: AtomicU32 = AtomicU32::new(1);
static RENDERBUFFER_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

/// WebGL Buffer object
#[derive(Debug, Clone)]
pub struct WebGLBuffer {
    /// Unique buffer ID
    pub id: u32,
    /// Buffer target (ARRAY_BUFFER or ELEMENT_ARRAY_BUFFER)
    pub target: Option<u32>,
    /// Buffer usage hint
    pub usage: u32,
    /// Buffer data
    pub data: Vec<u8>,
    /// Size in bytes
    pub size: usize,
    /// Marked for deletion
    pub delete_pending: bool,
    /// WGPU buffer handle (index into buffer pool)
    pub wgpu_buffer: Option<u32>,
}

impl WebGLBuffer {
    /// Create a new buffer
    pub fn new() -> Self {
        Self {
            id: BUFFER_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            target: None,
            usage: WebGLConstants::STATIC_DRAW,
            data: Vec::new(),
            size: 0,
            delete_pending: false,
            wgpu_buffer: None,
        }
    }

    /// Bind the buffer to a target
    pub fn bind(&mut self, target: u32) {
        self.target = Some(target);
    }

    /// Unbind the buffer
    pub fn unbind(&mut self) {
        self.target = None;
    }

    /// Set buffer data
    pub fn set_data(&mut self, data: &[u8], usage: u32) {
        self.data = data.to_vec();
        self.size = data.len();
        self.usage = usage;
    }

    /// Set buffer sub-data
    pub fn set_sub_data(&mut self, offset: usize, data: &[u8]) {
        if offset + data.len() <= self.data.len() {
            self.data[offset..offset + data.len()].copy_from_slice(data);
        }
    }

    /// Allocate buffer without data
    pub fn allocate(&mut self, size: usize, usage: u32) {
        self.data = vec![0u8; size];
        self.size = size;
        self.usage = usage;
    }

    /// Get buffer parameter
    pub fn get_parameter(&self, pname: u32) -> Option<i32> {
        match pname {
            WebGLConstants::BUFFER_SIZE => Some(self.size as i32),
            WebGLConstants::BUFFER_USAGE => Some(self.usage as i32),
            _ => None,
        }
    }

    /// Convert usage hint to WGPU usage
    pub fn wgpu_usage(&self, target: u32) -> wgpu::BufferUsages {
        let mut usage = wgpu::BufferUsages::COPY_DST;

        match target {
            WebGLConstants::ARRAY_BUFFER => {
                usage |= wgpu::BufferUsages::VERTEX;
            }
            WebGLConstants::ELEMENT_ARRAY_BUFFER => {
                usage |= wgpu::BufferUsages::INDEX;
            }
            _ => {}
        }

        // Add COPY_SRC for reading
        usage |= wgpu::BufferUsages::COPY_SRC;

        usage
    }
}

impl Default for WebGLBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// WebGL Framebuffer object
#[derive(Debug, Clone)]
pub struct WebGLFramebuffer {
    /// Unique framebuffer ID
    pub id: u32,
    /// Color attachment (texture or renderbuffer)
    pub color_attachment: Option<FramebufferAttachment>,
    /// Depth attachment
    pub depth_attachment: Option<FramebufferAttachment>,
    /// Stencil attachment
    pub stencil_attachment: Option<FramebufferAttachment>,
    /// Depth-stencil combined attachment
    pub depth_stencil_attachment: Option<FramebufferAttachment>,
    /// Framebuffer width
    pub width: u32,
    /// Framebuffer height
    pub height: u32,
    /// Marked for deletion
    pub delete_pending: bool,
}

/// Framebuffer attachment type
#[derive(Debug, Clone)]
pub enum FramebufferAttachment {
    /// Texture attachment
    Texture {
        /// Texture ID
        texture_id: u32,
        /// Mipmap level
        level: i32,
        /// Cube map face (for cube textures)
        face: Option<u32>,
    },
    /// Renderbuffer attachment
    Renderbuffer {
        /// Renderbuffer ID
        renderbuffer_id: u32,
    },
}

impl WebGLFramebuffer {
    /// Create a new framebuffer
    pub fn new() -> Self {
        Self {
            id: FRAMEBUFFER_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            color_attachment: None,
            depth_attachment: None,
            stencil_attachment: None,
            depth_stencil_attachment: None,
            width: 0,
            height: 0,
            delete_pending: false,
        }
    }

    /// Attach a texture
    pub fn attach_texture(
        &mut self,
        attachment: u32,
        texture_id: u32,
        level: i32,
        face: Option<u32>,
    ) {
        let attach = FramebufferAttachment::Texture {
            texture_id,
            level,
            face,
        };

        match attachment {
            WebGLConstants::COLOR_ATTACHMENT0 => {
                self.color_attachment = Some(attach);
            }
            WebGLConstants::DEPTH_ATTACHMENT => {
                self.depth_attachment = Some(attach);
            }
            WebGLConstants::STENCIL_ATTACHMENT => {
                self.stencil_attachment = Some(attach);
            }
            WebGLConstants::DEPTH_STENCIL_ATTACHMENT => {
                self.depth_stencil_attachment = Some(attach);
            }
            _ => {}
        }
    }

    /// Attach a renderbuffer
    pub fn attach_renderbuffer(&mut self, attachment: u32, renderbuffer_id: u32) {
        let attach = FramebufferAttachment::Renderbuffer { renderbuffer_id };

        match attachment {
            WebGLConstants::COLOR_ATTACHMENT0 => {
                self.color_attachment = Some(attach);
            }
            WebGLConstants::DEPTH_ATTACHMENT => {
                self.depth_attachment = Some(attach);
            }
            WebGLConstants::STENCIL_ATTACHMENT => {
                self.stencil_attachment = Some(attach);
            }
            WebGLConstants::DEPTH_STENCIL_ATTACHMENT => {
                self.depth_stencil_attachment = Some(attach);
            }
            _ => {}
        }
    }

    /// Check framebuffer status
    pub fn check_status(&self) -> u32 {
        // Check if we have at least a color attachment
        if self.color_attachment.is_none()
            && self.depth_attachment.is_none()
            && self.stencil_attachment.is_none()
            && self.depth_stencil_attachment.is_none()
        {
            return WebGLConstants::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT;
        }

        // For now, assume complete if we have any attachment
        WebGLConstants::FRAMEBUFFER_COMPLETE
    }

    /// Get attachment parameter
    pub fn get_attachment_parameter(&self, attachment: u32, pname: u32) -> Option<i32> {
        let attach = match attachment {
            WebGLConstants::COLOR_ATTACHMENT0 => &self.color_attachment,
            WebGLConstants::DEPTH_ATTACHMENT => &self.depth_attachment,
            WebGLConstants::STENCIL_ATTACHMENT => &self.stencil_attachment,
            WebGLConstants::DEPTH_STENCIL_ATTACHMENT => &self.depth_stencil_attachment,
            _ => return None,
        };

        match attach {
            Some(FramebufferAttachment::Texture {
                texture_id,
                level,
                face,
            }) => match pname {
                WebGLConstants::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE => {
                    Some(WebGLConstants::TEXTURE as i32)
                }
                WebGLConstants::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME => Some(*texture_id as i32),
                WebGLConstants::FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL => Some(*level),
                WebGLConstants::FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE => {
                    Some(face.unwrap_or(0) as i32)
                }
                _ => None,
            },
            Some(FramebufferAttachment::Renderbuffer { renderbuffer_id }) => match pname {
                WebGLConstants::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE => {
                    Some(WebGLConstants::RENDERBUFFER as i32)
                }
                WebGLConstants::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME => Some(*renderbuffer_id as i32),
                _ => None,
            },
            None => match pname {
                WebGLConstants::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE => {
                    Some(WebGLConstants::NONE as i32)
                }
                _ => None,
            },
        }
    }
}

impl Default for WebGLFramebuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// WebGL Renderbuffer object
#[derive(Debug, Clone)]
pub struct WebGLRenderbuffer {
    /// Unique renderbuffer ID
    pub id: u32,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
    /// Internal format
    pub internal_format: u32,
    /// Marked for deletion
    pub delete_pending: bool,
    /// WGPU texture handle (for renderbuffer storage)
    pub wgpu_texture: Option<u32>,
}

impl WebGLRenderbuffer {
    /// Create a new renderbuffer
    pub fn new() -> Self {
        Self {
            id: RENDERBUFFER_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            width: 0,
            height: 0,
            internal_format: WebGLConstants::RGBA4,
            delete_pending: false,
            wgpu_texture: None,
        }
    }

    /// Set renderbuffer storage
    pub fn storage(&mut self, internal_format: u32, width: u32, height: u32) {
        self.internal_format = internal_format;
        self.width = width;
        self.height = height;
    }

    /// Get renderbuffer parameter
    pub fn get_parameter(&self, pname: u32) -> Option<i32> {
        match pname {
            WebGLConstants::RENDERBUFFER_WIDTH => Some(self.width as i32),
            WebGLConstants::RENDERBUFFER_HEIGHT => Some(self.height as i32),
            WebGLConstants::RENDERBUFFER_INTERNAL_FORMAT => Some(self.internal_format as i32),
            WebGLConstants::RENDERBUFFER_RED_SIZE => {
                Some(get_format_component_size(self.internal_format, 0))
            }
            WebGLConstants::RENDERBUFFER_GREEN_SIZE => {
                Some(get_format_component_size(self.internal_format, 1))
            }
            WebGLConstants::RENDERBUFFER_BLUE_SIZE => {
                Some(get_format_component_size(self.internal_format, 2))
            }
            WebGLConstants::RENDERBUFFER_ALPHA_SIZE => {
                Some(get_format_component_size(self.internal_format, 3))
            }
            WebGLConstants::RENDERBUFFER_DEPTH_SIZE => {
                Some(get_format_depth_size(self.internal_format))
            }
            WebGLConstants::RENDERBUFFER_STENCIL_SIZE => {
                Some(get_format_stencil_size(self.internal_format))
            }
            _ => None,
        }
    }

    /// Convert internal format to WGPU format
    pub fn wgpu_format(&self) -> wgpu::TextureFormat {
        match self.internal_format {
            WebGLConstants::RGBA4 => wgpu::TextureFormat::Rgba8Unorm,
            WebGLConstants::RGB5_A1 => wgpu::TextureFormat::Rgba8Unorm,
            WebGLConstants::RGB565 => wgpu::TextureFormat::Rgba8Unorm,
            WebGLConstants::DEPTH_COMPONENT16 => wgpu::TextureFormat::Depth16Unorm,
            WebGLConstants::STENCIL_INDEX8 => wgpu::TextureFormat::Stencil8,
            WebGLConstants::DEPTH_STENCIL => wgpu::TextureFormat::Depth24PlusStencil8,
            _ => wgpu::TextureFormat::Rgba8Unorm,
        }
    }
}

impl Default for WebGLRenderbuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Get component size for internal format
fn get_format_component_size(format: u32, component: u32) -> i32 {
    match format {
        WebGLConstants::RGBA4 => 4,
        WebGLConstants::RGB5_A1 => {
            if component == 3 {
                1
            } else {
                5
            }
        }
        WebGLConstants::RGB565 => match component {
            0 | 2 => 5,
            1 => 6,
            _ => 0,
        },
        _ => 0,
    }
}

/// Get depth size for internal format
fn get_format_depth_size(format: u32) -> i32 {
    match format {
        WebGLConstants::DEPTH_COMPONENT16 => 16,
        WebGLConstants::DEPTH_STENCIL => 24,
        _ => 0,
    }
}

/// Get stencil size for internal format
fn get_format_stencil_size(format: u32) -> i32 {
    match format {
        WebGLConstants::STENCIL_INDEX8 => 8,
        WebGLConstants::DEPTH_STENCIL => 8,
        _ => 0,
    }
}

/// Vertex attribute array state
#[derive(Debug, Clone)]
pub struct VertexAttribArray {
    /// Whether the array is enabled
    pub enabled: bool,
    /// Size (number of components per vertex)
    pub size: i32,
    /// Data type
    pub data_type: u32,
    /// Normalized
    pub normalized: bool,
    /// Stride between vertices
    pub stride: i32,
    /// Offset in buffer
    pub offset: i32,
    /// Bound buffer ID
    pub buffer_id: Option<u32>,
    /// Divisor for instanced rendering (WebGL2)
    pub divisor: u32,
}

impl Default for VertexAttribArray {
    fn default() -> Self {
        Self {
            enabled: false,
            size: 4,
            data_type: WebGLConstants::FLOAT,
            normalized: false,
            stride: 0,
            offset: 0,
            buffer_id: None,
            divisor: 0,
        }
    }
}

impl VertexAttribArray {
    /// Get the byte size of the data type
    pub fn type_size(&self) -> usize {
        match self.data_type {
            WebGLConstants::BYTE | WebGLConstants::UNSIGNED_BYTE => 1,
            WebGLConstants::SHORT | WebGLConstants::UNSIGNED_SHORT => 2,
            WebGLConstants::INT | WebGLConstants::UNSIGNED_INT | WebGLConstants::FLOAT => 4,
            _ => 4,
        }
    }

    /// Get actual stride (0 means tightly packed)
    pub fn actual_stride(&self) -> usize {
        if self.stride == 0 {
            self.size as usize * self.type_size()
        } else {
            self.stride as usize
        }
    }
}
