use anyhow::Result;
use boa_engine::Context;
use std::sync::{Arc, Mutex};
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

        let mut context = Context::default();

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
}