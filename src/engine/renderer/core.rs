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
        if std::env::var("THALORA_SILENT").is_err() {
            eprintln!("🔧 RustRenderer::new() - Starting initialization");
        }

        let mut context = Context::default();
        if std::env::var("THALORA_SILENT").is_err() {
            eprintln!("🔧 RustRenderer::new() - Created Boa context");
        }

        let web_apis = WebApis::new();
        if std::env::var("THALORA_SILENT").is_err() {
            eprintln!("🔧 RustRenderer::new() - Created WebApis");
        }

        let event_manager = EventManager::new();
        if std::env::var("THALORA_SILENT").is_err() {
            eprintln!("🔧 RustRenderer::new() - Created EventManager");
        }

        // DOM is now natively handled by Boa engine (Document, Element, etc.)
        if std::env::var("THALORA_SILENT").is_err() {
            eprintln!("🔧 RustRenderer::new() - DOM is natively handled by Boa engine");
        }

        // Setup polyfills (now excludes DOM globals which are native)
        if std::env::var("THALORA_SILENT").is_err() {
            eprintln!("🔧 RustRenderer::new() - Setting up polyfills");
        }
        crate::apis::polyfills::setup_all_polyfills(&mut context).unwrap();
        if std::env::var("THALORA_SILENT").is_err() {
            eprintln!("🔧 RustRenderer::new() - Polyfills setup complete");
        }

        // Setup Web APIs polyfills (requires window and console to be defined)
        if std::env::var("THALORA_SILENT").is_err() {
            eprintln!("🔧 RustRenderer::new() - Setting up Web APIs");
        }
        web_apis.setup_all_apis(&mut context).unwrap();
        if std::env::var("THALORA_SILENT").is_err() {
            eprintln!("🔧 RustRenderer::new() - Web APIs setup complete");
        }

        // Setup native DOM globals (Document, Window, History, PageSwapEvent) - after builtins are initialized
        if std::env::var("THALORA_SILENT").is_err() {
            eprintln!("🔧 RustRenderer::new() - Setting up native DOM globals");
        }
        crate::apis::dom_native::setup_native_dom_globals(&mut context).unwrap();
        if std::env::var("THALORA_SILENT").is_err() {
            eprintln!("🔧 RustRenderer::new() - Native DOM globals setup complete");
        }

        // Setup REAL DOM event system (replaces mock implementations)
        if std::env::var("THALORA_SILENT").is_err() {
            eprintln!("🔧 RustRenderer::new() - Setting up DOM events");
        }
        event_manager.setup_events_api(&mut context).unwrap();
        if std::env::var("THALORA_SILENT").is_err() {
            eprintln!("🔧 RustRenderer::new() - DOM events setup complete");
        }

        // Setup REAL WebAssembly API (replaces mock implementations)
        if std::env::var("THALORA_SILENT").is_err() {
            eprintln!("🔧 RustRenderer::new() - Setting up WebAssembly API");
        }
        let wasm_api = match AdvancedWebAssemblyEngine::new() {
            Ok(api) => {
                if std::env::var("THALORA_SILENT").is_err() {
                    eprintln!("🔧 RustRenderer::new() - WebAssembly engine created");
                }
                api.setup_webassembly_api(&mut context).unwrap();
                if std::env::var("THALORA_SILENT").is_err() {
                    eprintln!("🔧 RustRenderer::new() - WebAssembly API setup complete");
                }
                Some(api)
            },
            Err(e) => {
                if std::env::var("THALORA_SILENT").is_err() {
                    eprintln!("Failed to initialize WebAssembly API: {}", e);
                }
                None
            }
        };

        if std::env::var("THALORA_SILENT").is_err() {
            eprintln!("🔧 RustRenderer::new() - Initialization complete");
        }
        Self {
            js_context: context,
            web_apis,
            history_initialized: false,
            event_manager,
            wasm_api,
        }
    }

    pub fn setup_history_api(&mut self, _browser: Arc<Mutex<crate::engine::browser::HeadlessWebBrowser>>) -> Result<()> {
        if std::env::var("THALORA_SILENT").is_err() {
            eprintln!("🔧 RustRenderer::setup_history_api() - History is now natively handled by Boa");
        }
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