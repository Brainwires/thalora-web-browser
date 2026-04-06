use crate::engine::browser::module_loader::HttpModuleLoader;
use anyhow::{Result, anyhow};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;
use thalora_browser_apis::boa_engine::{Context, JsValue, Source, js_string};

use crate::apis::polyfills::syntax_transformer::SyntaxTransformer;

#[allow(dead_code)]
pub struct JavaScriptEngine {
    context: Context,
    timers: Rc<RefCell<HashMap<u32, TimerHandle>>>,
    next_timer_id: Rc<RefCell<u32>>,
    promises: Vec<thalora_browser_apis::boa_engine::JsObject>,
    start_time: Instant,
    syntax_transformer: SyntaxTransformer,
}

#[allow(dead_code)]
#[derive(Debug)]
struct TimerHandle {
    id: u32,
    callback: JsValue,
    interval: Option<std::time::Duration>,
    created_at: Instant,
}

impl JavaScriptEngine {
    pub fn new() -> Result<Self> {
        let mut context = Context::builder()
            .module_loader(Rc::new(HttpModuleLoader::new("about:blank")))
            .build()
            .map_err(|e| anyhow!("failed to build JS context: {}", e))?;
        let timers = Rc::new(RefCell::new(HashMap::new()));
        let next_timer_id = Rc::new(RefCell::new(1));

        // Initialize all browser APIs (console, DOM, events, etc.)
        thalora_browser_apis::initialize_browser_apis(&mut context)
            .map_err(|e| anyhow!("Failed to initialize browser APIs: {}", e))?;

        // Setup additional Web APIs (credentials, service workers, etc.)
        let web_apis = crate::apis::WebApis::new();
        web_apis.setup_all_apis(&mut context)?;

        Ok(Self {
            context,
            timers,
            next_timer_id,
            promises: Vec::new(),
            start_time: Instant::now(),
            syntax_transformer: SyntaxTransformer::new(),
        })
    }

    pub async fn execute_enhanced(&mut self, code: &str) -> Result<JsValue> {
        // Pre-process the code with complete ES2025+ support
        let processed_code = self.syntax_transformer.transform_latest(code)?;
        let final_code = self
            .syntax_transformer
            .transform_modern_syntax(&processed_code)?;

        // Execute the processed code
        let result = self
            .context
            .eval(Source::from_bytes(&final_code))
            .map_err(|e| anyhow!("JavaScript execution error: {}", e))?;

        Ok(result)
    }

    /// Create a test instance for unit testing
    pub fn new_test() -> Result<Self> {
        Self::new()
    }

    pub fn get_global_object(&mut self, name: &str) -> Result<Option<JsValue>> {
        match self
            .context
            .global_object()
            .get(js_string!(name), &mut self.context)
        {
            Ok(value) if !value.is_undefined() => Ok(Some(value)),
            Ok(_) => Ok(None),
            Err(_) => Ok(None),
        }
    }

    pub fn set_global_object(&mut self, name: &str, value: JsValue) -> Result<()> {
        self.context
            .global_object()
            .set(js_string!(name), value, true, &mut self.context)
            .map_err(|e| anyhow!("Failed to set global object: {}", e))?;
        Ok(())
    }

    /// Execute JavaScript code with V8-compatible error handling
    pub async fn execute_v8_compatible(&mut self, code: &str) -> Result<JsValue> {
        // Use the enhanced ES2022 transformation pipeline
        self.execute_enhanced(code).await
    }

    /// Get engine version information
    pub fn version_info(&self) -> String {
        "Enhanced JavaScript Engine v3.0 (ES2025+ Compatible)".to_string()
    }

    /// Run pending microtasks / jobs on the engine's JS context. Useful in tests to
    /// flush promise resolution and async module loading.
    pub fn run_jobs(&mut self) -> Result<()> {
        self.context
            .run_jobs()
            .map_err(|e| anyhow!("JS job executor error: {}", e))?;
        Ok(())
    }
}
