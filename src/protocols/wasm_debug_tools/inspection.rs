use anyhow::{Result, anyhow};
use serde_json::Value;
use wasmparser::{Parser, Payload, TypeRef, ValType};

use super::state::{WasmDebugState, validate_module_id};

/// Convert a wasmparser ValType to a string representation
fn valtype_to_string(vt: &ValType) -> String {
    match vt {
        ValType::I32 => "i32".to_string(),
        ValType::I64 => "i64".to_string(),
        ValType::F32 => "f32".to_string(),
        ValType::F64 => "f64".to_string(),
        ValType::V128 => "v128".to_string(),
        ValType::Ref(rt) => format!("ref({:?})", rt),
    }
}

impl WasmDebugState {
    /// Inspect a loaded module's structure using wasmparser
    pub fn inspect_module(
        &self,
        module_id: &str,
        sections: Option<&[String]>,
        include_signatures: bool,
    ) -> Result<Value> {
        validate_module_id(module_id)?;
        let loaded = self.get_module(module_id)
            .ok_or_else(|| anyhow!("Module '{}' not found", module_id))?;

        let want_all = sections.is_none()
            || sections.as_ref().map_or(false, |s| s.iter().any(|x| x == "all"));
        let want = |name: &str| -> bool {
            want_all || sections.as_ref().map_or(false, |s| s.iter().any(|x| x == name))
        };

        let mut result = serde_json::json!({
            "module_id": module_id,
            "name": loaded.name,
            "binary_size": loaded.binary.len(),
            "instantiated": loaded.instance.is_some(),
        });

        // Parse the binary with wasmparser
        let parser = Parser::new(0);
        let mut imports = Vec::new();
        let mut exports = Vec::new();
        let mut functions = Vec::new();
        let mut memories = Vec::new();
        let mut tables = Vec::new();
        let mut globals = Vec::new();
        let mut custom_sections = Vec::new();
        let mut start_function: Option<u32> = None;
        let mut type_section: Vec<Value> = Vec::new();
        let mut func_type_indices: Vec<u32> = Vec::new();
        let mut import_func_count: u32 = 0;

        for payload in parser.parse_all(&loaded.binary) {
            match payload {
                Ok(Payload::TypeSection(reader)) => {
                    for (i, ty) in reader.into_iter_err_on_gc_types().enumerate() {
                        match ty {
                            Ok(func_type) => {
                                let params: Vec<String> = func_type.params().iter()
                                    .map(|p| valtype_to_string(p))
                                    .collect();
                                let results: Vec<String> = func_type.results().iter()
                                    .map(|r| valtype_to_string(r))
                                    .collect();
                                type_section.push(serde_json::json!({
                                    "index": i,
                                    "params": params,
                                    "results": results,
                                }));
                            }
                            Err(_) => {}
                        }
                    }
                }
                Ok(Payload::ImportSection(reader)) => {
                    for import in reader {
                        if let Ok(import) = import {
                            let kind = match import.ty {
                                TypeRef::Func(idx) => {
                                    import_func_count += 1;
                                    let sig = if include_signatures {
                                        type_section.get(idx as usize).cloned()
                                    } else {
                                        None
                                    };
                                    serde_json::json!({
                                        "kind": "function",
                                        "type_index": idx,
                                        "signature": sig,
                                    })
                                }
                                TypeRef::Table(t) => serde_json::json!({
                                    "kind": "table",
                                    "element_type": format!("{:?}", t.element_type),
                                    "initial": t.initial,
                                    "maximum": t.maximum,
                                }),
                                TypeRef::Memory(m) => serde_json::json!({
                                    "kind": "memory",
                                    "initial": m.initial,
                                    "maximum": m.maximum,
                                    "memory64": m.memory64,
                                    "shared": m.shared,
                                }),
                                TypeRef::Global(g) => serde_json::json!({
                                    "kind": "global",
                                    "content_type": valtype_to_string(&g.content_type),
                                    "mutable": g.mutable,
                                }),
                                TypeRef::Tag(t) => serde_json::json!({
                                    "kind": "tag",
                                    "type_index": t.func_type_idx,
                                }),
                            };
                            imports.push(serde_json::json!({
                                "module": import.module,
                                "name": import.name,
                                "details": kind,
                            }));
                        }
                    }
                }
                Ok(Payload::FunctionSection(reader)) => {
                    for func_idx in reader {
                        if let Ok(idx) = func_idx {
                            func_type_indices.push(idx);
                        }
                    }
                }
                Ok(Payload::ExportSection(reader)) => {
                    for export in reader {
                        if let Ok(export) = export {
                            let kind = match export.kind {
                                wasmparser::ExternalKind::Func => "function",
                                wasmparser::ExternalKind::Table => "table",
                                wasmparser::ExternalKind::Memory => "memory",
                                wasmparser::ExternalKind::Global => "global",
                                wasmparser::ExternalKind::Tag => "tag",
                            };
                            exports.push(serde_json::json!({
                                "name": export.name,
                                "kind": kind,
                                "index": export.index,
                            }));
                        }
                    }
                }
                Ok(Payload::MemorySection(reader)) => {
                    for (i, mem) in reader.into_iter().enumerate() {
                        if let Ok(mem) = mem {
                            memories.push(serde_json::json!({
                                "index": i,
                                "initial_pages": mem.initial,
                                "maximum_pages": mem.maximum,
                                "memory64": mem.memory64,
                                "shared": mem.shared,
                            }));
                        }
                    }
                }
                Ok(Payload::TableSection(reader)) => {
                    for (i, table) in reader.into_iter().enumerate() {
                        if let Ok(table) = table {
                            tables.push(serde_json::json!({
                                "index": i,
                                "element_type": format!("{:?}", table.ty.element_type),
                                "initial": table.ty.initial,
                                "maximum": table.ty.maximum,
                            }));
                        }
                    }
                }
                Ok(Payload::GlobalSection(reader)) => {
                    for (i, global) in reader.into_iter().enumerate() {
                        if let Ok(global) = global {
                            globals.push(serde_json::json!({
                                "index": i,
                                "content_type": valtype_to_string(&global.ty.content_type),
                                "mutable": global.ty.mutable,
                            }));
                        }
                    }
                }
                Ok(Payload::CustomSection(reader)) => {
                    custom_sections.push(serde_json::json!({
                        "name": reader.name(),
                        "data_size": reader.data().len(),
                    }));
                }
                Ok(Payload::StartSection { func, .. }) => {
                    start_function = Some(func);
                }
                _ => {}
            }
        }

        // Build function list with signatures
        if want("functions") {
            for (i, &type_idx) in func_type_indices.iter().enumerate() {
                let func_index = import_func_count + i as u32;
                let mut func_info = serde_json::json!({
                    "index": func_index,
                    "type_index": type_idx,
                });

                if include_signatures {
                    if let Some(sig) = type_section.get(type_idx as usize) {
                        func_info["signature"] = sig.clone();
                    }
                }

                // Check if this function is exported
                for exp in &exports {
                    if exp["kind"] == "function" && exp["index"] == func_index {
                        func_info["export_name"] = exp["name"].clone();
                    }
                }

                functions.push(func_info);
            }
        }

        // Build the result based on requested sections
        let map = result.as_object_mut().unwrap();

        if want("imports") {
            map.insert("imports".to_string(), serde_json::json!(imports));
        }
        if want("exports") {
            map.insert("exports".to_string(), serde_json::json!(exports));
        }
        if want("functions") {
            map.insert("functions".to_string(), serde_json::json!(functions));
        }
        if want("memories") {
            map.insert("memories".to_string(), serde_json::json!(memories));
        }
        if want("tables") {
            map.insert("tables".to_string(), serde_json::json!(tables));
        }
        if want("globals") {
            map.insert("globals".to_string(), serde_json::json!(globals));
        }
        if want("custom_sections") {
            map.insert("custom_sections".to_string(), serde_json::json!(custom_sections));
        }
        if want("start") {
            map.insert("start_function".to_string(), serde_json::json!(start_function));
        }

        Ok(result)
    }
}
