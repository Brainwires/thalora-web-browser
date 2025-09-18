use anyhow::Result;
use boa_engine::Context;
use std::sync::{Arc, Mutex};
use crate::apis::WebApis;
use crate::engine::dom::EnhancedDom;
use crate::apis::events::EventManager;
use crate::features::AdvancedWebAssemblyEngine;

pub struct RustRenderer {
    pub(super) js_context: Context,
    pub(super) web_apis: WebApis,
    pub(super) dom_manager: Option<EnhancedDom>,
    pub(super) history_initialized: bool,
    pub(super) event_manager: EventManager,
    pub(super) wasm_api: Option<AdvancedWebAssemblyEngine>,
}

impl RustRenderer {
    pub fn new() -> Self {
        let mut context = Context::default();
        let web_apis = WebApis::new();
        let event_manager = EventManager::new();

        // Setup DOM polyfills first (provides window, document, etc.)
        // Setup DOM with EnhancedDom
        let dom_manager = match EnhancedDom::new("") {
            Ok(dom) => Some(dom),
            Err(e) => {
                eprintln!("Failed to initialize EnhancedDom: {}", e);
                None
            }
        };
        // dom_manager.setup_dom_globals(&mut context).unwrap();

        // Setup polyfills first (includes console)
        crate::apis::polyfills::setup_all_polyfills(&mut context).unwrap();

        // Setup Web APIs polyfills (requires window and console to be defined)
        web_apis.setup_all_apis(&mut context).unwrap();

        // Setup REAL DOM event system (replaces mock implementations)
        event_manager.setup_events_api(&mut context).unwrap();

        // Setup REAL WebAssembly API (replaces mock implementations)
        let wasm_api = match AdvancedWebAssemblyEngine::new() {
            Ok(api) => {
                api.setup_webassembly_api(&mut context).unwrap();
                Some(api)
            },
            Err(e) => {
                eprintln!("Failed to initialize WebAssembly API: {}", e);
                None
            }
        };

        Self {
            js_context: context,
            web_apis,
            dom_manager,
            history_initialized: false,
            event_manager,
            wasm_api,
        }
    }

    pub fn setup_history_api(&mut self, browser: Arc<Mutex<crate::engine::browser::HeadlessWebBrowser>>) -> Result<()> {
        if !self.history_initialized {
            crate::apis::history::setup_real_history(&mut self.js_context, browser)?;
            self.history_initialized = true;
        }
        Ok(())
    }

    pub fn js_value_to_string(&mut self, value: boa_engine::JsValue) -> String {
        value.to_string(&mut self.js_context)
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_else(|_| "undefined".to_string())
    }
}