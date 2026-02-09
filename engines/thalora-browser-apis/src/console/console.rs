//! Console Web API implementation for Boa
//!
//! Native implementation of the Console standard
//! https://console.spec.whatwg.org/
//!
//! This implements the complete Console interface for debugging and logging

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction,
    object::ObjectInitializer, js_string,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

/// Global timers storage
static TIMERS: Lazy<Arc<Mutex<HashMap<String, std::time::Instant>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Global counters storage
static COUNTERS: Lazy<Arc<Mutex<HashMap<String, usize>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Console implementation
pub struct Console;

impl Console {
    /// Initialize the console object in the global scope
    pub fn init(context: &mut Context) {
        let console_obj = ObjectInitializer::new(context)
            .function(NativeFunction::from_fn_ptr(Self::log), js_string!("log"), 0)
            .function(NativeFunction::from_fn_ptr(Self::error), js_string!("error"), 0)
            .function(NativeFunction::from_fn_ptr(Self::warn), js_string!("warn"), 0)
            .function(NativeFunction::from_fn_ptr(Self::info), js_string!("info"), 0)
            .function(NativeFunction::from_fn_ptr(Self::debug), js_string!("debug"), 0)
            .function(NativeFunction::from_fn_ptr(Self::trace), js_string!("trace"), 0)
            .function(NativeFunction::from_fn_ptr(Self::assert), js_string!("assert"), 0)
            .function(NativeFunction::from_fn_ptr(Self::clear), js_string!("clear"), 0)
            .function(NativeFunction::from_fn_ptr(Self::count), js_string!("count"), 0)
            .function(NativeFunction::from_fn_ptr(Self::count_reset), js_string!("countReset"), 0)
            .function(NativeFunction::from_fn_ptr(Self::group), js_string!("group"), 0)
            .function(NativeFunction::from_fn_ptr(Self::group_collapsed), js_string!("groupCollapsed"), 0)
            .function(NativeFunction::from_fn_ptr(Self::group_end), js_string!("groupEnd"), 0)
            .function(NativeFunction::from_fn_ptr(Self::time), js_string!("time"), 0)
            .function(NativeFunction::from_fn_ptr(Self::time_log), js_string!("timeLog"), 0)
            .function(NativeFunction::from_fn_ptr(Self::time_end), js_string!("timeEnd"), 0)
            .function(NativeFunction::from_fn_ptr(Self::table), js_string!("table"), 0)
            .function(NativeFunction::from_fn_ptr(Self::dir), js_string!("dir"), 0)
            .function(NativeFunction::from_fn_ptr(Self::dirxml), js_string!("dirxml"), 0)
            .build();

        context
            .register_global_property(js_string!("console"), console_obj, boa_engine::property::Attribute::all())
            .expect("Failed to register console");
    }

    /// console.log()
    fn log(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args_with_context(args, context);
        println!("{}", message);
        Ok(JsValue::undefined())
    }

    /// console.error()
    fn error(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args_with_context(args, context);
        eprintln!("ERROR: {}", message);
        // Print stack traces for Error objects to aid debugging
        for arg in args {
            if let Some(obj) = arg.as_object() {
                if let Ok(stack_val) = obj.get(js_string!("stack"), context) {
                    if let Some(stack_str) = stack_val.as_string() {
                        let stack = stack_str.to_std_string_escaped();
                        if !stack.is_empty() {
                            eprintln!("ERROR STACK: {}", stack);
                        }
                    }
                }
            }
        }
        Ok(JsValue::undefined())
    }

    /// console.warn()
    fn warn(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args_with_context(args, context);
        eprintln!("WARN: {}", message);
        Ok(JsValue::undefined())
    }

    /// console.info()
    fn info(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args_with_context(args, context);
        println!("INFO: {}", message);
        Ok(JsValue::undefined())
    }

    /// console.debug()
    fn debug(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args_with_context(args, context);
        println!("DEBUG: {}", message);
        Ok(JsValue::undefined())
    }

    /// console.trace()
    fn trace(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args_with_context(args, context);
        println!("TRACE: {}", message);
        Ok(JsValue::undefined())
    }

    /// console.assert()
    fn assert(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let condition = args.get_or_undefined(0).to_boolean();
        if !condition {
            let message = Self::format_args(&args[1..]);
            eprintln!("Assertion failed: {}", message);
        }
        Ok(JsValue::undefined())
    }

    /// console.clear()
    fn clear(_: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // In a real terminal, this would clear the screen
        println!("\x1B[2J\x1B[1;1H");
        Ok(JsValue::undefined())
    }

    /// console.count()
    fn count(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = if args.is_empty() {
            "default".to_string()
        } else {
            args.get_or_undefined(0).to_string(context)?.to_std_string_escaped()
        };

        let mut counters = COUNTERS.lock().unwrap();
        let count = counters.entry(label.clone()).or_insert(0);
        *count += 1;
        println!("{}: {}", label, count);
        Ok(JsValue::undefined())
    }

    /// console.countReset()
    fn count_reset(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = if args.is_empty() {
            "default".to_string()
        } else {
            args.get_or_undefined(0).to_string(context)?.to_std_string_escaped()
        };

        let mut counters = COUNTERS.lock().unwrap();
        counters.remove(&label);
        Ok(JsValue::undefined())
    }

    /// console.group()
    fn group(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args_with_context(args, context);
        println!("▼ {}", message);
        Ok(JsValue::undefined())
    }

    /// console.groupCollapsed()
    fn group_collapsed(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args_with_context(args, context);
        println!("► {}", message);
        Ok(JsValue::undefined())
    }

    /// console.groupEnd()
    fn group_end(_: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::undefined())
    }

    /// console.time()
    fn time(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = if args.is_empty() {
            "default".to_string()
        } else {
            args.get_or_undefined(0).to_string(context)?.to_std_string_escaped()
        };

        let mut timers = TIMERS.lock().unwrap();
        timers.insert(label, std::time::Instant::now());
        Ok(JsValue::undefined())
    }

    /// console.timeLog()
    fn time_log(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = if args.is_empty() {
            "default".to_string()
        } else {
            args.get_or_undefined(0).to_string(context)?.to_std_string_escaped()
        };

        let timers = TIMERS.lock().unwrap();
        if let Some(start) = timers.get(&label) {
            let elapsed = start.elapsed();
            println!("{}: {:?}", label, elapsed);
        } else {
            eprintln!("Timer '{}' does not exist", label);
        }
        Ok(JsValue::undefined())
    }

    /// console.timeEnd()
    fn time_end(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = if args.is_empty() {
            "default".to_string()
        } else {
            args.get_or_undefined(0).to_string(context)?.to_std_string_escaped()
        };

        let mut timers = TIMERS.lock().unwrap();
        if let Some(start) = timers.remove(&label) {
            let elapsed = start.elapsed();
            println!("{}: {:?}", label, elapsed);
        } else {
            eprintln!("Timer '{}' does not exist", label);
        }
        Ok(JsValue::undefined())
    }

    /// console.table()
    fn table(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args_with_context(args, context);
        println!("TABLE: {}", message);
        Ok(JsValue::undefined())
    }

    /// console.dir()
    fn dir(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args_with_context(args, context);
        println!("DIR: {}", message);
        Ok(JsValue::undefined())
    }

    /// console.dirxml()
    fn dirxml(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args_with_context(args, context);
        println!("DIRXML: {}", message);
        Ok(JsValue::undefined())
    }

    /// Format console arguments into a string (no context, used by assert)
    fn format_args(args: &[JsValue]) -> String {
        args.iter()
            .map(|arg| Self::value_to_string(arg))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Format console arguments into a string with context for proper object serialization
    fn format_args_with_context(args: &[JsValue], context: &mut Context) -> String {
        let mut parts = Vec::with_capacity(args.len());
        for arg in args {
            parts.push(Self::value_to_string_with_context(arg, context));
        }
        parts.join(" ")
    }

    /// Convert a JsValue to a string for console output (without context)
    fn value_to_string(value: &JsValue) -> String {
        if value.is_null() {
            "null".to_string()
        } else if value.is_undefined() {
            "undefined".to_string()
        } else if value.is_boolean() {
            value.to_boolean().to_string()
        } else if value.is_number() {
            format!("{}", value.as_number().unwrap())
        } else if let Some(s) = value.as_string() {
            s.to_std_string_escaped()
        } else if value.is_object() {
            "[object Object]".to_string()
        } else {
            format!("{:?}", value)
        }
    }

    /// Convert a JsValue to a string with context for proper object serialization.
    /// For objects, tries .message property (Error objects), then .toString(),
    /// then falls back to JSON.stringify for meaningful output.
    fn value_to_string_with_context(value: &JsValue, context: &mut Context) -> String {
        if value.is_null() {
            return "null".to_string();
        }
        if value.is_undefined() {
            return "undefined".to_string();
        }
        if value.is_boolean() {
            return value.to_boolean().to_string();
        }
        if value.is_number() {
            return format!("{}", value.as_number().unwrap());
        }
        if let Some(s) = value.as_string() {
            return s.to_std_string_escaped();
        }
        if let Some(obj) = value.as_object() {
            // Try .message property first (for Error objects)
            if let Ok(msg) = obj.get(js_string!("message"), context) {
                if let Some(s) = msg.as_string() {
                    let msg_str = s.to_std_string_escaped();
                    // Also try to get the error name for "TypeError: msg" format
                    if let Ok(name_val) = obj.get(js_string!("name"), context) {
                        if let Some(name_s) = name_val.as_string() {
                            let name_str = name_s.to_std_string_escaped();
                            if !name_str.is_empty() {
                                return format!("{}: {}", name_str, msg_str);
                            }
                        }
                    }
                    return msg_str;
                }
            }
            // Try .toString() — skip if it returns the generic "[object Object]"
            if let Ok(s) = value.to_string(context) {
                let result = s.to_std_string_escaped();
                if result != "[object Object]" {
                    return result;
                }
            }
            // Try JSON.stringify for a meaningful representation
            if let Ok(json_fn) = context.global_object().get(js_string!("JSON"), context) {
                if let Some(json_obj) = json_fn.as_object() {
                    if let Ok(stringify) = json_obj.get(js_string!("stringify"), context) {
                        if let Some(stringify_obj) = stringify.as_object() {
                            if let Ok(result) = stringify_obj.call(&json_fn, &[value.clone()], context) {
                                if let Some(s) = result.as_string() {
                                    let json_str = s.to_std_string_escaped();
                                    if !json_str.is_empty() && json_str != "undefined" {
                                        return json_str;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            return "[object Object]".to_string();
        }
        format!("{:?}", value)
    }
}
