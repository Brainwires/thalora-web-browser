use anyhow::{Result, anyhow};
use base64::Engine as Base64Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde_json::Value;
use wasmtime::Memory;

use super::state::{MAX_MEMORY_READ_SIZE, WasmDebugState, validate_module_id};

impl WasmDebugState {
    /// Read from a module's linear memory
    pub fn read_memory(
        &mut self,
        module_id: &str,
        offset: u64,
        memory_index: u32,
        length: usize,
        format: &str,
        count: usize,
    ) -> Result<Value> {
        validate_module_id(module_id)?;
        let loaded = self
            .get_module_mut(module_id)
            .ok_or_else(|| anyhow!("Module '{}' not found", module_id))?;

        let instance = loaded
            .instance
            .ok_or_else(|| anyhow!("Module '{}' is not instantiated", module_id))?;

        // Get the memory export
        let memory = get_memory_by_index(&instance, &mut loaded.store, memory_index)?;
        let mem_data = memory.data(&loaded.store);
        let mem_size = mem_data.len();

        let offset = offset as usize;

        // Calculate actual read size based on format and count
        let read_size = match format {
            "i32" | "u32" | "f32" => 4 * count,
            "i64" | "f64" => 8 * count,
            "u8" | "bytes" => length.min(count.max(1)),
            "u16" => 2 * count,
            "utf8" => length,
            "hex" => length,
            _ => length,
        };

        let actual_length = read_size.min(MAX_MEMORY_READ_SIZE);

        // Bounds check
        if offset >= mem_size {
            return Err(anyhow!(
                "Offset {} is beyond memory size {} bytes",
                offset,
                mem_size
            ));
        }
        let end = (offset + actual_length).min(mem_size);
        let data = &mem_data[offset..end];

        let result = match format {
            "hex" => format_hex_dump(data, offset),
            "i32" => format_typed_values::<i32>(data, count),
            "i64" => format_typed_values::<i64>(data, count),
            "f32" => format_typed_f32(data, count),
            "f64" => format_typed_f64(data, count),
            "u8" => format_typed_values::<u8>(data, count),
            "u16" => format_typed_values::<u16>(data, count),
            "u32" => format_typed_values::<u32>(data, count),
            "utf8" => format_utf8(data),
            "bytes" => {
                serde_json::json!({
                    "base64": BASE64.encode(data),
                    "length": data.len(),
                })
            }
            _ => format_hex_dump(data, offset),
        };

        Ok(serde_json::json!({
            "module_id": module_id,
            "memory_index": memory_index,
            "offset": offset,
            "length": data.len(),
            "memory_size": mem_size,
            "format": format,
            "data": result,
        }))
    }

    /// Write to a module's linear memory
    pub fn write_memory(
        &mut self,
        module_id: &str,
        offset: u64,
        memory_index: u32,
        hex_data: Option<&str>,
        base64_data: Option<&str>,
        typed_values: Option<&[Value]>,
        typed_format: Option<&str>,
        utf8_string: Option<&str>,
    ) -> Result<Value> {
        validate_module_id(module_id)?;
        let loaded = self
            .get_module_mut(module_id)
            .ok_or_else(|| anyhow!("Module '{}' not found", module_id))?;

        let instance = loaded
            .instance
            .ok_or_else(|| anyhow!("Module '{}' is not instantiated", module_id))?;

        let memory = get_memory_by_index(&instance, &mut loaded.store, memory_index)?;
        let mem_size = memory.data(&loaded.store).len();
        let offset = offset as usize;

        // Parse the write data from the appropriate source
        let bytes_to_write = if let Some(hex) = hex_data {
            parse_hex_string(hex)?
        } else if let Some(b64) = base64_data {
            BASE64
                .decode(b64)
                .map_err(|e| anyhow!("Invalid base64 data: {}", e))?
        } else if let Some(values) = typed_values {
            let fmt = typed_format.unwrap_or("i32");
            encode_typed_values(values, fmt)?
        } else if let Some(s) = utf8_string {
            s.as_bytes().to_vec()
        } else {
            return Err(anyhow!(
                "No write data provided. Provide one of: hex_data, base64_data, typed_values, utf8_string"
            ));
        };

        // Bounds check
        if offset + bytes_to_write.len() > mem_size {
            return Err(anyhow!(
                "Write of {} bytes at offset {} exceeds memory size {} bytes",
                bytes_to_write.len(),
                offset,
                mem_size
            ));
        }

        // Perform the write
        let mem_data = memory.data_mut(&mut loaded.store);
        mem_data[offset..offset + bytes_to_write.len()].copy_from_slice(&bytes_to_write);

        Ok(serde_json::json!({
            "module_id": module_id,
            "memory_index": memory_index,
            "offset": offset,
            "bytes_written": bytes_to_write.len(),
            "memory_size": mem_size,
            "status": "written"
        }))
    }
}

/// Get a Memory export by index from an instance
fn get_memory_by_index(
    instance: &wasmtime::Instance,
    store: &mut wasmtime::Store<super::state::FuelState>,
    memory_index: u32,
) -> Result<Memory> {
    let memories: Vec<Memory> = instance
        .exports(&mut *store)
        .filter_map(|export| export.into_memory())
        .collect();

    memories.get(memory_index as usize).copied().ok_or_else(|| {
        anyhow!(
            "Memory index {} not found (module has {} memories)",
            memory_index,
            memories.len()
        )
    })
}

/// Format data as a hex dump with ASCII sidebar
fn format_hex_dump(data: &[u8], base_offset: usize) -> Value {
    let mut lines = Vec::new();

    for (i, chunk) in data.chunks(16).enumerate() {
        let addr = base_offset + i * 16;
        let hex_part: String = chunk
            .iter()
            .enumerate()
            .map(|(j, b)| {
                if j == 8 {
                    format!(" {:02x}", b)
                } else {
                    format!("{:02x}", b)
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let ascii_part: String = chunk
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() || b == b' ' {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();

        // Pad hex part to fixed width
        let padded_hex = format!("{:<48}", hex_part);
        lines.push(format!("{:08x}  {}  |{}|", addr, padded_hex, ascii_part));
    }

    serde_json::json!({
        "hex_dump": lines.join("\n"),
        "total_bytes": data.len(),
    })
}

/// Format data as typed values (generic for integer types)
fn format_typed_values<T: Copy + Into<i128>>(data: &[u8], count: usize) -> Value
where
    T: serde::Serialize,
{
    let size = std::mem::size_of::<T>();
    let mut values = Vec::new();
    let actual_count = count.min(data.len() / size);

    for i in 0..actual_count {
        let start = i * size;
        if start + size <= data.len() {
            let bytes = &data[start..start + size];
            // Safety: we know the slice is the right size
            let mut arr = [0u8; 16];
            arr[..size].copy_from_slice(bytes);
            let val: T = unsafe { std::ptr::read_unaligned(arr.as_ptr() as *const T) };
            values.push(serde_json::to_value(val).unwrap_or(Value::Null));
        }
    }

    serde_json::json!({
        "values": values,
        "type": std::any::type_name::<T>().rsplit("::").next().unwrap_or("unknown"),
        "count": values.len(),
    })
}

/// Format data as f32 values
fn format_typed_f32(data: &[u8], count: usize) -> Value {
    let mut values = Vec::new();
    let actual_count = count.min(data.len() / 4);

    for i in 0..actual_count {
        let start = i * 4;
        if start + 4 <= data.len() {
            let bytes: [u8; 4] = data[start..start + 4].try_into().unwrap();
            let val = f32::from_le_bytes(bytes);
            values.push(serde_json::json!(val));
        }
    }

    serde_json::json!({
        "values": values,
        "type": "f32",
        "count": values.len(),
    })
}

/// Format data as f64 values
fn format_typed_f64(data: &[u8], count: usize) -> Value {
    let mut values = Vec::new();
    let actual_count = count.min(data.len() / 8);

    for i in 0..actual_count {
        let start = i * 8;
        if start + 8 <= data.len() {
            let bytes: [u8; 8] = data[start..start + 8].try_into().unwrap();
            let val = f64::from_le_bytes(bytes);
            values.push(serde_json::json!(val));
        }
    }

    serde_json::json!({
        "values": values,
        "type": "f64",
        "count": values.len(),
    })
}

/// Format data as a UTF-8 string
fn format_utf8(data: &[u8]) -> Value {
    match std::str::from_utf8(data) {
        Ok(s) => serde_json::json!({
            "string": s,
            "valid_utf8": true,
            "length": s.len(),
        }),
        Err(e) => {
            // Try lossy conversion
            let lossy = String::from_utf8_lossy(data);
            serde_json::json!({
                "string": lossy,
                "valid_utf8": false,
                "error": format!("Invalid UTF-8 at byte {}", e.valid_up_to()),
                "length": data.len(),
            })
        }
    }
}

/// Parse a hex string into bytes
fn parse_hex_string(hex: &str) -> Result<Vec<u8>> {
    let hex = hex.replace(' ', "").replace('\n', "").replace('\r', "");
    if hex.len() % 2 != 0 {
        return Err(anyhow!("Hex string must have even length"));
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16)
            .map_err(|e| anyhow!("Invalid hex at position {}: {}", i, e))?;
        bytes.push(byte);
    }
    Ok(bytes)
}

/// Encode typed values into bytes
fn encode_typed_values(values: &[Value], format: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();

    for val in values {
        match format {
            "i32" => {
                let v = val
                    .as_i64()
                    .ok_or_else(|| anyhow!("Expected integer value, got {:?}", val))?
                    as i32;
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            "i64" => {
                let v = val
                    .as_i64()
                    .ok_or_else(|| anyhow!("Expected integer value, got {:?}", val))?;
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            "f32" => {
                let v = val
                    .as_f64()
                    .ok_or_else(|| anyhow!("Expected float value, got {:?}", val))?
                    as f32;
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            "f64" => {
                let v = val
                    .as_f64()
                    .ok_or_else(|| anyhow!("Expected float value, got {:?}", val))?;
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            "u8" => {
                let v = val
                    .as_u64()
                    .ok_or_else(|| anyhow!("Expected unsigned integer value, got {:?}", val))?
                    as u8;
                bytes.push(v);
            }
            "u16" => {
                let v = val
                    .as_u64()
                    .ok_or_else(|| anyhow!("Expected unsigned integer value, got {:?}", val))?
                    as u16;
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            "u32" => {
                let v = val
                    .as_u64()
                    .ok_or_else(|| anyhow!("Expected unsigned integer value, got {:?}", val))?
                    as u32;
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            _ => return Err(anyhow!("Unsupported typed format: {}", format)),
        }
    }

    Ok(bytes)
}
