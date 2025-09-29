use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::engine::V8JavaScriptEngine;

/// V8 Runtime wrapper that provides higher-level runtime management
/// Similar to deno_core but simpler and tailored for Thalora
pub struct V8Runtime {
    engine: Arc<Mutex<V8JavaScriptEngine>>,
}

impl V8Runtime {
    /// Create a new V8 runtime instance
    pub fn new() -> Result<Self> {
        let engine = V8JavaScriptEngine::new()?;
        Ok(Self {
            engine: Arc::new(Mutex::new(engine)),
        })
    }

    /// Execute JavaScript code in the runtime
    pub async fn execute(&self, code: &str) -> Result<serde_json::Value> {
        let mut engine = self.engine.lock().await;
        engine.execute(code).await
    }

    /// Execute JavaScript code with enhanced features
    pub async fn execute_enhanced(&self, code: &str) -> Result<serde_json::Value> {
        let mut engine = self.engine.lock().await;
        engine.execute_enhanced(code).await
    }

    /// Get version information
    pub async fn version_info(&self) -> String {
        let engine = self.engine.lock().await;
        engine.version_info()
    }

    /// Run pending microtasks
    pub async fn run_jobs(&self) -> Result<()> {
        let mut engine = self.engine.lock().await;
        engine.run_jobs()
    }

    /// Get global object
    pub async fn get_global_object(&self, name: &str) -> Result<Option<serde_json::Value>> {
        let mut engine = self.engine.lock().await;
        engine.get_global_object(name)
    }

    /// Set global object
    pub async fn set_global_object(&self, name: &str, value: serde_json::Value) -> Result<()> {
        let mut engine = self.engine.lock().await;
        engine.set_global_object(name, value)
    }
}