use anyhow::Result;
use thalora_browser_apis::boa_engine::{Context, module::IdleModuleLoader};
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::collections::HashMap;
use crate::apis::WebApis;
use crate::engine::engine_trait::EngineType;
use super::layout_integration::{LayoutIntegration, ElementLayoutInfo};
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
    /// Layout integration for computing element positions
    /// This enables realistic getBoundingClientRect() values
    layout_integration: LayoutIntegration,
    /// Cached layout results for the current document
    pub(super) layout_cache: HashMap<String, ElementLayoutInfo>,
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

                // Setup dynamic script execution hooks
                // This must be called AFTER DOM is initialized so Node/Element prototypes exist
                crate::apis::polyfills::setup_dynamic_script_hooks(&mut context).unwrap();

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
                    layout_integration: LayoutIntegration::new(),
                    layout_cache: HashMap::new(),
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
                    layout_integration: LayoutIntegration::new(),
                    layout_cache: HashMap::new(),
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
    /// Also computes CSS layout for realistic getBoundingClientRect() values
    pub fn update_document_html(&mut self, html_content: &str) -> Result<()> {
        use thalora_browser_apis::boa_engine::js_string;
        // Prevent re-entrant updates which could cause infinite recursion by
        // a JS getter calling back into document update.
        if self.in_update {
            eprintln!("🔍 DEBUG: update_document_html re-entrant call detected - skipping to avoid recursion");
            return Ok(());
        }

        self.in_update = true;

        // Compute layout for realistic element positions
        // This enables getBoundingClientRect() to return real values
        match self.layout_integration.compute_layout_for_html(html_content) {
            Ok(layouts) => {
                // Convert to registry format and populate the global registry
                let registry_layouts: std::collections::HashMap<String, thalora_browser_apis::layout_registry::ComputedLayout> =
                    layouts.iter().map(|(id, info)| {
                        (id.clone(), thalora_browser_apis::layout_registry::ComputedLayout {
                            x: info.x,
                            y: info.y,
                            width: info.width,
                            height: info.height,
                            top: info.top,
                            right: info.right,
                            bottom: info.bottom,
                            left: info.left,
                        })
                    }).collect();

                // Populate the global layout registry for DOM element access
                thalora_browser_apis::layout_registry::set_layouts(registry_layouts);

                self.layout_cache = layouts;
                eprintln!("🔍 DEBUG: Computed layout for {} elements", self.layout_cache.len());
            }
            Err(e) => {
                eprintln!("🔍 DEBUG: Layout computation failed (non-fatal): {}", e);
                // Continue even if layout fails - DOM still needs to be updated
            }
        }

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

    /// Register a script that has been loaded/executed
    /// This makes the script visible in document.scripts and getElementsByTagName("script")
    pub fn register_script(&mut self, entry: thalora_browser_apis::dom::document::ScriptEntry) -> Result<()> {
        use thalora_browser_apis::boa_engine::js_string;

        match self.engine_type {
            EngineType::Boa => {
                if let Some(ctx) = &mut self.js_context {
                    let global = ctx.global_object().clone();
                    if let Ok(document_value) = global.get(js_string!("document"), ctx) {
                        if let Some(document_obj) = document_value.as_object() {
                            if let Some(document_data) = document_obj.downcast_ref::<thalora_browser_apis::dom::document::DocumentData>() {
                                document_data.register_script(entry);
                            }
                        }
                    }
                }
            }
            EngineType::V8 => {
                // TODO: Implement V8 script registration
            }
        }

        Ok(())
    }

    /// Set the currently executing script for document.currentScript
    /// This creates an HTMLScriptElement with the given attributes and sets it as __currentScript__
    pub fn set_current_script(&mut self, entry: &thalora_browser_apis::dom::document::ScriptEntry) -> Result<()> {
        use thalora_browser_apis::boa_engine::js_string;

        match self.engine_type {
            EngineType::Boa => {
                if let Some(ctx) = &mut self.js_context {
                    // Create an HTMLScriptElement for the current script
                    let script_constructor = ctx.intrinsics().constructors().html_script_element().constructor();
                    if let Ok(script_obj) = script_constructor.construct(&[], None, ctx) {
                        // Set all the script attributes
                        if let Some(script_data) = script_obj.downcast_ref::<thalora_browser_apis::dom::html_script_element::HTMLScriptElementData>() {
                            if let Some(ref src) = entry.src {
                                script_data.set_src(src.clone());
                            }
                            if let Some(ref type_) = entry.script_type {
                                script_data.set_type(type_.clone());
                            }
                            script_data.set_async(entry.async_);
                            script_data.set_defer(entry.defer);
                            script_data.set_text(entry.text.clone());

                            // Set all custom attributes (including data-* attributes)
                            for (key, value) in &entry.attributes {
                                script_data.set_attribute(key, value.clone());
                            }
                        }

                        // Set __currentScript__ on global object
                        let global = ctx.global_object().clone();
                        let script_value: thalora_browser_apis::boa_engine::JsValue = script_obj.into();
                        if let Err(e) = global.set(js_string!("__currentScript__"), script_value, false, ctx) {
                            return Err(anyhow::anyhow!("Failed to set __currentScript__: {:?}", e));
                        }
                        eprintln!("🔍 DEBUG: Set __currentScript__ for src={:?}", entry.src);
                    }
                }
            }
            EngineType::V8 => {
                // TODO: Implement V8 currentScript
            }
        }

        Ok(())
    }

    /// Clear the currently executing script (set document.currentScript to null)
    pub fn clear_current_script(&mut self) -> Result<()> {
        use thalora_browser_apis::boa_engine::{js_string, JsValue};

        match self.engine_type {
            EngineType::Boa => {
                if let Some(ctx) = &mut self.js_context {
                    let global = ctx.global_object().clone();
                    if let Err(e) = global.set(js_string!("__currentScript__"), JsValue::null(), false, ctx) {
                        return Err(anyhow::anyhow!("Failed to clear __currentScript__: {:?}", e));
                    }
                }
            }
            EngineType::V8 => {
                // TODO: Implement V8 currentScript
            }
        }

        Ok(())
    }

    /// Get the computed layout for an element by ID or selector
    /// Returns (x, y, width, height, top, right, bottom, left) or None if not found
    pub fn get_element_layout(&self, element_id: &str) -> Option<(f64, f64, f64, f64, f64, f64, f64, f64)> {
        self.layout_cache.get(element_id).map(|info| {
            (info.x, info.y, info.width, info.height, info.top, info.right, info.bottom, info.left)
        })
    }

    /// Get all computed layouts
    pub fn get_all_layouts(&self) -> &HashMap<String, ElementLayoutInfo> {
        &self.layout_cache
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