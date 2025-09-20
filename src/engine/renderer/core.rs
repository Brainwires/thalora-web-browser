use anyhow::Result;
use boa_engine::{Context, module::IdleModuleLoader};
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use crate::apis::WebApis;
use crate::apis::events::EventManager;
use crate::features::AdvancedWebAssemblyEngine;

pub struct RustRenderer {
    pub(super) js_context: Context,
    pub(super) web_apis: WebApis,
    pub(super) history_initialized: bool,
    pub(super) event_manager: EventManager,
    pub(super) wasm_api: Option<AdvancedWebAssemblyEngine>,
    // Guard to prevent re-entrant updates/evaluations which previously caused
    // infinite recursion / stack overflows when JS evaluation or window getters
    // triggered additional document updates.
    pub(super) in_update: bool,
}

impl RustRenderer {
    pub fn new() -> Self {

        let mut context = Context::builder()
            .module_loader(Rc::new(IdleModuleLoader))
            .build()
            .expect("failed to build JS context");

        let web_apis = WebApis::new();

        let event_manager = EventManager::new();

        // DOM is now natively handled by Boa engine (Document, Element, etc.)

        // Setup polyfills (now excludes DOM globals which are native)
        crate::apis::polyfills::setup_all_polyfills(&mut context).unwrap();

        // Setup Web APIs polyfills (requires window and console to be defined)
        web_apis.setup_all_apis(&mut context).unwrap();

        // Setup native DOM globals (Document, Window, History, PageSwapEvent) - after builtins are initialized
        crate::apis::dom_native::setup_native_dom_globals(&mut context).unwrap();

        // Setup REAL DOM event system (replaces mock implementations)
        event_manager.setup_events_api(&mut context).unwrap();

        // Setup REAL WebAssembly API (replaces mock implementations)
        let wasm_api = match AdvancedWebAssemblyEngine::new() {
            Ok(api) => {
                api.setup_webassembly_api(&mut context).unwrap();
                Some(api)
            },
            Err(_e) => None
        };

        Self {
            js_context: context,
            web_apis,
            history_initialized: false,
            event_manager,
            wasm_api,
            in_update: false,
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

    pub fn js_value_to_string(&mut self, value: boa_engine::JsValue) -> String {
        value.to_string(&mut self.js_context)
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_else(|_| "undefined".to_string())
    }

    /// Update the document's HTML content to enable real DOM querying
    pub fn update_document_html(&mut self, html_content: &str) -> Result<()> {
        use boa_engine::js_string;
        // Prevent re-entrant updates which could cause infinite recursion by
        // a JS getter calling back into document update.
        if self.in_update {
            eprintln!("🔍 DEBUG: update_document_html re-entrant call detected - skipping to avoid recursion");
            return Ok(());
        }

        self.in_update = true;
        // Get the global document object
        let global = self.js_context.global_object().clone();
        if let Ok(document_value) = global.get(js_string!("document"), &mut self.js_context) {
            if let Some(document_obj) = document_value.as_object() {
                // Check if this is a Document object with our DocumentData
                if let Some(document_data) = document_obj.downcast_ref::<boa_engine::builtins::document::DocumentData>() {
                    document_data.set_html_content(html_content);
                    document_data.set_ready_state("complete");
                }
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

        match self.js_context.eval(boa_engine::Source::from_bytes(debug_script)) {
            Ok(value) => Ok(self.js_value_to_string(value)),
            Err(_) => Ok("Failed to get debug info".to_string())
        }
    }
}