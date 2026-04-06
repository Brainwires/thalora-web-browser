//! WebGL Texture Operations
//!
//! Handles WebGL texture creation, binding, and data operations.

use std::sync::atomic::{AtomicU32, Ordering};

use super::state::WebGLConstants;

static TEXTURE_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

/// WebGL Texture object
#[derive(Debug, Clone)]
pub struct WebGLTexture {
    /// Unique texture ID
    pub id: u32,
    /// Texture target (TEXTURE_2D or TEXTURE_CUBE_MAP)
    pub target: Option<u32>,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
    /// Internal format
    pub internal_format: u32,
    /// Texture format
    pub format: u32,
    /// Data type
    pub data_type: u32,
    /// Minification filter
    pub min_filter: u32,
    /// Magnification filter
    pub mag_filter: u32,
    /// Wrap S
    pub wrap_s: u32,
    /// Wrap T
    pub wrap_t: u32,
    /// Texture data (for 2D textures)
    pub data: Option<Vec<u8>>,
    /// Cube map face data (for cube maps)
    pub cube_faces: [Option<Vec<u8>>; 6],
    /// Whether mipmaps have been generated
    pub mipmaps_generated: bool,
    /// Mipmap levels
    pub mipmap_levels: Vec<TextureMipLevel>,
    /// Marked for deletion
    pub delete_pending: bool,
    /// WGPU texture handle
    pub wgpu_texture: Option<u32>,
    /// Whether the texture is complete
    pub complete: bool,
}

/// Mipmap level data
#[derive(Debug, Clone)]
pub struct TextureMipLevel {
    /// Level number
    pub level: u32,
    /// Width at this level
    pub width: u32,
    /// Height at this level
    pub height: u32,
    /// Data for this level
    pub data: Vec<u8>,
}

impl WebGLTexture {
    /// Create a new texture
    pub fn new() -> Self {
        Self {
            id: TEXTURE_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            target: None,
            width: 0,
            height: 0,
            internal_format: WebGLConstants::RGBA,
            format: WebGLConstants::RGBA,
            data_type: WebGLConstants::UNSIGNED_BYTE,
            min_filter: WebGLConstants::NEAREST_MIPMAP_LINEAR,
            mag_filter: WebGLConstants::LINEAR,
            wrap_s: WebGLConstants::REPEAT,
            wrap_t: WebGLConstants::REPEAT,
            data: None,
            cube_faces: [None, None, None, None, None, None],
            mipmaps_generated: false,
            mipmap_levels: Vec::new(),
            delete_pending: false,
            wgpu_texture: None,
            complete: false,
        }
    }

    /// Bind the texture to a target
    pub fn bind(&mut self, target: u32) {
        self.target = Some(target);
    }

    /// Unbind the texture
    pub fn unbind(&mut self) {
        self.target = None;
    }

    /// Set texture image data
    pub fn tex_image_2d(
        &mut self,
        _target: u32,
        level: i32,
        internal_format: u32,
        width: u32,
        height: u32,
        format: u32,
        data_type: u32,
        data: Option<&[u8]>,
    ) {
        if level == 0 {
            self.width = width;
            self.height = height;
            self.internal_format = internal_format;
            self.format = format;
            self.data_type = data_type;
            self.data = data.map(|d| d.to_vec());
            self.update_completeness();
        } else {
            // Store mipmap level
            let mip = TextureMipLevel {
                level: level as u32,
                width,
                height,
                data: data.map(|d| d.to_vec()).unwrap_or_default(),
            };

            // Replace or insert
            if let Some(existing) = self
                .mipmap_levels
                .iter_mut()
                .find(|m| m.level == level as u32)
            {
                *existing = mip;
            } else {
                self.mipmap_levels.push(mip);
            }
        }
    }

    /// Set texture sub-image data
    pub fn tex_sub_image_2d(
        &mut self,
        _target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        width: u32,
        height: u32,
        format: u32,
        data_type: u32,
        data: &[u8],
    ) {
        let pixel_size = get_pixel_size(format, data_type);

        if level == 0 {
            if let Some(ref mut tex_data) = self.data {
                let tex_width = self.width as usize;
                let row_size = width as usize * pixel_size;

                for row in 0..height as usize {
                    let src_start = row * row_size;
                    let dst_start =
                        ((y_offset as usize + row) * tex_width + x_offset as usize) * pixel_size;

                    if src_start + row_size <= data.len() && dst_start + row_size <= tex_data.len()
                    {
                        tex_data[dst_start..dst_start + row_size]
                            .copy_from_slice(&data[src_start..src_start + row_size]);
                    }
                }
            }
        } else {
            // Handle mipmap level sub-image
            if let Some(mip) = self
                .mipmap_levels
                .iter_mut()
                .find(|m| m.level == level as u32)
            {
                let mip_width = mip.width as usize;
                let row_size = width as usize * pixel_size;

                for row in 0..height as usize {
                    let src_start = row * row_size;
                    let dst_start =
                        ((y_offset as usize + row) * mip_width + x_offset as usize) * pixel_size;

                    if src_start + row_size <= data.len() && dst_start + row_size <= mip.data.len()
                    {
                        mip.data[dst_start..dst_start + row_size]
                            .copy_from_slice(&data[src_start..src_start + row_size]);
                    }
                }
            }
        }
    }

    /// Set cube map face data
    pub fn tex_image_cube_face(
        &mut self,
        face: u32,
        level: i32,
        internal_format: u32,
        width: u32,
        height: u32,
        format: u32,
        data_type: u32,
        data: Option<&[u8]>,
    ) {
        let face_index = (face - WebGLConstants::TEXTURE_CUBE_MAP_POSITIVE_X) as usize;
        if face_index >= 6 {
            return;
        }

        if level == 0 {
            self.width = width;
            self.height = height;
            self.internal_format = internal_format;
            self.format = format;
            self.data_type = data_type;
            self.cube_faces[face_index] = data.map(|d| d.to_vec());
            self.update_completeness();
        }
    }

    /// Set texture parameter
    pub fn tex_parameter(&mut self, pname: u32, param: u32) {
        match pname {
            WebGLConstants::TEXTURE_MIN_FILTER => {
                self.min_filter = param;
            }
            WebGLConstants::TEXTURE_MAG_FILTER => {
                self.mag_filter = param;
            }
            WebGLConstants::TEXTURE_WRAP_S => {
                self.wrap_s = param;
            }
            WebGLConstants::TEXTURE_WRAP_T => {
                self.wrap_t = param;
            }
            _ => {}
        }
    }

    /// Get texture parameter
    pub fn get_tex_parameter(&self, pname: u32) -> Option<u32> {
        match pname {
            WebGLConstants::TEXTURE_MIN_FILTER => Some(self.min_filter),
            WebGLConstants::TEXTURE_MAG_FILTER => Some(self.mag_filter),
            WebGLConstants::TEXTURE_WRAP_S => Some(self.wrap_s),
            WebGLConstants::TEXTURE_WRAP_T => Some(self.wrap_t),
            _ => None,
        }
    }

    /// Generate mipmaps
    pub fn generate_mipmap(&mut self) {
        if self.data.is_none() && self.target != Some(WebGLConstants::TEXTURE_CUBE_MAP) {
            return;
        }

        // Clear existing mipmaps
        self.mipmap_levels.clear();

        let mut level = 1u32;
        let mut w = self.width / 2;
        let mut h = self.height / 2;

        // Calculate mipmap levels
        while w >= 1 && h >= 1 {
            let pixel_size = get_pixel_size(self.format, self.data_type);
            let data_size = (w * h) as usize * pixel_size;

            // For now, just allocate empty mipmaps
            // Real implementation would downsample the texture data
            self.mipmap_levels.push(TextureMipLevel {
                level,
                width: w,
                height: h,
                data: vec![0u8; data_size],
            });

            // Generate mipmap data by simple box filter
            if let Some(ref base_data) = self.data
                && level == 1
                && !base_data.is_empty()
            {
                let mip = self.mipmap_levels.last_mut().unwrap();
                generate_mipmap_level(
                    base_data,
                    self.width,
                    self.height,
                    &mut mip.data,
                    w,
                    h,
                    pixel_size,
                );
            }

            level += 1;
            w /= 2;
            h /= 2;
        }

        self.mipmaps_generated = true;
    }

    /// Check if texture is complete for rendering
    fn update_completeness(&mut self) {
        if self.target == Some(WebGLConstants::TEXTURE_CUBE_MAP) {
            // Cube map is complete if all 6 faces have data
            self.complete = self.cube_faces.iter().all(|f| f.is_some());
        } else {
            // 2D texture is complete if it has data or valid dimensions
            self.complete = self.width > 0 && self.height > 0;
        }
    }

    /// Check if texture uses power-of-two dimensions
    pub fn is_power_of_two(&self) -> bool {
        is_power_of_two(self.width) && is_power_of_two(self.height)
    }

    /// Convert to WGPU texture format
    pub fn wgpu_format(&self) -> wgpu::TextureFormat {
        match (self.internal_format, self.data_type) {
            (WebGLConstants::RGBA, WebGLConstants::UNSIGNED_BYTE) => {
                wgpu::TextureFormat::Rgba8Unorm
            }
            (WebGLConstants::RGB, WebGLConstants::UNSIGNED_BYTE) => wgpu::TextureFormat::Rgba8Unorm,
            (WebGLConstants::LUMINANCE, WebGLConstants::UNSIGNED_BYTE) => {
                wgpu::TextureFormat::R8Unorm
            }
            (WebGLConstants::ALPHA, WebGLConstants::UNSIGNED_BYTE) => wgpu::TextureFormat::R8Unorm,
            (WebGLConstants::LUMINANCE_ALPHA, WebGLConstants::UNSIGNED_BYTE) => {
                wgpu::TextureFormat::Rg8Unorm
            }
            (WebGLConstants::DEPTH_COMPONENT, WebGLConstants::UNSIGNED_SHORT) => {
                wgpu::TextureFormat::Depth16Unorm
            }
            (WebGLConstants::DEPTH_COMPONENT, WebGLConstants::UNSIGNED_INT) => {
                wgpu::TextureFormat::Depth32Float
            }
            _ => wgpu::TextureFormat::Rgba8Unorm,
        }
    }

    /// Convert to WGPU sampler descriptor
    pub fn wgpu_sampler_descriptor(&self) -> wgpu::SamplerDescriptor<'static> {
        wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: webgl_wrap_to_wgpu(self.wrap_s),
            address_mode_v: webgl_wrap_to_wgpu(self.wrap_t),
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: webgl_filter_to_wgpu(self.mag_filter),
            min_filter: webgl_filter_to_wgpu(self.min_filter),
            mipmap_filter: webgl_mipmap_filter_to_wgpu(self.min_filter),
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        }
    }
}

impl Default for WebGLTexture {
    fn default() -> Self {
        Self::new()
    }
}

/// Get pixel size in bytes
pub fn get_pixel_size(format: u32, data_type: u32) -> usize {
    let components = match format {
        WebGLConstants::ALPHA | WebGLConstants::LUMINANCE | WebGLConstants::DEPTH_COMPONENT => 1,
        WebGLConstants::LUMINANCE_ALPHA => 2,
        WebGLConstants::RGB => 3,
        WebGLConstants::RGBA => 4,
        _ => 4,
    };

    let type_size = match data_type {
        WebGLConstants::UNSIGNED_BYTE | WebGLConstants::BYTE => 1,
        WebGLConstants::UNSIGNED_SHORT | WebGLConstants::SHORT => 2,
        WebGLConstants::UNSIGNED_SHORT_5_6_5
        | WebGLConstants::UNSIGNED_SHORT_4_4_4_4
        | WebGLConstants::UNSIGNED_SHORT_5_5_5_1 => {
            return 2; // Packed formats
        }
        WebGLConstants::UNSIGNED_INT | WebGLConstants::INT | WebGLConstants::FLOAT => 4,
        _ => 1,
    };

    components * type_size
}

/// Check if value is power of two
fn is_power_of_two(n: u32) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

/// Generate mipmap level by box filtering
fn generate_mipmap_level(
    src: &[u8],
    src_width: u32,
    src_height: u32,
    dst: &mut [u8],
    dst_width: u32,
    dst_height: u32,
    pixel_size: usize,
) {
    for y in 0..dst_height {
        for x in 0..dst_width {
            let src_x = x * 2;
            let src_y = y * 2;

            // Sample 4 pixels
            let mut pixel = vec![0u32; pixel_size];

            for dy in 0..2 {
                for dx in 0..2 {
                    let sx = (src_x + dx).min(src_width - 1) as usize;
                    let sy = (src_y + dy).min(src_height - 1) as usize;
                    let src_idx = (sy * src_width as usize + sx) * pixel_size;

                    for c in 0..pixel_size {
                        if src_idx + c < src.len() {
                            pixel[c] += src[src_idx + c] as u32;
                        }
                    }
                }
            }

            // Average and write
            let dst_idx = (y as usize * dst_width as usize + x as usize) * pixel_size;
            for c in 0..pixel_size {
                if dst_idx + c < dst.len() {
                    dst[dst_idx + c] = (pixel[c] / 4) as u8;
                }
            }
        }
    }
}

/// Convert WebGL wrap mode to WGPU
fn webgl_wrap_to_wgpu(wrap: u32) -> wgpu::AddressMode {
    match wrap {
        WebGLConstants::REPEAT => wgpu::AddressMode::Repeat,
        WebGLConstants::CLAMP_TO_EDGE => wgpu::AddressMode::ClampToEdge,
        WebGLConstants::MIRRORED_REPEAT => wgpu::AddressMode::MirrorRepeat,
        _ => wgpu::AddressMode::Repeat,
    }
}

/// Convert WebGL filter to WGPU
fn webgl_filter_to_wgpu(filter: u32) -> wgpu::FilterMode {
    match filter {
        WebGLConstants::NEAREST
        | WebGLConstants::NEAREST_MIPMAP_NEAREST
        | WebGLConstants::NEAREST_MIPMAP_LINEAR => wgpu::FilterMode::Nearest,
        WebGLConstants::LINEAR
        | WebGLConstants::LINEAR_MIPMAP_NEAREST
        | WebGLConstants::LINEAR_MIPMAP_LINEAR => wgpu::FilterMode::Linear,
        _ => wgpu::FilterMode::Linear,
    }
}

/// Convert WebGL mipmap filter to WGPU
fn webgl_mipmap_filter_to_wgpu(filter: u32) -> wgpu::FilterMode {
    match filter {
        WebGLConstants::NEAREST_MIPMAP_NEAREST | WebGLConstants::LINEAR_MIPMAP_NEAREST => {
            wgpu::FilterMode::Nearest
        }
        WebGLConstants::NEAREST_MIPMAP_LINEAR | WebGLConstants::LINEAR_MIPMAP_LINEAR => {
            wgpu::FilterMode::Linear
        }
        _ => wgpu::FilterMode::Nearest,
    }
}
