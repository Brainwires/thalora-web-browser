use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use v8::{Context, CreateParams, HandleScope, OwnedIsolate, Script, TryCatch, Local, Value};

/// V8-based JavaScript engine that implements the same interface as the Boa engine
/// for compatibility with Thalora's existing architecture
pub struct V8JavaScriptEngine {
    isolate: OwnedIsolate,
    global_context: v8::Global<Context>,
    timers: Arc<Mutex<HashMap<u32, TimerHandle>>>,
    next_timer_id: Arc<Mutex<u32>>,
    start_time: Instant,
}

#[derive(Debug)]
struct TimerHandle {
    id: u32,
    callback: String, // Store JS code as string for simplicity
    interval: Option<std::time::Duration>,
    created_at: Instant,
}

impl V8JavaScriptEngine {
    /// Create a new V8 JavaScript engine instance
    pub fn new() -> Result<Self> {
        // Initialize V8 platform (should be done once per process)
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();

        // Create V8 isolate
        let mut isolate = v8::Isolate::new(CreateParams::default());

        // Create global context
        let global_context = {
            let scope = &mut HandleScope::new(&mut isolate);
            let context = Context::new(scope, Default::default());
            v8::Global::new(scope, context)
        };

        let timers = Arc::new(Mutex::new(HashMap::new()));
        let next_timer_id = Arc::new(Mutex::new(1));

        let mut engine = Self {
            isolate,
            global_context,
            timers,
            next_timer_id,
            start_time: Instant::now(),
        };

        // Setup Web APIs similar to the Boa implementation
        engine.setup_web_apis()?;

        Ok(engine)
    }

    /// Execute JavaScript code and return the result
    pub async fn execute(&mut self, code: &str) -> Result<serde_json::Value> {
        let scope = &mut HandleScope::new(&mut self.isolate);
        let context = Local::new(scope, &self.global_context);
        let scope = &mut v8::ContextScope::new(scope, context);

        let code_v8 = v8::String::new(scope, code)
            .ok_or_else(|| anyhow!("Failed to create V8 string from code"))?;

        let script = Script::compile(scope, code_v8, None)
            .ok_or_else(|| anyhow!("Failed to compile JavaScript"))?;

        let mut try_catch = TryCatch::new(scope);
        
        match script.run(&mut try_catch) {
            Some(result) => {
                // Convert V8 value to JSON-compatible value
                Self::v8_value_to_json(&mut try_catch, result)
            }
            None => {
                if let Some(exception) = try_catch.exception() {
                    let exception_str = exception.to_rust_string_lossy(&mut try_catch);
                    Err(anyhow!("JavaScript execution error: {}", exception_str))
                } else {
                    Err(anyhow!("JavaScript execution failed with unknown error"))
                }
            }
        }
    }

    /// Enhanced execution with ES2025+ compatibility (matching Boa interface)
    pub async fn execute_enhanced(&mut self, code: &str) -> Result<serde_json::Value> {
        // For now, delegate to regular execute
        // Future: Add syntax transformation for enhanced compatibility
        self.execute(code).await
    }

    /// V8-compatible execution (already V8!)
    pub async fn execute_v8_compatible(&mut self, code: &str) -> Result<serde_json::Value> {
        self.execute(code).await
    }

    /// Create a test instance for unit testing
    pub fn new_test() -> Result<Self> {
        Self::new()
    }

    /// Get a global object by name
    pub fn get_global_object(&mut self, name: &str) -> Result<Option<serde_json::Value>> {
        let scope = &mut HandleScope::new(&mut self.isolate);
        let context = Local::new(scope, &self.global_context);
        let scope = &mut v8::ContextScope::new(scope, context);

        let key = v8::String::new(scope, name)
            .ok_or_else(|| anyhow!("Failed to create V8 string for key"))?;

        let global = context.global(scope);
        
        match global.get(scope, key.into()) {
            Some(value) if !value.is_undefined() && !value.is_null() => {
                Ok(Some(Self::v8_value_to_json(scope, value)?))
            }
            _ => Ok(None)
        }
    }

    /// Set a global object
    pub fn set_global_object(&mut self, name: &str, value: serde_json::Value) -> Result<()> {
        let scope = &mut HandleScope::new(&mut self.isolate);
        let context = Local::new(scope, &self.global_context);
        let scope = &mut v8::ContextScope::new(scope, context);

        let key = v8::String::new(scope, name)
            .ok_or_else(|| anyhow!("Failed to create V8 string for key"))?;

        let v8_value = Self::json_to_v8_value(scope, value)?;
        let global = context.global(scope);

        global.set(scope, key.into(), v8_value)
            .ok_or_else(|| anyhow!("Failed to set global object"))?;

        Ok(())
    }

    /// Get engine version information
    pub fn version_info(&self) -> String {
        format!("V8 JavaScript Engine v{}", v8::V8::get_version())
    }

    /// Run pending microtasks (similar to Boa's run_jobs)
    pub fn run_jobs(&mut self) -> Result<()> {
        self.isolate.perform_microtask_checkpoint();
        Ok(())
    }

    /// Setup Web APIs similar to Boa implementation
    /// Note: Boa has many native implementations, V8 needs polyfills for these
    fn setup_web_apis(&mut self) -> Result<()> {
        let scope = &mut HandleScope::new(&mut self.isolate);
        let context = Local::new(scope, &self.global_context);
        let scope = &mut v8::ContextScope::new(scope, context);

        // Setup basic console object (Boa has this natively)
        Self::setup_console(scope)?;
        
        // Setup timer functions (Boa has these natively)
        Self::setup_timers(scope)?;

        // Setup fetch placeholder (Boa has this natively)
        Self::setup_fetch_placeholder(scope)?;
        
        // Setup storage APIs (Boa has these natively)
        Self::setup_storage_apis(scope)?;
        
        // Setup event system (Boa has this natively)
        Self::setup_event_system(scope)?;
        
        // Setup WebSocket placeholder (Boa has this natively)
        Self::setup_websocket_placeholder(scope)?;

        Ok(())
    }

    /// Setup console object with basic logging
    fn setup_console(scope: &mut v8::ContextScope<HandleScope>) -> Result<()> {
        let console_obj = v8::Object::new(scope);
        let global = scope.get_current_context().global(scope);

        // console.log function
        // console.log function
        let log_func = v8::Function::new(
            scope,
            |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
                let mut messages = Vec::new();
                for i in 0..args.length() {
                    let arg = args.get(i);
                    messages.push(arg.to_rust_string_lossy(scope));
                }
                tracing::info!("[V8 Console] {}", messages.join(" "));
            }
        ).unwrap();

        let log_key = v8::String::new(scope, "log").unwrap();
        console_obj.set(scope, log_key.into(), log_func.into());

        let console_key = v8::String::new(scope, "console").unwrap();
        global.set(scope, console_key.into(), console_obj.into());

        Ok(())
    }

    /// Setup timer functions (setTimeout, setInterval, clearTimeout, clearInterval)
    fn setup_timers(scope: &mut v8::ContextScope<HandleScope>) -> Result<()> {
        let global = scope.get_current_context().global(scope);

        // setTimeout function (simplified)
        let set_timeout = v8::Function::new(
            scope,
            |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
                if args.length() >= 2 {
                    let _callback = args.get(0).to_rust_string_lossy(scope);
                    let timeout = args.get(1).int32_value(scope).unwrap_or(0);
                    tracing::debug!("[V8 Timer] setTimeout called with {}ms timeout", timeout);
                    // Return a timer ID
                    let timer_id = v8::Integer::new(scope, 1);
                    rv.set(timer_id.into());
                }
            }
        ).unwrap();

        let set_timeout_key = v8::String::new(scope, "setTimeout").unwrap();
        global.set(scope, set_timeout_key.into(), set_timeout.into());

        // clearTimeout function
        let clear_timeout = v8::Function::new(
            scope,
            |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
                if args.length() > 0 {
                    let timer_id = args.get(0).int32_value(scope).unwrap_or(0);
                    tracing::debug!("[V8 Timer] clearTimeout called with ID: {}", timer_id);
                }
            }
        ).unwrap();

        let clear_timeout_key = v8::String::new(scope, "clearTimeout").unwrap();
        global.set(scope, clear_timeout_key.into(), clear_timeout.into());

        Ok(())
    }

    /// Setup fetch API placeholder
    fn setup_fetch_placeholder(scope: &mut v8::ContextScope<HandleScope>) -> Result<()> {
        let global = scope.get_current_context().global(scope);

        // Fetch API implementation (simplified)

        let fetch_func = v8::Function::new(
            scope,
            |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
                if args.length() > 0 {
                    let url = args.get(0).to_rust_string_lossy(scope);
                    tracing::warn!("[V8 Fetch] fetch('{}') called - returning rejected promise", url);
                }
                
                // Return a rejected promise for now
                let resolver = v8::PromiseResolver::new(scope).unwrap();
                let error = v8::String::new(scope, "fetch not implemented").unwrap();
                resolver.reject(scope, error.into());
                rv.set(resolver.get_promise(scope).into());
            }
        ).unwrap();

        let fetch_key = v8::String::new(scope, "fetch").unwrap();
        global.set(scope, fetch_key.into(), fetch_func.into());

        Ok(())
    }

    /// Setup storage APIs (localStorage, sessionStorage) - Boa has these natively
    fn setup_storage_apis(scope: &mut v8::ContextScope<HandleScope>) -> Result<()> {
        let global = scope.get_current_context().global(scope);

        // localStorage placeholder
        let local_storage_obj = v8::Object::new(scope);
        
        // localStorage.getItem implementation
        let get_item = v8::Function::new(
            scope,
            |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
                if args.length() > 0 {
                    let key_val = args.get(0);
                    let key = key_val.to_rust_string_lossy(scope);
                    tracing::debug!("[V8 Engine] localStorage.getItem: {}", key);
                }
                rv.set_null();
            }
        ).unwrap();
        
        // localStorage.setItem implementation
        let set_item = v8::Function::new(
            scope,
            |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
                if args.length() >= 2 {
                    let key_val = args.get(0);
                    let value_val = args.get(1);
                    let key = key_val.to_rust_string_lossy(scope);
                    let value = value_val.to_rust_string_lossy(scope);
                    tracing::debug!("[V8 Engine] localStorage.setItem: {} = {}", key, value);
                }
            }
        ).unwrap();

        let get_item_key = v8::String::new(scope, "getItem").unwrap();
        local_storage_obj.set(scope, get_item_key.into(), get_item.into());
        
        let set_item_key = v8::String::new(scope, "setItem").unwrap();
        local_storage_obj.set(scope, set_item_key.into(), set_item.into());

        let local_storage_key = v8::String::new(scope, "localStorage").unwrap();
        global.set(scope, local_storage_key.into(), local_storage_obj.into());

        Ok(())
    }

    /// Setup basic event system - Boa has this natively
    fn setup_event_system(scope: &mut v8::ContextScope<HandleScope>) -> Result<()> {
        let global = scope.get_current_context().global(scope);

        // Event constructor
        // Event constructor implementation (simplified)
        let event_constructor = v8::Function::new(
            scope,
            |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
                if args.length() > 0 {
                    let event_type_val = args.get(0);
                    let event_type = event_type_val.to_rust_string_lossy(scope);
                    tracing::debug!("[V8 Engine] Creating Event: {}", event_type);
                }
                
                // Return undefined for now - need proper scope for object creation
                rv.set_undefined();
            }
        ).unwrap();

        let event_key = v8::String::new(scope, "Event").unwrap();
        global.set(scope, event_key.into(), event_constructor.into());

        Ok(())
    }

    /// Setup WebSocket placeholder - Boa has this natively  
    fn setup_websocket_placeholder(scope: &mut v8::ContextScope<HandleScope>) -> Result<()> {
        let global = scope.get_current_context().global(scope);

        // WebSocket constructor implementation (simplified)
        let websocket_constructor = v8::Function::new(
            scope,
            |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
                if args.length() > 0 {
                    let url_val = args.get(0);
                    let url = url_val.to_rust_string_lossy(scope);
                    tracing::warn!("[V8 WebSocket] new WebSocket('{}') called - not implemented yet", url);
                }
                
                // Return undefined for now - need proper scope for object creation
                rv.set_undefined();
            }
        ).unwrap();

        let websocket_key = v8::String::new(scope, "WebSocket").unwrap();
        global.set(scope, websocket_key.into(), websocket_constructor.into());

        Ok(())
    }

    /// Convert V8 value to JSON-compatible value
    fn v8_value_to_json(scope: &mut v8::HandleScope, value: Local<Value>) -> Result<serde_json::Value> {
        if value.is_null() || value.is_undefined() {
            Ok(serde_json::Value::Null)
        } else if value.is_boolean() {
            Ok(serde_json::Value::Bool(value.boolean_value(scope)))
        } else if value.is_number() {
            if let Some(int_val) = value.int32_value(scope) {
                Ok(serde_json::Value::Number(serde_json::Number::from(int_val)))
            } else if let Some(num_val) = value.number_value(scope) {
                if let Some(json_num) = serde_json::Number::from_f64(num_val) {
                    Ok(serde_json::Value::Number(json_num))
                } else {
                    Ok(serde_json::Value::Null)
                }
            } else {
                Ok(serde_json::Value::Null)
            }
        } else if value.is_string() {
            Ok(serde_json::Value::String(value.to_rust_string_lossy(scope)))
        } else {
            // For complex objects, convert to string representation
            Ok(serde_json::Value::String(value.to_rust_string_lossy(scope)))
        }
    }

    /// Convert JSON value to V8 value
    fn json_to_v8_value<'a>(scope: &mut v8::HandleScope<'a>, value: serde_json::Value) -> Result<v8::Local<'a, v8::Value>> {
        match value {
            serde_json::Value::Null => Ok(v8::null(scope).into()),
            serde_json::Value::Bool(b) => Ok(v8::Boolean::new(scope, b).into()),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                        Ok(v8::Integer::new(scope, i as i32).into())
                    } else {
                        Ok(v8::Number::new(scope, i as f64).into())
                    }
                } else if let Some(f) = n.as_f64() {
                    Ok(v8::Number::new(scope, f).into())
                } else {
                    Ok(v8::null(scope).into())
                }
            }
            serde_json::Value::String(s) => {
                v8::String::new(scope, &s)
                    .map(|v| v.into())
                    .ok_or_else(|| anyhow!("Failed to create V8 string"))
            }
            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                // For complex types, serialize to JSON string and parse in V8
                let json_str = serde_json::to_string(&value)?;
                let js_code = format!("JSON.parse('{}')", json_str.replace("'", "\\'"));
                
                let code_v8 = v8::String::new(scope, &js_code)
                    .ok_or_else(|| anyhow!("Failed to create V8 string for JSON parse"))?;
                
                let script = Script::compile(scope, code_v8, None)
                    .ok_or_else(|| anyhow!("Failed to compile JSON parse script"))?;
                
                script.run(scope)
                    .ok_or_else(|| anyhow!("Failed to run JSON parse script"))
            }
        }
    }
}

impl Drop for V8JavaScriptEngine {
    fn drop(&mut self) {
        // V8 cleanup is handled automatically by the isolate
    }
}