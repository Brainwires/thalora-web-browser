//! WASM Debug Tools - Standalone MCP tools for WebAssembly module debugging
//!
//! Provides 10 MCP tools for loading, inspecting, disassembling, executing,
//! and profiling WASM modules outside of the JavaScript context.
//!
//! Gated by the `wasm-debug` Cargo feature and `THALORA_ENABLE_WASM_DEBUG` env var.

mod state;
mod validation;
mod inspection;
mod disassembly;
mod memory;
mod execution;
mod profiling;

pub use state::{WasmDebugState, LoadedModule, FuelState, ModuleSummary};
pub use execution::TypedArg;

use anyhow::Result;
use serde_json::Value;

/// Top-level facade for WASM debug MCP tool operations
pub struct WasmDebugTools {
    state: WasmDebugState,
}

impl WasmDebugTools {
    /// Create a new WasmDebugTools instance
    pub fn new() -> Result<Self> {
        Ok(Self {
            state: WasmDebugState::new()?,
        })
    }

    /// Load a WASM module from base64 or WAT text
    pub fn load_module(&mut self, args: Value) -> Result<Value> {
        let module_id = args.get("module_id").and_then(|v| v.as_str());
        let wasm_base64 = args.get("wasm_base64").and_then(|v| v.as_str());
        let wat_text = args.get("wat_text").and_then(|v| v.as_str());
        let name = args.get("name").and_then(|v| v.as_str());
        let instantiate = args.get("instantiate").and_then(|v| v.as_bool()).unwrap_or(true);

        self.state.load_module(module_id, wasm_base64, wat_text, name, instantiate)
    }

    /// Unload a module
    pub fn unload_module(&mut self, args: Value) -> Result<Value> {
        let module_id = args.get("module_id").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: module_id"))?;
        self.state.unload_module(module_id)
    }

    /// List all loaded modules
    pub fn list_modules(&self) -> Value {
        self.state.list_loaded_modules()
    }

    /// Validate WASM without loading
    pub fn validate(&self, args: Value) -> Result<Value> {
        let wasm_base64 = args.get("wasm_base64").and_then(|v| v.as_str());
        let wat_text = args.get("wat_text").and_then(|v| v.as_str());
        self.state.validate_wasm(wasm_base64, wat_text)
    }

    /// Inspect a loaded module's structure
    pub fn inspect(&self, args: Value) -> Result<Value> {
        let module_id = args.get("module_id").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: module_id"))?;
        let sections: Option<Vec<String>> = args.get("sections")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
        let include_signatures = args.get("include_signatures").and_then(|v| v.as_bool()).unwrap_or(true);

        self.state.inspect_module(module_id, sections.as_deref(), include_signatures)
    }

    /// Disassemble a loaded module to WAT
    pub fn disassemble(&self, args: Value) -> Result<Value> {
        let module_id = args.get("module_id").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: module_id"))?;
        let function_index = args.get("function_index").and_then(|v| v.as_u64()).map(|v| v as u32);
        let function_name = args.get("function_name").and_then(|v| v.as_str());
        let fold_expressions = args.get("fold_expressions").and_then(|v| v.as_bool()).unwrap_or(true);

        self.state.disassemble_module(module_id, function_index, function_name, fold_expressions)
    }

    /// Read from a module's linear memory
    pub fn read_memory(&mut self, args: Value) -> Result<Value> {
        let module_id = args.get("module_id").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: module_id"))?;
        let offset = args.get("offset").and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: offset"))?;
        let memory_index = args.get("memory_index").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let length = args.get("length").and_then(|v| v.as_u64()).unwrap_or(256) as usize;
        let format = args.get("format").and_then(|v| v.as_str()).unwrap_or("hex");
        let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

        self.state.read_memory(module_id, offset, memory_index, length, format, count)
    }

    /// Write to a module's linear memory
    pub fn write_memory(&mut self, args: Value) -> Result<Value> {
        let module_id = args.get("module_id").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: module_id"))?;
        let offset = args.get("offset").and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: offset"))?;
        let memory_index = args.get("memory_index").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let hex_data = args.get("hex_data").and_then(|v| v.as_str());
        let base64_data = args.get("base64_data").and_then(|v| v.as_str());
        let typed_values: Option<Vec<Value>> = args.get("typed_values")
            .and_then(|v| v.as_array())
            .map(|arr| arr.clone());
        let typed_format = args.get("typed_format").and_then(|v| v.as_str());
        let utf8_string = args.get("utf8_string").and_then(|v| v.as_str());

        self.state.write_memory(
            module_id,
            offset,
            memory_index,
            hex_data,
            base64_data,
            typed_values.as_deref(),
            typed_format,
            utf8_string,
        )
    }

    /// Call an exported function
    pub async fn call_function(&mut self, args: Value) -> Result<Value> {
        let module_id = args.get("module_id").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: module_id"))?;
        let function_name = args.get("function_name").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: function_name"))?;
        let timeout_ms = args.get("timeout_ms").and_then(|v| v.as_u64());

        let typed_args = parse_typed_args(args.get("args"))?;
        let typed_args_ref = typed_args.as_deref();

        self.state.call_function(module_id, function_name, typed_args_ref, timeout_ms).await
    }

    /// Profile a function's execution
    pub async fn profile_function(&mut self, args: Value) -> Result<Value> {
        let module_id = args.get("module_id").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: module_id"))?;
        let function_name = args.get("function_name").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: function_name"))?;
        let iterations = args.get("iterations").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
        let fuel_limit = args.get("fuel_limit").and_then(|v| v.as_u64());
        let timeout_ms = args.get("timeout_ms").and_then(|v| v.as_u64());

        let typed_args = parse_typed_args(args.get("args"))?;
        let typed_args_ref = typed_args.as_deref();

        self.state.profile_function(
            module_id,
            function_name,
            typed_args_ref,
            iterations,
            fuel_limit,
            timeout_ms,
        ).await
    }
}

/// Parse typed arguments from JSON array
fn parse_typed_args(args_val: Option<&Value>) -> Result<Option<Vec<TypedArg>>> {
    let arr = match args_val {
        Some(Value::Array(arr)) if !arr.is_empty() => arr,
        _ => return Ok(None),
    };

    let mut typed_args = Vec::new();
    for (i, arg) in arr.iter().enumerate() {
        let val_type = arg.get("type").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Argument {} missing 'type' field", i))?
            .to_string();
        let value = arg.get("value")
            .ok_or_else(|| anyhow::anyhow!("Argument {} missing 'value' field", i))?
            .clone();
        typed_args.push(TypedArg { val_type, value });
    }

    Ok(Some(typed_args))
}
