//! WebGL Shader Compilation
//!
//! Handles GLSL shader compilation and linking using naga for GLSL parsing
//! and conversion to WGPU-compatible SPIR-V.

use naga::front::glsl::{Frontend, Options};
use naga::valid::{Capabilities, ValidationFlags, Validator};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

use super::state::WebGLConstants;

static SHADER_ID_COUNTER: AtomicU32 = AtomicU32::new(1);
static PROGRAM_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

/// WebGL Shader object
#[derive(Debug)]
pub struct WebGLShader {
    /// Unique shader ID
    pub id: u32,
    /// Shader type (VERTEX_SHADER or FRAGMENT_SHADER)
    pub shader_type: u32,
    /// Original GLSL source code
    pub source: String,
    /// Compiled naga module (if successful)
    pub module: Option<naga::Module>,
    /// Compilation info log
    pub info_log: String,
    /// Whether compilation succeeded
    pub compiled: bool,
    /// Marked for deletion
    pub delete_pending: bool,
}

impl WebGLShader {
    /// Create a new shader
    pub fn new(shader_type: u32) -> Self {
        Self {
            id: SHADER_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            shader_type,
            source: String::new(),
            module: None,
            info_log: String::new(),
            compiled: false,
            delete_pending: false,
        }
    }

    /// Set shader source
    pub fn set_source(&mut self, source: &str) {
        self.source = source.to_string();
        self.compiled = false;
        self.info_log.clear();
        self.module = None;
    }

    /// Compile the shader
    pub fn compile(&mut self) {
        if self.source.is_empty() {
            self.info_log = "ERROR: No shader source provided".to_string();
            self.compiled = false;
            return;
        }

        let stage = match self.shader_type {
            WebGLConstants::VERTEX_SHADER => naga::ShaderStage::Vertex,
            WebGLConstants::FRAGMENT_SHADER => naga::ShaderStage::Fragment,
            _ => {
                self.info_log = format!("ERROR: Invalid shader type: {}", self.shader_type);
                self.compiled = false;
                return;
            }
        };

        // Preprocess GLSL source for WebGL compatibility
        let processed_source = preprocess_glsl(&self.source, self.shader_type);

        let mut frontend = Frontend::default();
        let options = Options::from(stage);

        match frontend.parse(&options, &processed_source) {
            Ok(module) => {
                // Validate the module
                let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
                match validator.validate(&module) {
                    Ok(_info) => {
                        self.module = Some(module);
                        self.compiled = true;
                        self.info_log = String::new();
                    }
                    Err(err) => {
                        self.info_log = format!("ERROR: Shader validation failed: {:?}", err);
                        self.compiled = false;
                    }
                }
            }
            Err(errors) => {
                let mut error_msg = String::new();
                for err in errors.errors.iter() {
                    error_msg.push_str(&format!("ERROR: {:?}\n", err));
                }
                self.info_log = error_msg;
                self.compiled = false;
            }
        }
    }

    /// Get shader parameter
    pub fn get_parameter(&self, pname: u32) -> Option<ShaderParameter> {
        match pname {
            WebGLConstants::SHADER_TYPE => Some(ShaderParameter::Int(self.shader_type as i32)),
            WebGLConstants::DELETE_STATUS => {
                Some(ShaderParameter::Bool(self.delete_pending))
            }
            WebGLConstants::COMPILE_STATUS => Some(ShaderParameter::Bool(self.compiled)),
            _ => None,
        }
    }

    /// Get info log
    pub fn get_info_log(&self) -> &str {
        &self.info_log
    }

    /// Get source
    pub fn get_source(&self) -> &str {
        &self.source
    }
}

/// Preprocess GLSL source for WebGL compatibility with naga
fn preprocess_glsl(source: &str, shader_type: u32) -> String {
    let mut result = String::new();

    // Check if source already has a version directive
    let has_version = source.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("#version")
    });

    // Add version directive if not present (naga requires it)
    if !has_version {
        result.push_str("#version 300 es\n");
    }

    // Add precision qualifier for fragment shaders if not present
    if shader_type == WebGLConstants::FRAGMENT_SHADER {
        let has_precision = source.contains("precision ");
        if !has_precision {
            result.push_str("precision highp float;\n");
        }
    }

    result.push_str(source);

    // Convert WebGL 1.0 GLSL to GLSL ES 300 syntax if needed
    let result = convert_webgl1_to_es300(&result, shader_type);

    result
}

/// Convert WebGL 1.0 GLSL syntax to GLSL ES 300
fn convert_webgl1_to_es300(source: &str, shader_type: u32) -> String {
    let mut result = source.to_string();

    // Replace attribute with in (vertex shader only)
    if shader_type == WebGLConstants::VERTEX_SHADER {
        result = result.replace("attribute ", "in ");
    }

    // Replace varying with in/out
    if shader_type == WebGLConstants::VERTEX_SHADER {
        result = result.replace("varying ", "out ");
    } else if shader_type == WebGLConstants::FRAGMENT_SHADER {
        result = result.replace("varying ", "in ");
    }

    // Replace texture2D with texture
    result = result.replace("texture2D(", "texture(");
    result = result.replace("textureCube(", "texture(");

    // Replace gl_FragColor with out variable if not already done
    if shader_type == WebGLConstants::FRAGMENT_SHADER && result.contains("gl_FragColor") {
        // Add output variable declaration if gl_FragColor is used
        if !result.contains("out vec4 fragColor") && !result.contains("layout(location = 0) out") {
            // Find the position after precision declarations
            let insert_pos = result
                .find("void main")
                .unwrap_or_else(|| result.len());
            result.insert_str(insert_pos, "out vec4 fragColor;\n");
        }
        result = result.replace("gl_FragColor", "fragColor");
    }

    result
}

/// Shader parameter value
#[derive(Debug, Clone)]
pub enum ShaderParameter {
    Int(i32),
    Bool(bool),
}

/// Uniform location information
#[derive(Debug, Clone)]
pub struct UniformLocation {
    /// Unique location ID
    pub id: u32,
    /// Uniform name
    pub name: String,
    /// Binding index in the shader
    pub binding: u32,
    /// Uniform type
    pub uniform_type: u32,
    /// Array size (1 for non-arrays)
    pub size: i32,
}

/// Attribute location information
#[derive(Debug, Clone)]
pub struct AttributeInfo {
    /// Location index
    pub location: u32,
    /// Attribute name
    pub name: String,
    /// Attribute type
    pub attr_type: u32,
    /// Size
    pub size: i32,
}

/// WebGL Program object
#[derive(Debug)]
pub struct WebGLProgram {
    /// Unique program ID
    pub id: u32,
    /// Attached vertex shader ID
    pub vertex_shader: Option<u32>,
    /// Attached fragment shader ID
    pub fragment_shader: Option<u32>,
    /// Whether the program has been linked
    pub linked: bool,
    /// Link info log
    pub info_log: String,
    /// Uniform locations
    pub uniforms: HashMap<String, UniformLocation>,
    /// Attribute locations
    pub attributes: HashMap<String, AttributeInfo>,
    /// Combined naga module (after linking)
    pub module: Option<naga::Module>,
    /// WGPU shader module (compiled)
    pub wgpu_module: Option<WgpuShaderModule>,
    /// Marked for deletion
    pub delete_pending: bool,
    /// Validated
    pub validated: bool,
}

/// Wrapper for WGPU shader module data
#[derive(Debug)]
pub struct WgpuShaderModule {
    /// SPIR-V bytecode for vertex shader
    pub vertex_spirv: Vec<u32>,
    /// SPIR-V bytecode for fragment shader
    pub fragment_spirv: Vec<u32>,
}

impl WebGLProgram {
    /// Create a new program
    pub fn new() -> Self {
        Self {
            id: PROGRAM_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            vertex_shader: None,
            fragment_shader: None,
            linked: false,
            info_log: String::new(),
            uniforms: HashMap::new(),
            attributes: HashMap::new(),
            module: None,
            wgpu_module: None,
            delete_pending: false,
            validated: false,
        }
    }

    /// Attach a shader to the program
    pub fn attach_shader(&mut self, shader: &WebGLShader) {
        match shader.shader_type {
            WebGLConstants::VERTEX_SHADER => {
                self.vertex_shader = Some(shader.id);
            }
            WebGLConstants::FRAGMENT_SHADER => {
                self.fragment_shader = Some(shader.id);
            }
            _ => {}
        }
        self.linked = false;
    }

    /// Detach a shader from the program
    pub fn detach_shader(&mut self, shader: &WebGLShader) {
        match shader.shader_type {
            WebGLConstants::VERTEX_SHADER => {
                if self.vertex_shader == Some(shader.id) {
                    self.vertex_shader = None;
                }
            }
            WebGLConstants::FRAGMENT_SHADER => {
                if self.fragment_shader == Some(shader.id) {
                    self.fragment_shader = None;
                }
            }
            _ => {}
        }
        self.linked = false;
    }

    /// Link the program
    pub fn link(&mut self, shaders: &HashMap<u32, WebGLShader>) {
        self.info_log.clear();
        self.uniforms.clear();
        self.attributes.clear();

        // Check that both shaders are attached
        let vertex_id = match self.vertex_shader {
            Some(id) => id,
            None => {
                self.info_log = "ERROR: No vertex shader attached".to_string();
                self.linked = false;
                return;
            }
        };

        let fragment_id = match self.fragment_shader {
            Some(id) => id,
            None => {
                self.info_log = "ERROR: No fragment shader attached".to_string();
                self.linked = false;
                return;
            }
        };

        // Get shaders
        let vertex_shader = match shaders.get(&vertex_id) {
            Some(s) => s,
            None => {
                self.info_log = "ERROR: Vertex shader not found".to_string();
                self.linked = false;
                return;
            }
        };

        let fragment_shader = match shaders.get(&fragment_id) {
            Some(s) => s,
            None => {
                self.info_log = "ERROR: Fragment shader not found".to_string();
                self.linked = false;
                return;
            }
        };

        // Check compilation status
        if !vertex_shader.compiled {
            self.info_log = "ERROR: Vertex shader not compiled".to_string();
            self.linked = false;
            return;
        }

        if !fragment_shader.compiled {
            self.info_log = "ERROR: Fragment shader not compiled".to_string();
            self.linked = false;
            return;
        }

        // Get modules
        let vertex_module = match &vertex_shader.module {
            Some(m) => m,
            None => {
                self.info_log = "ERROR: Vertex shader module missing".to_string();
                self.linked = false;
                return;
            }
        };

        let fragment_module = match &fragment_shader.module {
            Some(m) => m,
            None => {
                self.info_log = "ERROR: Fragment shader module missing".to_string();
                self.linked = false;
                return;
            }
        };

        // Extract attributes from vertex shader - use entry points for input bindings
        let mut attr_location = 0u32;
        for entry in &vertex_module.entry_points {
            for arg in &entry.function.arguments {
                if let Some(binding) = &arg.binding {
                    if let Some(name) = &arg.name {
                        let type_info = &vertex_module.types[arg.ty];
                        let attr_type = naga_type_to_webgl(&type_info.inner);
                        let size = get_type_size(&type_info.inner);

                        // Get location from binding
                        let location = match binding {
                            naga::Binding::Location { location, .. } => *location,
                            _ => attr_location,
                        };

                        self.attributes.insert(
                            name.clone(),
                            AttributeInfo {
                                location,
                                name: name.clone(),
                                attr_type,
                                size,
                            },
                        );
                        attr_location = attr_location.max(location + 1);
                    }
                }
            }
        }

        // Extract uniforms
        let mut uniform_binding = 0u32;
        for (_handle, var) in vertex_module.global_variables.iter() {
            if var.space == naga::AddressSpace::Uniform {
                if let Some(name) = &var.name {
                    let type_info = &vertex_module.types[var.ty];
                    let uniform_type = naga_type_to_webgl(&type_info.inner);
                    let size = get_type_size(&type_info.inner);

                    self.uniforms.insert(
                        name.clone(),
                        UniformLocation {
                            id: uniform_binding,
                            name: name.clone(),
                            binding: uniform_binding,
                            uniform_type,
                            size,
                        },
                    );
                    uniform_binding += 1;
                }
            }
        }

        // Also check fragment shader for uniforms
        for (_handle, var) in fragment_module.global_variables.iter() {
            if var.space == naga::AddressSpace::Uniform {
                if let Some(name) = &var.name {
                    if !self.uniforms.contains_key(name) {
                        let type_info = &fragment_module.types[var.ty];
                        let uniform_type = naga_type_to_webgl(&type_info.inner);
                        let size = get_type_size(&type_info.inner);

                        self.uniforms.insert(
                            name.clone(),
                            UniformLocation {
                                id: uniform_binding,
                                name: name.clone(),
                                binding: uniform_binding,
                                uniform_type,
                                size,
                            },
                        );
                        uniform_binding += 1;
                    }
                }
            }
        }

        // Convert to SPIR-V
        let vertex_spirv = match module_to_spirv(vertex_module, naga::ShaderStage::Vertex) {
            Ok(spirv) => spirv,
            Err(err) => {
                self.info_log = format!("ERROR: Failed to convert vertex shader to SPIR-V: {}", err);
                self.linked = false;
                return;
            }
        };

        let fragment_spirv = match module_to_spirv(fragment_module, naga::ShaderStage::Fragment) {
            Ok(spirv) => spirv,
            Err(err) => {
                self.info_log = format!("ERROR: Failed to convert fragment shader to SPIR-V: {}", err);
                self.linked = false;
                return;
            }
        };

        self.wgpu_module = Some(WgpuShaderModule {
            vertex_spirv,
            fragment_spirv,
        });

        self.linked = true;
        self.info_log = String::new();
    }

    /// Get program parameter
    pub fn get_parameter(&self, pname: u32) -> Option<ProgramParameter> {
        match pname {
            WebGLConstants::DELETE_STATUS => Some(ProgramParameter::Bool(self.delete_pending)),
            WebGLConstants::LINK_STATUS => Some(ProgramParameter::Bool(self.linked)),
            WebGLConstants::VALIDATE_STATUS => Some(ProgramParameter::Bool(self.validated)),
            WebGLConstants::ATTACHED_SHADERS => {
                let count = (if self.vertex_shader.is_some() { 1 } else { 0 })
                    + (if self.fragment_shader.is_some() { 1 } else { 0 });
                Some(ProgramParameter::Int(count))
            }
            WebGLConstants::ACTIVE_ATTRIBUTES => {
                Some(ProgramParameter::Int(self.attributes.len() as i32))
            }
            WebGLConstants::ACTIVE_UNIFORMS => {
                Some(ProgramParameter::Int(self.uniforms.len() as i32))
            }
            _ => None,
        }
    }

    /// Get info log
    pub fn get_info_log(&self) -> &str {
        &self.info_log
    }

    /// Get uniform location
    pub fn get_uniform_location(&self, name: &str) -> Option<&UniformLocation> {
        self.uniforms.get(name)
    }

    /// Get attribute location
    pub fn get_attrib_location(&self, name: &str) -> Option<i32> {
        self.attributes.get(name).map(|a| a.location as i32)
    }

    /// Get active uniform info
    pub fn get_active_uniform(&self, index: u32) -> Option<(&String, &UniformLocation)> {
        self.uniforms.iter().nth(index as usize)
    }

    /// Get active attribute info
    pub fn get_active_attrib(&self, index: u32) -> Option<(&String, &AttributeInfo)> {
        self.attributes.iter().nth(index as usize)
    }

    /// Validate the program
    pub fn validate(&mut self) {
        if self.linked {
            self.validated = true;
            self.info_log = String::new();
        } else {
            self.validated = false;
            self.info_log = "ERROR: Program not linked".to_string();
        }
    }
}

impl Default for WebGLProgram {
    fn default() -> Self {
        Self::new()
    }
}

/// Program parameter value
#[derive(Debug, Clone)]
pub enum ProgramParameter {
    Int(i32),
    Bool(bool),
}

/// Convert naga type to WebGL type constant
fn naga_type_to_webgl(ty: &naga::TypeInner) -> u32 {
    match ty {
        naga::TypeInner::Scalar(scalar) => match scalar.kind {
            naga::ScalarKind::Float => WebGLConstants::FLOAT,
            naga::ScalarKind::Sint => WebGLConstants::INT,
            naga::ScalarKind::Uint => WebGLConstants::UNSIGNED_INT,
            naga::ScalarKind::Bool => WebGLConstants::BOOL,
            naga::ScalarKind::AbstractInt | naga::ScalarKind::AbstractFloat => WebGLConstants::FLOAT,
        },
        naga::TypeInner::Vector { size, scalar } => {
            let base = match scalar.kind {
                naga::ScalarKind::Float => match size {
                    naga::VectorSize::Bi => WebGLConstants::FLOAT_VEC2,
                    naga::VectorSize::Tri => WebGLConstants::FLOAT_VEC3,
                    naga::VectorSize::Quad => WebGLConstants::FLOAT_VEC4,
                },
                naga::ScalarKind::Sint => match size {
                    naga::VectorSize::Bi => WebGLConstants::INT_VEC2,
                    naga::VectorSize::Tri => WebGLConstants::INT_VEC3,
                    naga::VectorSize::Quad => WebGLConstants::INT_VEC4,
                },
                naga::ScalarKind::Bool => match size {
                    naga::VectorSize::Bi => WebGLConstants::BOOL_VEC2,
                    naga::VectorSize::Tri => WebGLConstants::BOOL_VEC3,
                    naga::VectorSize::Quad => WebGLConstants::BOOL_VEC4,
                },
                _ => WebGLConstants::FLOAT_VEC4,
            };
            base
        }
        naga::TypeInner::Matrix { columns, rows, .. } => {
            match (columns, rows) {
                (naga::VectorSize::Bi, naga::VectorSize::Bi) => WebGLConstants::FLOAT_MAT2,
                (naga::VectorSize::Tri, naga::VectorSize::Tri) => WebGLConstants::FLOAT_MAT3,
                (naga::VectorSize::Quad, naga::VectorSize::Quad) => WebGLConstants::FLOAT_MAT4,
                _ => WebGLConstants::FLOAT_MAT4,
            }
        }
        naga::TypeInner::Image { .. } => WebGLConstants::SAMPLER_2D,
        naga::TypeInner::Sampler { .. } => WebGLConstants::SAMPLER_2D,
        _ => WebGLConstants::FLOAT,
    }
}

/// Get size of a type (for arrays)
fn get_type_size(ty: &naga::TypeInner) -> i32 {
    match ty {
        naga::TypeInner::Array { size, .. } => match size {
            naga::ArraySize::Constant(size) => size.get() as i32,
            naga::ArraySize::Dynamic => 0,
        },
        _ => 1,
    }
}

/// Convert naga module to SPIR-V
fn module_to_spirv(module: &naga::Module, _stage: naga::ShaderStage) -> Result<Vec<u32>, String> {
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    let info = validator
        .validate(module)
        .map_err(|e| format!("Validation error: {:?}", e))?;

    let options = naga::back::spv::Options {
        flags: naga::back::spv::WriterFlags::empty(),
        ..Default::default()
    };

    let mut output = Vec::new();
    let mut writer = naga::back::spv::Writer::new(&options).map_err(|e| format!("Writer creation error: {:?}", e))?;

    writer
        .write(module, &info, None, &Default::default(), &mut output)
        .map_err(|e| format!("SPIR-V write error: {:?}", e))?;

    Ok(output)
}
