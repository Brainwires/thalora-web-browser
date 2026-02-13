use anyhow::{Result, anyhow};
use serde_json::Value;
use wasmtime::{Val, ValType};

use super::state::{WasmDebugState, validate_module_id, DEFAULT_FUEL, MAX_FUEL};
use super::execution::TypedArg;

/// Maximum number of profiling iterations
const MAX_ITERATIONS: u32 = 10_000;

/// Default profiling timeout (30 seconds)
const DEFAULT_PROFILE_TIMEOUT_MS: u64 = 30_000;

impl WasmDebugState {
    /// Profile a function's execution over N iterations
    pub async fn profile_function(
        &mut self,
        module_id: &str,
        function_name: &str,
        args: Option<&[TypedArg]>,
        iterations: u32,
        fuel_limit: Option<u64>,
        timeout_ms: Option<u64>,
    ) -> Result<Value> {
        validate_module_id(module_id)?;

        let iterations = iterations.min(MAX_ITERATIONS).max(1);
        let fuel_per_call = fuel_limit.unwrap_or(DEFAULT_FUEL).min(MAX_FUEL);
        let timeout = timeout_ms.unwrap_or(DEFAULT_PROFILE_TIMEOUT_MS);

        let loaded = self.get_module_mut(module_id)
            .ok_or_else(|| anyhow!("Module '{}' not found", module_id))?;

        let instance = loaded.instance
            .ok_or_else(|| anyhow!("Module '{}' is not instantiated", module_id))?;

        // Find the exported function
        let func = instance.get_func(&mut loaded.store, function_name)
            .ok_or_else(|| anyhow!("Function '{}' not found in module exports", function_name))?;

        let func_ty = func.ty(&loaded.store);
        let param_types: Vec<ValType> = func_ty.params().collect();
        let result_types: Vec<ValType> = func_ty.results().collect();

        // Convert arguments
        let args_slice = args.unwrap_or(&[]);
        if args_slice.len() != param_types.len() {
            return Err(anyhow!(
                "Function '{}' expects {} arguments, got {}",
                function_name,
                param_types.len(),
                args_slice.len()
            ));
        }

        let mut wasm_args = Vec::new();
        for (i, (arg, expected_type)) in args_slice.iter().zip(param_types.iter()).enumerate() {
            let val = convert_to_wasm_val_for_profile(&arg.value, expected_type, i)?;
            wasm_args.push(val);
        }

        // Get memory usage before profiling
        let memory_pages_before = get_memory_pages(&instance, &mut loaded.store);

        // Set up epoch-based timeout
        let engine = loaded.store.engine().clone();
        let timeout_handle = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(timeout)).await;
            engine.increment_epoch();
        });

        // Run iterations
        let mut timings_ns = Vec::with_capacity(iterations as usize);
        let mut fuel_per_iteration = Vec::with_capacity(iterations as usize);
        let mut completed_iterations = 0u32;
        let mut last_results: Vec<Val> = Vec::new();
        let overall_start = std::time::Instant::now();

        for _ in 0..iterations {
            // Refuel
            loaded.store.set_fuel(fuel_per_call).unwrap_or(());
            let fuel_before = loaded.store.get_fuel().unwrap_or(0);

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

            let iter_start = std::time::Instant::now();
            let result = func.call(&mut loaded.store, &wasm_args, &mut wasm_results);
            let iter_elapsed = iter_start.elapsed();

            let fuel_after = loaded.store.get_fuel().unwrap_or(0);
            let fuel_used = fuel_before.saturating_sub(fuel_after);

            match result {
                Ok(()) => {
                    timings_ns.push(iter_elapsed.as_nanos() as f64);
                    fuel_per_iteration.push(fuel_used);
                    completed_iterations += 1;
                    last_results = wasm_results;
                }
                Err(e) => {
                    let error_msg = format!("{}", e);
                    let is_timeout = error_msg.contains("epoch") || error_msg.contains("interrupt");

                    // Cancel timeout handle
                    timeout_handle.abort();

                    if is_timeout {
                        return Ok(build_profile_result(
                            module_id,
                            function_name,
                            completed_iterations,
                            iterations,
                            &timings_ns,
                            &fuel_per_iteration,
                            memory_pages_before,
                            get_memory_pages(&instance, &mut loaded.store),
                            Some("timeout"),
                            &last_results,
                        ));
                    } else {
                        return Err(anyhow!("Profiling failed at iteration {}: {}", completed_iterations, e));
                    }
                }
            }

            // Check overall timeout
            if overall_start.elapsed().as_millis() >= timeout as u128 {
                timeout_handle.abort();
                return Ok(build_profile_result(
                    module_id,
                    function_name,
                    completed_iterations,
                    iterations,
                    &timings_ns,
                    &fuel_per_iteration,
                    memory_pages_before,
                    get_memory_pages(&instance, &mut loaded.store),
                    Some("timeout"),
                    &last_results,
                ));
            }
        }

        // Cancel timeout
        timeout_handle.abort();

        let memory_pages_after = get_memory_pages(&instance, &mut loaded.store);

        Ok(build_profile_result(
            module_id,
            function_name,
            completed_iterations,
            iterations,
            &timings_ns,
            &fuel_per_iteration,
            memory_pages_before,
            memory_pages_after,
            None,
            &last_results,
        ))
    }
}

/// Build the profiling result JSON
fn build_profile_result(
    module_id: &str,
    function_name: &str,
    completed: u32,
    requested: u32,
    timings_ns: &[f64],
    fuel_per_iter: &[u64],
    mem_pages_before: Option<u64>,
    mem_pages_after: Option<u64>,
    early_stop_reason: Option<&str>,
    last_results: &[Val],
) -> Value {
    let timing_stats = compute_stats(timings_ns);
    let fuel_stats = compute_stats_u64(fuel_per_iter);

    let last_result_values: Vec<Value> = last_results.iter()
        .map(|v| match v {
            Val::I32(x) => serde_json::json!(x),
            Val::I64(x) => serde_json::json!(x),
            Val::F32(bits) => serde_json::json!(f32::from_bits(*bits)),
            Val::F64(bits) => serde_json::json!(f64::from_bits(*bits)),
            _ => serde_json::json!(format!("{:?}", v)),
        })
        .collect();

    let mut result = serde_json::json!({
        "module_id": module_id,
        "function": function_name,
        "iterations_completed": completed,
        "iterations_requested": requested,
        "timing": {
            "total_ms": timing_stats.sum / 1_000_000.0,
            "avg_ms": timing_stats.avg / 1_000_000.0,
            "min_ms": timing_stats.min / 1_000_000.0,
            "max_ms": timing_stats.max / 1_000_000.0,
            "stddev_ms": timing_stats.stddev / 1_000_000.0,
        },
        "fuel": {
            "total": fuel_stats.sum as u64,
            "avg": fuel_stats.avg as u64,
            "min": fuel_stats.min as u64,
            "max": fuel_stats.max as u64,
        },
        "memory": {
            "pages_before": mem_pages_before,
            "pages_after": mem_pages_after,
        },
        "last_results": last_result_values,
        "status": if early_stop_reason.is_some() { "partial" } else { "complete" },
    });

    if let Some(reason) = early_stop_reason {
        result["early_stop_reason"] = serde_json::json!(reason);
    }

    result
}

struct Stats {
    sum: f64,
    avg: f64,
    min: f64,
    max: f64,
    stddev: f64,
}

fn compute_stats(values: &[f64]) -> Stats {
    if values.is_empty() {
        return Stats { sum: 0.0, avg: 0.0, min: 0.0, max: 0.0, stddev: 0.0 };
    }

    let sum: f64 = values.iter().sum();
    let avg = sum / values.len() as f64;
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let variance = if values.len() > 1 {
        values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / (values.len() - 1) as f64
    } else {
        0.0
    };
    let stddev = variance.sqrt();

    Stats { sum, avg, min, max, stddev }
}

fn compute_stats_u64(values: &[u64]) -> Stats {
    let floats: Vec<f64> = values.iter().map(|&v| v as f64).collect();
    compute_stats(&floats)
}

/// Get memory pages for an instance
fn get_memory_pages(instance: &wasmtime::Instance, store: &mut wasmtime::Store<super::state::FuelState>) -> Option<u64> {
    // Collect memory exports first to avoid borrow conflict
    let memories: Vec<wasmtime::Memory> = instance.exports(&mut *store)
        .filter_map(|export| export.into_memory())
        .collect();

    memories.first().map(|memory| memory.size(&*store))
}

/// Convert a JSON value to a wasmtime Val for profiling
fn convert_to_wasm_val_for_profile(value: &Value, expected: &ValType, arg_index: usize) -> Result<Val> {
    match expected {
        ValType::I32 => {
            let v = value.as_i64()
                .ok_or_else(|| anyhow!("Arg {}: expected integer", arg_index))?;
            Ok(Val::I32(v as i32))
        }
        ValType::I64 => {
            let v = value.as_i64()
                .ok_or_else(|| anyhow!("Arg {}: expected integer", arg_index))?;
            Ok(Val::I64(v))
        }
        ValType::F32 => {
            let v = value.as_f64()
                .ok_or_else(|| anyhow!("Arg {}: expected float", arg_index))?;
            Ok(Val::F32((v as f32).to_bits()))
        }
        ValType::F64 => {
            let v = value.as_f64()
                .ok_or_else(|| anyhow!("Arg {}: expected float", arg_index))?;
            Ok(Val::F64(v.to_bits()))
        }
        _ => Err(anyhow!("Arg {}: unsupported type {:?}", arg_index, expected)),
    }
}
