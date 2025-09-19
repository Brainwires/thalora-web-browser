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
        }
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

        Ok(())
    }
}