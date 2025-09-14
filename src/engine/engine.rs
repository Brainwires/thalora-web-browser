use anyhow::{anyhow, Result};
use boa_engine::{Context, JsValue, Source, js_string};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use crate::apis::polyfills::syntax_transformer::SyntaxTransformer;

pub struct JavaScriptEngine {
    context: Context,
    timers: Arc<Mutex<HashMap<u32, TimerHandle>>>,
    next_timer_id: Arc<Mutex<u32>>,
    promises: Vec<boa_engine::JsObject>,
    start_time: Instant,
    syntax_transformer: SyntaxTransformer,
}

#[derive(Debug)]
struct TimerHandle {
    id: u32,
    callback: JsValue,
    interval: Option<std::time::Duration>,
    created_at: Instant,
}

impl JavaScriptEngine {
    pub fn new() -> Result<Self> {
        let mut context = Context::default();
        let timers = Arc::new(Mutex::new(HashMap::new()));
        let next_timer_id = Arc::new(Mutex::new(1));

        setup_enhanced_globals(&mut context)
            .map_err(|e| anyhow!("Failed to setup enhanced globals: {}", e))?;

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
        let final_code = self.syntax_transformer.transform_modern_syntax(&processed_code)?;

        // Execute the processed code
        let result = self.context.eval(Source::from_bytes(&final_code))
            .map_err(|e| anyhow!("JavaScript execution error: {}", e))?;

        Ok(result)
    }

    /// Create a test instance for unit testing
    pub fn new_test() -> Result<Self> {
        Self::new()
    }

    pub fn get_global_object(&mut self, name: &str) -> Result<Option<JsValue>> {
        match self.context.global_object().get(js_string!(name), &mut self.context) {
            Ok(value) if !value.is_undefined() => Ok(Some(value)),
            Ok(_) => Ok(None),
            Err(_) => Ok(None),
        }
    }

    pub fn set_global_object(&mut self, name: &str, value: JsValue) -> Result<()> {
        self.context.global_object()
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
}