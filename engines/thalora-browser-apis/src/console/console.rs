//! Console Web API implementation for Boa
//!
//! Native implementation of the Console standard
//! https://console.spec.whatwg.org/
//!
//! This implements the complete Console interface for debugging and logging

use boa_engine::{
    Context, JsArgs, JsResult, JsValue, NativeFunction, js_string, object::ObjectInitializer,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
            .function(
                NativeFunction::from_fn_ptr(Self::error),
                js_string!("error"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::warn),
                js_string!("warn"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::info),
                js_string!("info"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::debug),
                js_string!("debug"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::trace),
                js_string!("trace"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::assert),
                js_string!("assert"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::clear),
                js_string!("clear"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::count),
                js_string!("count"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::count_reset),
                js_string!("countReset"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::group),
                js_string!("group"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::group_collapsed),
                js_string!("groupCollapsed"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::group_end),
                js_string!("groupEnd"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::time),
                js_string!("time"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::time_log),
                js_string!("timeLog"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::time_end),
                js_string!("timeEnd"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::table),
                js_string!("table"),
                0,
            )
            .function(NativeFunction::from_fn_ptr(Self::dir), js_string!("dir"), 0)
            .function(
                NativeFunction::from_fn_ptr(Self::dirxml),
                js_string!("dirxml"),
                0,
            )
            .build();

        context
            .register_global_property(
                js_string!("console"),
                console_obj,
                boa_engine::property::Attribute::all(),
            )
            .expect("Failed to register console");
    }

    /// console.log()
    fn log(_: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args(args);
        println!("{}", message);
        Ok(JsValue::undefined())
    }

    /// console.error()
    fn error(_: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args(args);
        eprintln!("ERROR: {}", message);
        Ok(JsValue::undefined())
    }

    /// console.warn()
    fn warn(_: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args(args);
        eprintln!("WARN: {}", message);
        Ok(JsValue::undefined())
    }

    /// console.info()
    fn info(_: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args(args);
        println!("INFO: {}", message);
        Ok(JsValue::undefined())
    }

    /// console.debug()
    fn debug(_: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args(args);
        println!("DEBUG: {}", message);
        Ok(JsValue::undefined())
    }

    /// console.trace()
    fn trace(_: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args(args);
        println!("TRACE: {}", message);
        Ok(JsValue::undefined())
    }

    /// console.assert()
    fn assert(_: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
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
            args.get_or_undefined(0)
                .to_string(context)?
                .to_std_string_escaped()
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
            args.get_or_undefined(0)
                .to_string(context)?
                .to_std_string_escaped()
        };

        let mut counters = COUNTERS.lock().unwrap();
        counters.remove(&label);
        Ok(JsValue::undefined())
    }

    /// console.group()
    fn group(_: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args(args);
        println!("▼ {}", message);
        Ok(JsValue::undefined())
    }

    /// console.groupCollapsed()
    fn group_collapsed(_: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args(args);
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
            args.get_or_undefined(0)
                .to_string(context)?
                .to_std_string_escaped()
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
            args.get_or_undefined(0)
                .to_string(context)?
                .to_std_string_escaped()
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
            args.get_or_undefined(0)
                .to_string(context)?
                .to_std_string_escaped()
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
    fn table(_: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args(args);
        println!("TABLE: {}", message);
        Ok(JsValue::undefined())
    }

    /// console.dir()
    fn dir(_: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args(args);
        println!("DIR: {}", message);
        Ok(JsValue::undefined())
    }

    /// console.dirxml()
    fn dirxml(_: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let message = Self::format_args(args);
        println!("DIRXML: {}", message);
        Ok(JsValue::undefined())
    }

    /// Format console arguments into a string
    fn format_args(args: &[JsValue]) -> String {
        args.iter()
            .map(Self::value_to_string)
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Convert a JsValue to a string for console output
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
}
