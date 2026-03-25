use anyhow::{Result, anyhow};
use base64::Engine as Base64Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde_json::Value;
use std::time::Instant;
use wasmtime::{Instance, Linker, Module};

use super::state::{
    DEFAULT_FUEL, FuelState, LoadedModule, MAX_WASM_BINARY_SIZE, WasmDebugState, validate_module_id,
};

impl WasmDebugState {
    /// Load a WASM module from base64 binary or WAT text
    pub fn load_module(
        &mut self,
        module_id: Option<&str>,
        wasm_base64: Option<&str>,
        wat_text: Option<&str>,
        name: Option<&str>,
        instantiate: bool,
    ) -> Result<Value> {
        // Parse the binary from either base64 or WAT
        let binary = if let Some(b64) = wasm_base64 {
            let bytes = BASE64
                .decode(b64)
                .map_err(|e| anyhow!("Invalid base64: {}", e))?;
            if bytes.len() > MAX_WASM_BINARY_SIZE {
                return Err(anyhow!(
                    "WASM binary too large: {} bytes (max {})",
                    bytes.len(),
                    MAX_WASM_BINARY_SIZE
                ));
            }
            bytes
        } else if let Some(wat) = wat_text {
            let bytes = wat::parse_str(wat).map_err(|e| anyhow!("Failed to parse WAT: {}", e))?;
            if bytes.len() > MAX_WASM_BINARY_SIZE {
                return Err(anyhow!(
                    "Compiled WASM binary too large: {} bytes (max {})",
                    bytes.len(),
                    MAX_WASM_BINARY_SIZE
                ));
            }
            bytes
        } else {
            return Err(anyhow!("Either wasm_base64 or wat_text must be provided"));
        };

        // Generate or validate module ID
        let id = if let Some(mid) = module_id {
            validate_module_id(mid)?;
            if self.has_module(mid) {
                return Err(anyhow!(
                    "Module '{}' is already loaded. Unload it first.",
                    mid
                ));
            }
            mid.to_string()
        } else {
            // Auto-generate ID
            let id = format!(
                "module_{}",
                uuid::Uuid::new_v4().to_string().replace('-', "")[..12].to_string()
            );
            id
        };

        let module_name = name.unwrap_or(&id).to_string();

        // Compile the module
        let module = Module::new(self.engine(), &binary)
            .map_err(|e| anyhow!("Failed to compile WASM module: {}", e))?;

        // Create a store with fuel
        let mut store = self.new_store();

        // Optionally instantiate
        let instance = if instantiate {
            // Check if module has imports - if so, we can only auto-instantiate if there are none
            let imports = module.imports().collect::<Vec<_>>();
            if imports.is_empty() {
                let linker = Linker::new(self.engine());
                match linker.instantiate(&mut store, &module) {
                    Ok(inst) => Some(inst),
                    Err(e) => {
                        return Err(anyhow!("Failed to instantiate module: {}", e));
                    }
                }
            } else {
                // Module has imports - cannot auto-instantiate
                None
            }
        } else {
            None
        };

        let is_instantiated = instance.is_some();
        let binary_size = binary.len();

        let loaded = LoadedModule {
            module,
            instance,
            store,
            binary,
            name: module_name.clone(),
            loaded_at: Instant::now(),
        };

        self.insert_module(id.clone(), loaded)?;

        Ok(serde_json::json!({
            "module_id": id,
            "name": module_name,
            "binary_size": binary_size,
            "instantiated": is_instantiated,
            "status": "loaded"
        }))
    }

    /// Unload a module and free its resources
    pub fn unload_module(&mut self, module_id: &str) -> Result<Value> {
        validate_module_id(module_id)?;
        match self.remove_module(module_id) {
            Some(m) => Ok(serde_json::json!({
                "module_id": module_id,
                "name": m.name,
                "status": "unloaded"
            })),
            None => Err(anyhow!("Module '{}' not found", module_id)),
        }
    }

    /// List all loaded modules
    pub fn list_loaded_modules(&self) -> Value {
        let modules = self.list_modules();
        serde_json::json!({
            "count": modules.len(),
            "modules": modules
        })
    }

    /// Validate WASM without loading it into the module store
    pub fn validate_wasm(
        &self,
        wasm_base64: Option<&str>,
        wat_text: Option<&str>,
    ) -> Result<Value> {
        let binary = if let Some(b64) = wasm_base64 {
            let bytes = BASE64
                .decode(b64)
                .map_err(|e| anyhow!("Invalid base64: {}", e))?;
            if bytes.len() > MAX_WASM_BINARY_SIZE {
                return Err(anyhow!(
                    "WASM binary too large: {} bytes (max {})",
                    bytes.len(),
                    MAX_WASM_BINARY_SIZE
                ));
            }
            bytes
        } else if let Some(wat) = wat_text {
            match wat::parse_str(wat) {
                Ok(bytes) => {
                    if bytes.len() > MAX_WASM_BINARY_SIZE {
                        return Err(anyhow!(
                            "Compiled WASM binary too large: {} bytes (max {})",
                            bytes.len(),
                            MAX_WASM_BINARY_SIZE
                        ));
                    }
                    bytes
                }
                Err(e) => {
                    return Ok(serde_json::json!({
                        "valid": false,
                        "error": format!("WAT parse error: {}", e),
                        "source": "wat"
                    }));
                }
            }
        } else {
            return Err(anyhow!("Either wasm_base64 or wat_text must be provided"));
        };

        // Try to compile with wasmtime for full validation
        match Module::new(self.engine(), &binary) {
            Ok(module) => {
                // Gather summary info using wasmparser
                let imports_count = module.imports().count();
                let exports_count = module.exports().count();

                Ok(serde_json::json!({
                    "valid": true,
                    "binary_size": binary.len(),
                    "imports_count": imports_count,
                    "exports_count": exports_count,
                }))
            }
            Err(e) => Ok(serde_json::json!({
                "valid": false,
                "error": format!("Validation error: {}", e),
                "binary_size": binary.len()
            })),
        }
    }
}
