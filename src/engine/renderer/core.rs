use anyhow::Result;
use thalora_browser_apis::boa_engine::{Context, module::IdleModuleLoader};
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use crate::apis::WebApis;
use crate::engine::engine_trait::EngineType;
// events API is now natively implemented in Boa engine
// WebAssembly is now natively implemented in Boa engine

#[allow(dead_code)]
pub struct RustRenderer {
    pub(super) js_context: Option<Context>,
    pub(super) v8_engine: Option<Box<dyn crate::engine::engine_trait::ThaloraBrowserEngine>>,
    pub(super) engine_type: EngineType,
    pub(super) web_apis: WebApis,
    pub(super) history_initialized: bool,
    // event manager is now handled by Boa engine
    // WebAssembly API is now natively implemented in Boa engine
    // Guard to prevent re-entrant updates/evaluations which previously caused
    // infinite recursion / stack overflows when JS evaluation or window getters
    // triggered additional document updates.
    pub(super) in_update: bool,
}

impl RustRenderer {
    pub fn new() -> Self {
        Self::new_with_engine(EngineType::Boa)
    }

    pub fn new_with_engine(engine_type: EngineType) -> Self {
        match engine_type {
            EngineType::Boa => {
                let mut context = Context::builder()
                    .module_loader(Rc::new(IdleModuleLoader))
                    .build()
                    .expect("failed to build JS context");

                let web_apis = WebApis::new();

                // CRITICAL: Initialize browser APIs from Boa engine FIRST
                // This registers all the intrinsics (Document, Window, Navigator, etc.)
                // and creates global instances. This replaces setup_native_dom_globals()
                // which was causing duplicate registrations.
                thalora_browser_apis::initialize_browser_apis(&mut context).unwrap();

                // Setup polyfills (now excludes DOM globals which are native)
                crate::apis::polyfills::setup_all_polyfills(&mut context).unwrap();

                // Setup Web APIs polyfills (requires window and console to be defined)
                web_apis.setup_all_apis(&mut context).unwrap();

                // NOTE: setup_native_dom_globals() is NOT called here anymore because
                // initialize_browser_apis() already creates all the global instances.
                // Calling both would cause duplicate property registrations and assertion failures.

                Self {
                    js_context: Some(context),
                    v8_engine: None,
                    engine_type: EngineType::Boa,
                    web_apis,
                    history_initialized: false,
                    in_update: false,
                }
            }
            EngineType::V8 => {
                let v8_engine = crate::engine::engine_trait::EngineFactory::create_engine(EngineType::V8)
                    .expect("Failed to create V8 engine");

                let web_apis = WebApis::new();

                Self {
                    js_context: None,
                    v8_engine: Some(v8_engine),
                    engine_type: EngineType::V8,
                    web_apis,
                    history_initialized: false,
                    in_update: false,
                }
            }
        }
    }

    /// Check whether the renderer is currently performing an update.
    pub fn is_in_update(&self) -> bool {
        self.in_update
    }

    /// Set the renderer "in update" guard flag.
    pub fn set_in_update(&mut self, value: bool) {
        self.in_update = value;
    }

    pub fn setup_history_api(&mut self, _browser: Arc<Mutex<crate::engine::browser::HeadlessWebBrowser>>) -> Result<()> {
        // History is now natively implemented in Boa engine, no additional setup needed
        self.history_initialized = true;
        Ok(())
    }

    pub fn js_value_to_string(&mut self, value: thalora_browser_apis::boa_engine::JsValue) -> String {
        if let Some(ctx) = &mut self.js_context {
            value.to_string(ctx)
                .map(|s| s.to_std_string_escaped())
                .unwrap_or_else(|_| "undefined".to_string())
        } else {
            "undefined".to_string()
        }
    }

    /// Evaluate JavaScript code and return the result as JSON Value
    pub fn eval_js_json(&mut self, source: &str) -> Result<serde_json::Value> {
        match self.engine_type {
            EngineType::Boa => {
                if let Some(ctx) = &mut self.js_context {
                    let result = ctx.eval(thalora_browser_apis::boa_engine::Source::from_bytes(source))
                        .map_err(|e| anyhow::Error::msg(format!("JavaScript evaluation failed: {:?}", e)))?;

                    // Convert Boa JsValue to JSON
                    self.boa_value_to_json(result)
                } else {
                    Err(anyhow::Error::msg("Boa context not available"))
                }
            }
            EngineType::V8 => {
                if let Some(engine) = &mut self.v8_engine {
                    engine.execute(source)
                } else {
                    Err(anyhow::Error::msg("V8 engine not available"))
                }
            }
        }
    }

    /// Evaluate JavaScript code and return the result (Boa-specific)
    pub fn eval_js(&mut self, source: &str) -> Result<thalora_browser_apis::boa_engine::JsValue> {
        if let Some(ctx) = &mut self.js_context {
            ctx.eval(thalora_browser_apis::boa_engine::Source::from_bytes(source))
                .map_err(|e| anyhow::Error::msg(format!("JavaScript evaluation failed: {:?}", e)))
        } else {
            Err(anyhow::Error::msg("Boa context not available"))
        }
    }

    /// Helper to convert Boa JsValue to serde_json::Value
    fn boa_value_to_json(&self, value: thalora_browser_apis::boa_engine::JsValue) -> Result<serde_json::Value> {
        if value.is_undefined() || value.is_null() {
            Ok(serde_json::Value::Null)
        } else if value.is_boolean() {
            Ok(serde_json::Value::Bool(value.as_boolean().unwrap_or(false)))
        } else if value.is_string() {
            let s = value.as_string().ok_or_else(|| anyhow::Error::msg("Failed to convert string"))?;
            Ok(serde_json::Value::String(s.to_std_string_lossy()))
        } else if value.is_number() {
            let n = value.as_number().unwrap_or(0.0);
            if let Some(num) = serde_json::Number::from_f64(n) {
                Ok(serde_json::Value::Number(num))
            } else {
                Ok(serde_json::Value::Null)
            }
        } else {
            Ok(serde_json::Value::String("[Object]".to_string()))
        }
    }

    /// Update the document's HTML content to enable real DOM querying
    pub fn update_document_html(&mut self, html_content: &str) -> Result<()> {
        use thalora_browser_apis::boa_engine::js_string;
        // Prevent re-entrant updates which could cause infinite recursion by
        // a JS getter calling back into document update.
        if self.in_update {
            eprintln!("🔍 DEBUG: update_document_html re-entrant call detected - skipping to avoid recursion");
            return Ok(());
        }

        self.in_update = true;

        match self.engine_type {
            EngineType::Boa => {
                if let Some(ctx) = &mut self.js_context {
                    // Get the global document object
                    let global = ctx.global_object().clone();
                    if let Ok(document_value) = global.get(js_string!("document"), ctx) {
                        if let Some(document_obj) = document_value.as_object() {
                            // Check if this is a Document object with our DocumentData
                            if let Some(document_data) = document_obj.downcast_ref::<thalora_browser_apis::dom::document::DocumentData>() {
                                document_data.set_html_content(html_content);
                                document_data.set_ready_state("complete");
                            }
                        }
                    }
                }
            }
            EngineType::V8 => {
                // V8 DOM handling - for now, just acknowledge the HTML content
                // TODO: Implement V8 DOM manipulation
            }
        }

        self.in_update = false;

        Ok(())
    }

    /// TEMPORARY: Get debugging information from Bing debug polyfill
    pub fn get_bing_debug_info(&mut self) -> Result<String> {
        let debug_script = r#"
            (function() {
                try {
                    if (typeof window._BING_DEBUG !== 'undefined') {
                        return JSON.stringify(window._BING_DEBUG.report(), null, 2);
                    } else {
                        return 'No debug info available';
                    }
                } catch(e) {
                    return 'Error getting debug info: ' + e.message;
                }
            })()
        "#;

        if let Some(ctx) = &mut self.js_context {
            match ctx.eval(thalora_browser_apis::boa_engine::Source::from_bytes(debug_script)) {
                Ok(value) => Ok(self.js_value_to_string(value)),
                Err(_) => Ok("Failed to get debug info".to_string())
            }
        } else {
            Ok("JavaScript context not available".to_string())
        }
    }
}