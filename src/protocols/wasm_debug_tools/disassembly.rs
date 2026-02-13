use anyhow::{Result, anyhow};
use serde_json::Value;
use wasmparser::{Parser, Payload};

use super::state::{WasmDebugState, validate_module_id};

impl WasmDebugState {
    /// Disassemble a loaded module (or a single function) to WAT text
    pub fn disassemble_module(
        &self,
        module_id: &str,
        function_index: Option<u32>,
        function_name: Option<&str>,
        _fold_expressions: bool,
    ) -> Result<Value> {
        validate_module_id(module_id)?;
        let loaded = self.get_module(module_id)
            .ok_or_else(|| anyhow!("Module '{}' not found", module_id))?;

        // Resolve function_name to function_index if provided
        let target_func_index = if let Some(name) = function_name {
            self.resolve_export_func_index(module_id, name)?
        } else {
            function_index
        };

        if let Some(func_idx) = target_func_index {
            // Disassemble a single function
            self.disassemble_single_function(&loaded.binary, func_idx)
        } else {
            // Full module disassembly using wasmprinter
            let wat = wasmprinter::print_bytes(&loaded.binary)
                .map_err(|e| anyhow!("Failed to disassemble module: {}", e))?;

            Ok(serde_json::json!({
                "module_id": module_id,
                "format": "wat",
                "wat": wat,
                "binary_size": loaded.binary.len(),
            }))
        }
    }

    /// Resolve an export name to a function index
    fn resolve_export_func_index(&self, module_id: &str, name: &str) -> Result<Option<u32>> {
        let loaded = self.get_module(module_id)
            .ok_or_else(|| anyhow!("Module '{}' not found", module_id))?;

        let parser = Parser::new(0);
        for payload in parser.parse_all(&loaded.binary) {
            if let Ok(Payload::ExportSection(reader)) = payload {
                for export in reader {
                    if let Ok(export) = export {
                        if export.name == name {
                            if let wasmparser::ExternalKind::Func = export.kind {
                                return Ok(Some(export.index));
                            } else {
                                return Err(anyhow!("Export '{}' is not a function", name));
                            }
                        }
                    }
                }
            }
        }
        Err(anyhow!("Export function '{}' not found", name))
    }

    /// Disassemble a single function by extracting its code range
    fn disassemble_single_function(&self, binary: &[u8], target_func_idx: u32) -> Result<Value> {
        // First, do a full disassembly and try to extract the function
        let full_wat = wasmprinter::print_bytes(binary)
            .map_err(|e| anyhow!("Failed to disassemble module: {}", e))?;

        // Parse to find function boundaries and import count
        let parser = Parser::new(0);
        let mut import_func_count: u32 = 0;
        let mut code_func_count: u32 = 0;

        for payload in parser.parse_all(binary) {
            match payload {
                Ok(Payload::ImportSection(reader)) => {
                    for import in reader {
                        if let Ok(import) = import {
                            if let wasmparser::TypeRef::Func(_) = import.ty {
                                import_func_count += 1;
                            }
                        }
                    }
                }
                Ok(Payload::CodeSectionEntry(_)) => {
                    code_func_count += 1;
                }
                _ => {}
            }
        }

        // The target_func_idx includes imported functions
        // Code section functions start at index import_func_count
        let local_func_idx = if target_func_idx >= import_func_count {
            target_func_idx - import_func_count
        } else {
            return Err(anyhow!(
                "Function index {} is an imported function and cannot be disassembled",
                target_func_idx
            ));
        };

        if local_func_idx >= code_func_count {
            return Err(anyhow!(
                "Function index {} out of range (module has {} imported + {} local functions)",
                target_func_idx,
                import_func_count,
                code_func_count
            ));
        }

        // Try to extract the function from the full WAT output
        // Functions in WAT are formatted as (func $name ... )
        // We'll search for the function by looking for patterns
        let func_marker = format!("(func (;{};)", target_func_idx);
        let func_marker_alt = format!("(func ${}", target_func_idx);

        let func_wat = if let Some(start_pos) = full_wat.find(&func_marker)
            .or_else(|| full_wat.find(&func_marker_alt))
        {
            // Find the matching closing paren
            extract_balanced_parens(&full_wat[start_pos..])
                .unwrap_or_else(|| full_wat[start_pos..].to_string())
        } else {
            // Fallback: try to find by counting (func occurrences
            extract_nth_function(&full_wat, local_func_idx as usize)
                .unwrap_or_else(|| format!(";; Could not isolate function {}", target_func_idx))
        };

        Ok(serde_json::json!({
            "function_index": target_func_idx,
            "local_index": local_func_idx,
            "format": "wat",
            "wat": func_wat,
        }))
    }
}

/// Extract a balanced parenthesized expression from the start of the string
fn extract_balanced_parens(s: &str) -> Option<String> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, ch) in s.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape_next = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                return Some(s[..=i].to_string());
            }
        }
    }
    None
}

/// Extract the Nth function definition from WAT text
fn extract_nth_function(wat: &str, n: usize) -> Option<String> {
    let mut count = 0usize;
    let mut search_from = 0;

    while let Some(pos) = wat[search_from..].find("(func ") {
        let abs_pos = search_from + pos;
        if count == n {
            return extract_balanced_parens(&wat[abs_pos..]);
        }
        count += 1;
        search_from = abs_pos + 6; // Skip past "(func "
    }
    None
}
