use serde_json::Value;

/// WASM Debug tool definitions for loading, inspecting, disassembling,
/// executing, and profiling WebAssembly modules
pub(crate) fn get_wasm_debug_tool_definitions() -> Vec<Value> {
    vec![
        // 1. Load Module
        serde_json::json!({
            "name": "wasm_debug_load_module",
            "description": "Load a WebAssembly module from base64 binary or WAT text. Returns a module_id for use with other wasm_debug tools. Modules with no imports are auto-instantiated by default.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "module_id": {
                        "type": "string",
                        "description": "Optional custom module ID (alphanumeric, hyphens, underscores, max 64 chars). Auto-generated if not provided."
                    },
                    "wasm_base64": {
                        "type": "string",
                        "description": "Base64-encoded WASM binary. Provide either this or wat_text."
                    },
                    "wat_text": {
                        "type": "string",
                        "description": "WebAssembly Text format (WAT) source. Provide either this or wasm_base64."
                    },
                    "name": {
                        "type": "string",
                        "description": "Human-readable name for the module"
                    },
                    "instantiate": {
                        "type": "boolean",
                        "description": "Whether to instantiate the module (default: true). Only modules with no imports can be auto-instantiated."
                    }
                }
            }
        }),
        // 2. Unload Module
        serde_json::json!({
            "name": "wasm_debug_unload_module",
            "description": "Unload a WASM module and free its resources.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "module_id": {
                        "type": "string",
                        "description": "ID of the module to unload"
                    }
                },
                "required": ["module_id"]
            }
        }),
        // 3. List Modules
        serde_json::json!({
            "name": "wasm_debug_list_modules",
            "description": "List all loaded WASM modules with their IDs, names, sizes, and instantiation status.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        // 4. Validate
        serde_json::json!({
            "name": "wasm_debug_validate",
            "description": "Validate a WASM module without loading it. Reports errors and structure summary. Useful for checking if a module is well-formed before loading.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "wasm_base64": {
                        "type": "string",
                        "description": "Base64-encoded WASM binary to validate"
                    },
                    "wat_text": {
                        "type": "string",
                        "description": "WAT text to validate (will be compiled to binary first)"
                    }
                }
            }
        }),
        // 5. Inspect
        serde_json::json!({
            "name": "wasm_debug_inspect",
            "description": "Inspect a loaded module's structure: imports, exports, functions (with type signatures), memories, tables, globals, custom sections, and start function.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "module_id": {
                        "type": "string",
                        "description": "ID of the module to inspect"
                    },
                    "sections": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Sections to include: imports, exports, functions, memories, tables, globals, custom_sections, start, all (default: all)"
                    },
                    "include_signatures": {
                        "type": "boolean",
                        "description": "Include function type signatures (default: true)"
                    }
                },
                "required": ["module_id"]
            }
        }),
        // 6. Disassemble
        serde_json::json!({
            "name": "wasm_debug_disassemble",
            "description": "Disassemble a WASM module (or a single function) to WebAssembly Text format (WAT).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "module_id": {
                        "type": "string",
                        "description": "ID of the module to disassemble"
                    },
                    "function_index": {
                        "type": "integer",
                        "description": "Optional: disassemble only this function by index"
                    },
                    "function_name": {
                        "type": "string",
                        "description": "Optional: disassemble only this exported function by name"
                    },
                    "fold_expressions": {
                        "type": "boolean",
                        "description": "Use folded expression syntax in WAT output (default: true)"
                    }
                },
                "required": ["module_id"]
            }
        }),
        // 7. Read Memory
        serde_json::json!({
            "name": "wasm_debug_read_memory",
            "description": "Read from a module's linear memory. Supports hex dump, typed values (i32/i64/f32/f64/u8/u16/u32), UTF-8 strings, and raw bytes.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "module_id": {
                        "type": "string",
                        "description": "ID of the module"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Byte offset to start reading from"
                    },
                    "memory_index": {
                        "type": "integer",
                        "description": "Memory index (default: 0)"
                    },
                    "length": {
                        "type": "integer",
                        "description": "Number of bytes to read (default: 256, max: 1MB)"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["hex", "i32", "i64", "f32", "f64", "u8", "u16", "u32", "utf8", "bytes"],
                        "description": "Output format (default: hex)"
                    },
                    "count": {
                        "type": "integer",
                        "description": "Number of typed values to read (default: 1, used with typed formats)"
                    }
                },
                "required": ["module_id", "offset"]
            }
        }),
        // 8. Write Memory
        serde_json::json!({
            "name": "wasm_debug_write_memory",
            "description": "Write to a module's linear memory. Provide data as hex, base64, typed values, or UTF-8 string.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "module_id": {
                        "type": "string",
                        "description": "ID of the module"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Byte offset to write to"
                    },
                    "memory_index": {
                        "type": "integer",
                        "description": "Memory index (default: 0)"
                    },
                    "hex_data": {
                        "type": "string",
                        "description": "Hex-encoded bytes to write (e.g. \"48656c6c6f\")"
                    },
                    "base64_data": {
                        "type": "string",
                        "description": "Base64-encoded bytes to write"
                    },
                    "typed_values": {
                        "type": "array",
                        "items": {},
                        "description": "Array of numeric values to write"
                    },
                    "typed_format": {
                        "type": "string",
                        "enum": ["i32", "i64", "f32", "f64", "u8", "u16", "u32"],
                        "description": "Format for typed_values"
                    },
                    "utf8_string": {
                        "type": "string",
                        "description": "UTF-8 string to write"
                    }
                },
                "required": ["module_id", "offset"]
            }
        }),
        // 9. Call Function
        serde_json::json!({
            "name": "wasm_debug_call_function",
            "description": "Call an exported WASM function with typed arguments. Returns results with types and execution timing.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "module_id": {
                        "type": "string",
                        "description": "ID of the module"
                    },
                    "function_name": {
                        "type": "string",
                        "description": "Name of the exported function to call"
                    },
                    "args": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "type": {
                                    "type": "string",
                                    "enum": ["i32", "i64", "f32", "f64"],
                                    "description": "WASM value type"
                                },
                                "value": {
                                    "description": "The argument value"
                                }
                            },
                            "required": ["type", "value"]
                        },
                        "description": "Function arguments with types"
                    },
                    "timeout_ms": {
                        "type": "integer",
                        "description": "Execution timeout in milliseconds (default: 5000, max: 30000)"
                    }
                },
                "required": ["module_id", "function_name"]
            }
        }),
        // 10. Profile Function
        serde_json::json!({
            "name": "wasm_debug_profile_function",
            "description": "Profile a WASM function's execution over N iterations. Returns wall-clock timing stats, fuel consumed, and memory usage.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "module_id": {
                        "type": "string",
                        "description": "ID of the module"
                    },
                    "function_name": {
                        "type": "string",
                        "description": "Name of the exported function to profile"
                    },
                    "args": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "type": {
                                    "type": "string",
                                    "enum": ["i32", "i64", "f32", "f64"]
                                },
                                "value": {
                                    "description": "The argument value"
                                }
                            },
                            "required": ["type", "value"]
                        },
                        "description": "Function arguments with types"
                    },
                    "iterations": {
                        "type": "integer",
                        "description": "Number of iterations to run (default: 1, max: 10000)"
                    },
                    "fuel_limit": {
                        "type": "integer",
                        "description": "Fuel limit per iteration (default: 1 billion)"
                    },
                    "timeout_ms": {
                        "type": "integer",
                        "description": "Overall timeout in milliseconds (default: 30000)"
                    }
                },
                "required": ["module_id", "function_name"]
            }
        }),
    ]
}
