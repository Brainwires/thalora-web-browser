use anyhow::{Result, anyhow};
use serde_json::Value;
use wasmtime::{Val, ValType, Func};

use super::state::{WasmDebugState, validate_module_id, DEFAULT_FUEL};

/// Maximum timeout for function execution (30 seconds)
const MAX_TIMEOUT_MS: u64 = 30_000;

/// Default timeout for function execution (5 seconds)
const DEFAULT_TIMEOUT_MS: u64 = 5_000;

/// Argument descriptor for calling a WASM function
#[derive(Debug)]
pub struct TypedArg {
    pub val_type: String,
    pub value: Value,
}

impl WasmDebugState {
    /// Call an exported WASM function
    pub async fn call_function(
        &mut self,
        module_id: &str,
        function_name: &str,
        args: Option<&[TypedArg]>,
        timeout_ms: Option<u64>,
    ) -> Result<Value> {
        validate_module_id(module_id)?;

        let timeout = timeout_ms
            .unwrap_or(DEFAULT_TIMEOUT_MS)
            .min(MAX_TIMEOUT_MS);

        // Get the module and find the function
        let loaded = self.get_module_mut(module_id)
            .ok_or_else(|| anyhow!("Module '{}' not found", module_id))?;

        let instance = loaded.instance
            .ok_or_else(|| anyhow!("Module '{}' is not instantiated", module_id))?;

        // Find the exported function
        let func = instance.get_func(&mut loaded.store, function_name)
            .ok_or_else(|| anyhow!("Function '{}' not found in module exports", function_name))?;

        let func_ty = func.ty(&loaded.store);

        // Validate and convert arguments
        let param_types: Vec<ValType> = func_ty.params().collect();
        let result_types: Vec<ValType> = func_ty.results().collect();

        let args_slice = args.unwrap_or(&[]);
        if args_slice.len() != param_types.len() {
            return Err(anyhow!(
                "Function '{}' expects {} arguments, got {}",
                function_name,
                param_types.len(),
                args_slice.len()
            ));
        }

        // Convert arguments to wasmtime Val types
        let mut wasm_args = Vec::new();
        for (i, (arg, expected_type)) in args_slice.iter().zip(param_types.iter()).enumerate() {
            let val = convert_to_wasm_val(&arg.value, &arg.val_type, expected_type, i)?;
            wasm_args.push(val);
        }

        // Prepare result slots
        let mut wasm_results: Vec<Val> = result_types.iter()
            .map(|t| match t {
                ValType::I32 => Val::I32(0),
                ValType::I64 => Val::I64(0),
                ValType::F32 => Val::F32(0),
                ValType::F64 => Val::F64(0),
                _ => Val::I32(0),
            })
            .collect();

        // Refuel the store before execution
        loaded.store.set_fuel(DEFAULT_FUEL).unwrap_or(());

        // Get fuel before execution
        let fuel_before = loaded.store.get_fuel().unwrap_or(0);

        // Set up epoch-based timeout
        let engine = loaded.store.engine().clone();

        // Spawn a background task to increment the epoch after timeout
        let timeout_handle = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(timeout)).await;
            engine.increment_epoch();
        });

        // Execute the function
        let start_time = std::time::Instant::now();
        let call_result = func.call(&mut loaded.store, &wasm_args, &mut wasm_results);
        let elapsed = start_time.elapsed();

        // Cancel the timeout task
        timeout_handle.abort();

        // Get fuel after execution
        let fuel_after = loaded.store.get_fuel().unwrap_or(0);
        let fuel_consumed = fuel_before.saturating_sub(fuel_after);

        match call_result {
            Ok(()) => {
                // Convert results to JSON
                let results: Vec<Value> = wasm_results.iter()
                    .map(|v| wasm_val_to_json(v))
                    .collect();

                let result_types_str: Vec<String> = result_types.iter()
                    .map(|t| format!("{:?}", t))
                    .collect();

                Ok(serde_json::json!({
                    "module_id": module_id,
                    "function": function_name,
                    "results": results,
                    "result_types": result_types_str,
                    "execution_time_ms": elapsed.as_secs_f64() * 1000.0,
                    "fuel_consumed": fuel_consumed,
                    "status": "success"
                }))
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                let is_timeout = error_msg.contains("epoch")
                    || error_msg.contains("interrupt")
                    || elapsed.as_millis() >= timeout as u128;

                if is_timeout {
                    Ok(serde_json::json!({
                        "module_id": module_id,
                        "function": function_name,
                        "status": "timeout",
                        "error": format!("Function execution timed out after {}ms", timeout),
                        "execution_time_ms": elapsed.as_secs_f64() * 1000.0,
                        "fuel_consumed": fuel_consumed,
                    }))
                } else {
                    Err(anyhow!("Function call failed: {}", e))
                }
            }
        }
    }
}

/// Convert a JSON value + type hint to a wasmtime Val
fn convert_to_wasm_val(value: &Value, type_hint: &str, expected: &ValType, arg_index: usize) -> Result<Val> {
    match expected {
        ValType::I32 => {
            let v = value.as_i64()
                .ok_or_else(|| anyhow!("Argument {} ({}): expected integer, got {:?}", arg_index, type_hint, value))?;
            Ok(Val::I32(v as i32))
        }
        ValType::I64 => {
            let v = value.as_i64()
                .ok_or_else(|| anyhow!("Argument {} ({}): expected integer, got {:?}", arg_index, type_hint, value))?;
            Ok(Val::I64(v))
        }
        ValType::F32 => {
            let v = value.as_f64()
                .ok_or_else(|| anyhow!("Argument {} ({}): expected float, got {:?}", arg_index, type_hint, value))?;
            Ok(Val::F32((v as f32).to_bits()))
        }
        ValType::F64 => {
            let v = value.as_f64()
                .ok_or_else(|| anyhow!("Argument {} ({}): expected float, got {:?}", arg_index, type_hint, value))?;
            Ok(Val::F64(v.to_bits()))
        }
        _ => Err(anyhow!(
            "Argument {} ({}): unsupported parameter type {:?}",
            arg_index, type_hint, expected
        )),
    }
}

/// Convert a wasmtime Val to a JSON value
fn wasm_val_to_json(val: &Val) -> Value {
    match val {
        Val::I32(v) => serde_json::json!(v),
        Val::I64(v) => serde_json::json!(v),
        Val::F32(bits) => serde_json::json!(f32::from_bits(*bits)),
        Val::F64(bits) => serde_json::json!(f64::from_bits(*bits)),
        _ => serde_json::json!(format!("{:?}", val)),
    }
}
