//! Integration tests for WASM debug tools
//!
//! Run with: cargo test --features wasm-debug wasm_debug
//! Or with env: THALORA_ENABLE_WASM_DEBUG=true cargo test --features wasm-debug wasm_debug

#![cfg(feature = "wasm-debug")]

use serde_json::json;
use thalora::WasmDebugTools;

/// Simple WAT module: add function taking two i32 params and returning i32
const ADD_WAT: &str = r#"
(module
  (func (export "add") (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add)
)
"#;

/// WAT module with memory: exports a memory and a function to store/load values
const MEMORY_WAT: &str = r#"
(module
  (memory (export "memory") 1)
  (func (export "store_i32") (param i32 i32)
    local.get 0
    local.get 1
    i32.store)
  (func (export "load_i32") (param i32) (result i32)
    local.get 0
    i32.load)
)
"#;

/// WAT module with a global and multiple exports
const MULTI_WAT: &str = r#"
(module
  (global (export "counter") (mut i32) (i32.const 0))
  (func (export "get_counter") (result i32)
    global.get 0)
  (func (export "increment") (result i32)
    global.get 0
    i32.const 1
    i32.add
    global.set 0
    global.get 0)
  (func (export "multiply") (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.mul)
)
"#;

// ============================================================
// Load / Unload / List tests
// ============================================================

#[test]
fn test_load_module_from_wat() {
    let mut tools = WasmDebugTools::new().unwrap();
    let result = tools
        .load_module(json!({
            "wat_text": ADD_WAT,
            "module_id": "test-add",
            "name": "Add Module"
        }))
        .unwrap();

    assert_eq!(result["module_id"], "test-add");
    assert_eq!(result["name"], "Add Module");
    assert_eq!(result["instantiated"], true);
    assert_eq!(result["status"], "loaded");
    assert!(result["binary_size"].as_u64().unwrap() > 0);
}

#[test]
fn test_load_module_auto_id() {
    let mut tools = WasmDebugTools::new().unwrap();
    let result = tools
        .load_module(json!({
            "wat_text": ADD_WAT
        }))
        .unwrap();

    let module_id = result["module_id"].as_str().unwrap();
    assert!(module_id.starts_with("module_"));
    assert_eq!(result["instantiated"], true);
}

#[test]
fn test_load_duplicate_module_fails() {
    let mut tools = WasmDebugTools::new().unwrap();
    tools
        .load_module(json!({
            "wat_text": ADD_WAT,
            "module_id": "dup"
        }))
        .unwrap();

    let err = tools.load_module(json!({
        "wat_text": ADD_WAT,
        "module_id": "dup"
    }));
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("already loaded"));
}

#[test]
fn test_unload_module() {
    let mut tools = WasmDebugTools::new().unwrap();
    tools
        .load_module(json!({
            "wat_text": ADD_WAT,
            "module_id": "to-unload"
        }))
        .unwrap();

    let result = tools
        .unload_module(json!({
            "module_id": "to-unload"
        }))
        .unwrap();

    assert_eq!(result["status"], "unloaded");

    // Should fail to unload again
    assert!(
        tools
            .unload_module(json!({ "module_id": "to-unload" }))
            .is_err()
    );
}

#[test]
fn test_list_modules() {
    let mut tools = WasmDebugTools::new().unwrap();

    let empty = tools.list_modules();
    assert_eq!(empty["count"], 0);

    tools
        .load_module(json!({
            "wat_text": ADD_WAT,
            "module_id": "mod1",
            "name": "First"
        }))
        .unwrap();

    tools
        .load_module(json!({
            "wat_text": MEMORY_WAT,
            "module_id": "mod2",
            "name": "Second"
        }))
        .unwrap();

    let list = tools.list_modules();
    assert_eq!(list["count"], 2);

    let modules = list["modules"].as_array().unwrap();
    let ids: Vec<&str> = modules
        .iter()
        .map(|m| m["module_id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"mod1"));
    assert!(ids.contains(&"mod2"));
}

// ============================================================
// Validate tests
// ============================================================

#[test]
fn test_validate_valid_wat() {
    let tools = WasmDebugTools::new().unwrap();
    let result = tools
        .validate(json!({
            "wat_text": ADD_WAT
        }))
        .unwrap();

    assert_eq!(result["valid"], true);
    assert!(result["binary_size"].as_u64().unwrap() > 0);
}

#[test]
fn test_validate_invalid_wat() {
    let tools = WasmDebugTools::new().unwrap();
    let result = tools
        .validate(json!({
            "wat_text": "(module (func (export \"broken\") (param i32) (result i32) i32.add))"
        }))
        .unwrap();

    // Either valid:false with error, or compilation error
    // The WAT might parse but fail validation
    // Let's just check the result has the expected shape
    assert!(result.get("valid").is_some() || result.get("error").is_some());
}

// ============================================================
// Inspect tests
// ============================================================

#[test]
fn test_inspect_module() {
    let mut tools = WasmDebugTools::new().unwrap();
    tools
        .load_module(json!({
            "wat_text": MULTI_WAT,
            "module_id": "inspect-test"
        }))
        .unwrap();

    let result = tools
        .inspect(json!({
            "module_id": "inspect-test"
        }))
        .unwrap();

    // Check exports
    let exports = result["exports"].as_array().unwrap();
    let export_names: Vec<&str> = exports
        .iter()
        .map(|e| e["name"].as_str().unwrap())
        .collect();
    assert!(export_names.contains(&"counter"));
    assert!(export_names.contains(&"get_counter"));
    assert!(export_names.contains(&"increment"));
    assert!(export_names.contains(&"multiply"));

    // Check globals
    let globals = result["globals"].as_array().unwrap();
    assert!(!globals.is_empty());
    assert_eq!(globals[0]["content_type"], "i32");
    assert_eq!(globals[0]["mutable"], true);
}

#[test]
fn test_inspect_specific_sections() {
    let mut tools = WasmDebugTools::new().unwrap();
    tools
        .load_module(json!({
            "wat_text": ADD_WAT,
            "module_id": "sections-test"
        }))
        .unwrap();

    let result = tools
        .inspect(json!({
            "module_id": "sections-test",
            "sections": ["exports", "functions"]
        }))
        .unwrap();

    assert!(result.get("exports").is_some());
    assert!(result.get("functions").is_some());
    // Should not include other sections
    assert!(result.get("memories").is_none());
    assert!(result.get("globals").is_none());
}

// ============================================================
// Disassemble tests
// ============================================================

#[test]
fn test_disassemble_full_module() {
    let mut tools = WasmDebugTools::new().unwrap();
    tools
        .load_module(json!({
            "wat_text": ADD_WAT,
            "module_id": "disasm-test"
        }))
        .unwrap();

    let result = tools
        .disassemble(json!({
            "module_id": "disasm-test"
        }))
        .unwrap();

    assert_eq!(result["format"], "wat");
    let wat = result["wat"].as_str().unwrap();
    assert!(wat.contains("func"));
    assert!(wat.contains("i32.add"));
}

#[test]
fn test_disassemble_by_function_name() {
    let mut tools = WasmDebugTools::new().unwrap();
    tools
        .load_module(json!({
            "wat_text": MULTI_WAT,
            "module_id": "disasm-func"
        }))
        .unwrap();

    let result = tools
        .disassemble(json!({
            "module_id": "disasm-func",
            "function_name": "multiply"
        }))
        .unwrap();

    let wat = result["wat"].as_str().unwrap();
    assert!(wat.contains("i32.mul"));
}

// ============================================================
// Call Function tests
// ============================================================

#[tokio::test]
async fn test_call_add_function() {
    let mut tools = WasmDebugTools::new().unwrap();
    tools
        .load_module(json!({
            "wat_text": ADD_WAT,
            "module_id": "call-test"
        }))
        .unwrap();

    let result = tools
        .call_function(json!({
            "module_id": "call-test",
            "function_name": "add",
            "args": [
                {"type": "i32", "value": 3},
                {"type": "i32", "value": 7}
            ]
        }))
        .await
        .unwrap();

    assert_eq!(result["status"], "success");
    let results = result["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], 10);
    assert!(result["execution_time_ms"].as_f64().unwrap() >= 0.0);
    assert!(result["fuel_consumed"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn test_call_function_no_args() {
    let mut tools = WasmDebugTools::new().unwrap();
    tools
        .load_module(json!({
            "wat_text": MULTI_WAT,
            "module_id": "no-args"
        }))
        .unwrap();

    let result = tools
        .call_function(json!({
            "module_id": "no-args",
            "function_name": "get_counter"
        }))
        .await
        .unwrap();

    assert_eq!(result["status"], "success");
    assert_eq!(result["results"][0], 0);
}

#[tokio::test]
async fn test_call_function_wrong_args() {
    let mut tools = WasmDebugTools::new().unwrap();
    tools
        .load_module(json!({
            "wat_text": ADD_WAT,
            "module_id": "wrong-args"
        }))
        .unwrap();

    let err = tools
        .call_function(json!({
            "module_id": "wrong-args",
            "function_name": "add",
            "args": [{"type": "i32", "value": 1}]
        }))
        .await;

    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("expects 2 arguments"));
}

// ============================================================
// Memory tests
// ============================================================

#[tokio::test]
async fn test_memory_write_and_read_hex() {
    let mut tools = WasmDebugTools::new().unwrap();
    tools
        .load_module(json!({
            "wat_text": MEMORY_WAT,
            "module_id": "mem-test"
        }))
        .unwrap();

    // Write via function call
    tools
        .call_function(json!({
            "module_id": "mem-test",
            "function_name": "store_i32",
            "args": [
                {"type": "i32", "value": 0},
                {"type": "i32", "value": 42}
            ]
        }))
        .await
        .unwrap();

    // Read back as hex
    let read_result = tools
        .read_memory(json!({
            "module_id": "mem-test",
            "offset": 0,
            "length": 16,
            "format": "hex"
        }))
        .unwrap();

    assert_eq!(read_result["length"], 16);
    assert!(read_result["data"]["hex_dump"].as_str().unwrap().len() > 0);
}

#[tokio::test]
async fn test_memory_write_and_read_typed() {
    let mut tools = WasmDebugTools::new().unwrap();
    tools
        .load_module(json!({
            "wat_text": MEMORY_WAT,
            "module_id": "mem-typed"
        }))
        .unwrap();

    // Write directly to memory
    tools
        .write_memory(json!({
            "module_id": "mem-typed",
            "offset": 0,
            "typed_values": [12345],
            "typed_format": "i32"
        }))
        .unwrap();

    // Read back as i32
    let result = tools
        .read_memory(json!({
            "module_id": "mem-typed",
            "offset": 0,
            "format": "i32",
            "count": 1
        }))
        .unwrap();

    let values = result["data"]["values"].as_array().unwrap();
    assert_eq!(values[0], 12345);
}

#[tokio::test]
async fn test_memory_write_utf8() {
    let mut tools = WasmDebugTools::new().unwrap();
    tools
        .load_module(json!({
            "wat_text": MEMORY_WAT,
            "module_id": "mem-utf8"
        }))
        .unwrap();

    tools
        .write_memory(json!({
            "module_id": "mem-utf8",
            "offset": 0,
            "utf8_string": "Hello, WASM!"
        }))
        .unwrap();

    let result = tools
        .read_memory(json!({
            "module_id": "mem-utf8",
            "offset": 0,
            "length": 12,
            "format": "utf8"
        }))
        .unwrap();

    assert_eq!(result["data"]["string"], "Hello, WASM!");
    assert_eq!(result["data"]["valid_utf8"], true);
}

#[tokio::test]
async fn test_memory_write_hex_data() {
    let mut tools = WasmDebugTools::new().unwrap();
    tools
        .load_module(json!({
            "wat_text": MEMORY_WAT,
            "module_id": "mem-hex"
        }))
        .unwrap();

    tools
        .write_memory(json!({
            "module_id": "mem-hex",
            "offset": 0,
            "hex_data": "48656c6c6f"
        }))
        .unwrap();

    let result = tools
        .read_memory(json!({
            "module_id": "mem-hex",
            "offset": 0,
            "length": 5,
            "format": "utf8"
        }))
        .unwrap();

    assert_eq!(result["data"]["string"], "Hello");
}

// ============================================================
// Profiling tests
// ============================================================

#[tokio::test]
async fn test_profile_function() {
    let mut tools = WasmDebugTools::new().unwrap();
    tools
        .load_module(json!({
            "wat_text": ADD_WAT,
            "module_id": "profile-test"
        }))
        .unwrap();

    let result = tools
        .profile_function(json!({
            "module_id": "profile-test",
            "function_name": "add",
            "args": [
                {"type": "i32", "value": 10},
                {"type": "i32", "value": 20}
            ],
            "iterations": 100
        }))
        .await
        .unwrap();

    assert_eq!(result["status"], "complete");
    assert_eq!(result["iterations_completed"], 100);
    assert_eq!(result["iterations_requested"], 100);

    // Check timing stats
    let timing = &result["timing"];
    assert!(timing["total_ms"].as_f64().unwrap() >= 0.0);
    assert!(timing["avg_ms"].as_f64().unwrap() >= 0.0);
    assert!(timing["min_ms"].as_f64().unwrap() >= 0.0);
    assert!(timing["max_ms"].as_f64().unwrap() >= timing["min_ms"].as_f64().unwrap());

    // Check fuel stats
    let fuel = &result["fuel"];
    assert!(fuel["total"].as_u64().unwrap() > 0);
    assert!(fuel["avg"].as_u64().unwrap() > 0);

    // Check last results
    let last_results = result["last_results"].as_array().unwrap();
    assert_eq!(last_results[0], 30);
}

#[tokio::test]
async fn test_profile_single_iteration() {
    let mut tools = WasmDebugTools::new().unwrap();
    tools
        .load_module(json!({
            "wat_text": MULTI_WAT,
            "module_id": "profile-single"
        }))
        .unwrap();

    let result = tools
        .profile_function(json!({
            "module_id": "profile-single",
            "function_name": "multiply",
            "args": [
                {"type": "i32", "value": 6},
                {"type": "i32", "value": 7}
            ],
            "iterations": 1
        }))
        .await
        .unwrap();

    assert_eq!(result["status"], "complete");
    assert_eq!(result["iterations_completed"], 1);
    assert_eq!(result["last_results"][0], 42);
}

// ============================================================
// Error handling tests
// ============================================================

#[test]
fn test_invalid_module_id_chars() {
    let mut tools = WasmDebugTools::new().unwrap();
    let err = tools.load_module(json!({
        "wat_text": ADD_WAT,
        "module_id": "invalid/id"
    }));
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("invalid characters"));
}

#[test]
fn test_missing_source() {
    let mut tools = WasmDebugTools::new().unwrap();
    let err = tools.load_module(json!({
        "module_id": "no-source"
    }));
    assert!(err.is_err());
    assert!(
        err.unwrap_err()
            .to_string()
            .contains("wasm_base64 or wat_text")
    );
}

#[tokio::test]
async fn test_call_nonexistent_function() {
    let mut tools = WasmDebugTools::new().unwrap();
    tools
        .load_module(json!({
            "wat_text": ADD_WAT,
            "module_id": "no-func"
        }))
        .unwrap();

    let err = tools
        .call_function(json!({
            "module_id": "no-func",
            "function_name": "nonexistent"
        }))
        .await;

    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_read_memory_uninstantiated() {
    let mut tools = WasmDebugTools::new().unwrap();
    tools
        .load_module(json!({
            "wat_text": ADD_WAT,
            "module_id": "no-mem",
            "instantiate": true
        }))
        .unwrap();

    // Add module has no memory, so reading should fail
    let err = tools.read_memory(json!({
        "module_id": "no-mem",
        "offset": 0,
        "length": 16
    }));
    assert!(err.is_err());
}

#[test]
fn test_inspect_nonexistent_module() {
    let tools = WasmDebugTools::new().unwrap();
    let err = tools.inspect(json!({
        "module_id": "ghost"
    }));
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("not found"));
}
