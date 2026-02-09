use anyhow::Result;
use serde_json::Value;
use thalora_browser_apis::boa_engine::js_string;

/// Global engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub engine_type: EngineType,
}

impl EngineConfig {
    pub fn new() -> Self {
        Self { engine_type: EngineType::Boa }
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Common interface for JavaScript engines in Thalora
///
/// Note: Made synchronous because JavaScript engines are inherently single-threaded
/// and Boa uses Rc/RefCell which are not Send/Sync. Async handling is done at higher levels.
pub trait ThaloraBrowserEngine {
    /// Execute JavaScript code and return the result
    fn execute(&mut self, code: &str) -> Result<Value>;

    /// Execute JavaScript with enhanced ES2025+ features
    fn execute_enhanced(&mut self, code: &str) -> Result<Value>;

    /// Get a global object by name
    fn get_global_object(&mut self, name: &str) -> Result<Option<Value>>;

    /// Set a global object
    fn set_global_object(&mut self, name: &str, value: Value) -> Result<()>;

    /// Get engine version information
    fn version_info(&self) -> String;

    /// Run pending microtasks/jobs
    fn run_jobs(&mut self) -> Result<()>;

    /// Process due timer callbacks (setTimeout/setInterval).
    /// Returns the number of timers that were executed.
    fn process_timers(&mut self) -> usize;

    /// Run the event loop: process timers and jobs until no more work or timeout.
    fn run_event_loop(&mut self, max_iterations: usize) -> Result<()>;

    /// Create a test instance (for unit testing)
    fn new_test() -> Result<Box<dyn ThaloraBrowserEngine>>
    where
        Self: Sized;

    /// Get engine type identifier
    fn engine_type(&self) -> &'static str;
}

/// Engine types available in Thalora
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineType {
    /// Boa JavaScript Engine (Pure Rust)
    Boa,
}

impl std::fmt::Display for EngineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineType::Boa => write!(f, "boa"),
        }
    }
}

impl std::str::FromStr for EngineType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "boa" => Ok(EngineType::Boa),
            _ => Err(anyhow::anyhow!("Unknown engine type: {}", s)),
        }
    }
}

/// Factory for creating JavaScript engines
pub struct EngineFactory;

impl EngineFactory {
    /// Create a new JavaScript engine of the specified type
    pub fn create_engine(engine_type: EngineType) -> Result<Box<dyn ThaloraBrowserEngine>> {
        match engine_type {
            EngineType::Boa => {
                let boa_engine = crate::engine::JavaScriptEngine::new()?;
                Ok(Box::new(BoaEngineWrapper::new(boa_engine)))
            }
        }
    }

    /// Get the default engine type
    pub fn default_engine() -> EngineType {
        EngineType::Boa
    }

    /// Create an engine using the default/configured engine type
    pub fn create_default_engine() -> Result<Box<dyn ThaloraBrowserEngine>> {
        Self::create_engine(Self::default_engine())
    }

    /// List available engines
    pub fn available_engines() -> Vec<EngineType> {
        vec![EngineType::Boa]
    }
}

/// Wrapper for Boa engine to implement the common trait
pub struct BoaEngineWrapper {
    engine: crate::engine::JavaScriptEngine,
}

impl BoaEngineWrapper {
    pub fn new(engine: crate::engine::JavaScriptEngine) -> Self {
        Self { engine }
    }
}

impl ThaloraBrowserEngine for BoaEngineWrapper {
    fn execute(&mut self, code: &str) -> Result<Value> {
        // Just execute simple JS directly - we'll create a simple version
        let result = futures::executor::block_on(self.engine.execute_enhanced(code))?;
        self.boa_to_json_value(result)
    }

    fn execute_enhanced(&mut self, code: &str) -> Result<Value> {
        let result = futures::executor::block_on(self.engine.execute_enhanced(code))?;
        self.boa_to_json_value(result)
    }

    fn get_global_object(&mut self, name: &str) -> Result<Option<Value>> {
        match self.engine.get_global_object(name)? {
            Some(js_value) => Ok(Some(self.boa_to_json_value(js_value)?)),
            None => Ok(None),
        }
    }

    fn set_global_object(&mut self, name: &str, value: Value) -> Result<()> {
        let js_value = self.json_to_boa_value(value)?;
        self.engine.set_global_object(name, js_value)
    }

    fn version_info(&self) -> String {
        self.engine.version_info()
    }

    fn run_jobs(&mut self) -> Result<()> {
        self.engine.run_jobs()
    }

    fn process_timers(&mut self) -> usize {
        self.engine.process_timers()
    }

    fn run_event_loop(&mut self, max_iterations: usize) -> Result<()> {
        self.engine.run_event_loop(max_iterations)
    }

    fn new_test() -> Result<Box<dyn ThaloraBrowserEngine>> {
        let engine = crate::engine::JavaScriptEngine::new_test()?;
        Ok(Box::new(BoaEngineWrapper::new(engine)))
    }

    fn engine_type(&self) -> &'static str {
        "boa"
    }
}

impl BoaEngineWrapper {
    fn boa_to_json_value(&self, js_value: thalora_browser_apis::boa_engine::JsValue) -> Result<Value> {
        if js_value.is_undefined() || js_value.is_null() {
            Ok(Value::Null)
        } else if js_value.is_boolean() {
            Ok(Value::Bool(js_value.as_boolean().unwrap_or(false)))
        } else if js_value.is_string() {
            let s = js_value.as_string().ok_or_else(|| anyhow::anyhow!("Failed to convert string"))?;
            Ok(Value::String(s.to_std_string_lossy()))
        } else if js_value.is_number() {
            let n = js_value.as_number().unwrap_or(0.0);
            if n.fract() == 0.0 && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
                Ok(Value::Number(serde_json::Number::from(n as i64)))
            } else {
                serde_json::Number::from_f64(n)
                    .map(Value::Number)
                    .ok_or_else(|| anyhow::anyhow!("Invalid number: {}", n))
            }
        } else if js_value.is_bigint() {
            // Convert BigInt to string representation
            Ok(Value::String("[BigInt]".to_string()))
        } else {
            // For complex types, try to convert to string representation
            // We need a context for to_string, so just use a simple representation
            Ok(Value::String("[Object]".to_string()))
        }
    }

    fn json_to_boa_value(&self, value: Value) -> Result<thalora_browser_apis::boa_engine::JsValue> {
        use thalora_browser_apis::boa_engine::JsValue;

        match value {
            Value::Null => Ok(JsValue::null()),
            Value::Bool(b) => Ok(JsValue::new(b)),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                        Ok(JsValue::new(i as i32))
                    } else {
                        Ok(JsValue::new(i as f64))
                    }
                } else if let Some(f) = n.as_f64() {
                    Ok(JsValue::new(f))
                } else {
                    Ok(JsValue::undefined())
                }
            }
            Value::String(s) => Ok(JsValue::new(js_string!(s))),
            Value::Array(_) | Value::Object(_) => {
                // For complex types, serialize to JSON string
                let json_str = serde_json::to_string(&value)?;
                Ok(JsValue::new(js_string!(json_str)))
            }
        }
    }
}
