//! HTTP-based ES Module Loader for Boa JS Engine
//!
//! Implements the `ModuleLoader` trait to fetch ES modules over HTTP,
//! enabling `<script type="module">` and dynamic `import()` support.
//! Uses `block_on_compat` + `get_shared_client()` from `http_blocking.rs`
//! to perform synchronous HTTP fetches compatible with Boa's executor.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use boa_engine::module::{Module, ModuleLoader, ModuleRequest, Referrer};
use boa_engine::{Context, JsNativeError, JsResult, JsString, Source};

use crate::http_blocking::{block_on_compat, get_shared_client};

/// HTTP-based module loader that fetches ES modules over HTTP/HTTPS.
///
/// URL strings are stored as `PathBuf` because Boa uses `Path` for module identity.
/// HTTP fetch happens synchronously via `block_on_compat` (spawns OS thread with tokio),
/// compatible with Boa's `futures_lite::block_on` executor.
pub struct HttpModuleLoader {
    /// Base URL of the current page (used for resolving relative imports)
    base_url: RefCell<String>,
    /// Cache of already-parsed modules keyed by resolved URL
    module_cache: RefCell<HashMap<String, Module>>,
}

impl HttpModuleLoader {
    /// Create a new HTTP module loader with the given base URL.
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: RefCell::new(base_url.to_string()),
            module_cache: RefCell::new(HashMap::new()),
        }
    }

    /// Update the base URL (called when navigating to a new page).
    pub fn set_base_url(&self, url: &str) {
        *self.base_url.borrow_mut() = url.to_string();
    }

    /// Resolve a module specifier to an absolute URL.
    ///
    /// Handles:
    /// - Absolute URLs (`https://...`, `http://...`)
    /// - Protocol-relative URLs (`//cdn.example.com/...`)
    /// - Root-relative paths (`/path/to/module.js`)
    /// - Relative paths (`./foo.js`, `../lib.js`)
    ///
    /// Returns error for bare specifiers (`lodash`, `vue`) since we don't
    /// support import maps yet.
    fn resolve_specifier(&self, specifier: &str, referrer_url: Option<&str>) -> JsResult<String> {
        // Already an absolute URL
        if specifier.starts_with("http://") || specifier.starts_with("https://") {
            return Ok(specifier.to_string());
        }

        // Protocol-relative URL
        if specifier.starts_with("//") {
            return Ok(format!("https:{}", specifier));
        }

        // Relative or root-relative path — resolve against referrer or base URL
        if specifier.starts_with("./") || specifier.starts_with("../") || specifier.starts_with('/') {
            let base_url_borrowed = self.base_url.borrow();
            let base = referrer_url
                .unwrap_or(&base_url_borrowed);

            let base_parsed = url::Url::parse(base).map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("Invalid base URL '{}': {}", base, e))
            })?;

            let resolved = base_parsed.join(specifier).map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("Failed to resolve '{}' against '{}': {}", specifier, base, e))
            })?;

            return Ok(resolved.to_string());
        }

        // Bare specifier (e.g., "vue", "lodash") — not supported without import maps
        Err(JsNativeError::typ()
            .with_message(format!(
                "Cannot resolve bare module specifier '{}'. Relative imports must start with './', '../', or '/'. Import maps are not yet supported.",
                specifier
            ))
            .into())
    }

    /// Fetch module source code from a URL over HTTP.
    fn fetch_module_source(&self, url: &str) -> JsResult<String> {
        eprintln!("MODULE: Fetching module from: {}", url);

        let url_owned = url.to_string();
        let client = get_shared_client();

        let result = block_on_compat(async move {
            let response = client
                .get(&url_owned)
                .send()
                .await
                .map_err(|e| format!("HTTP request failed for '{}': {}", url_owned, e))?;

            let status = response.status();
            if !status.is_success() {
                return Err(format!(
                    "Module fetch failed for '{}': HTTP {}",
                    url_owned, status
                ));
            }

            response
                .text()
                .await
                .map_err(|e| format!("Failed to read module body from '{}': {}", url_owned, e))
        });

        match result {
            Ok(source) => {
                eprintln!("MODULE: Successfully fetched {} bytes from {}", source.len(), url);
                Ok(source)
            }
            Err(msg) => {
                eprintln!("MODULE: Fetch error: {}", msg);
                Err(JsNativeError::typ().with_message(msg).into())
            }
        }
    }
}

impl ModuleLoader for HttpModuleLoader {
    async fn load_imported_module(
        self: Rc<Self>,
        referrer: Referrer,
        request: ModuleRequest,
        context: &RefCell<&mut Context>,
    ) -> JsResult<Module> {
        let specifier = request.specifier().to_std_string_escaped();

        // Determine referrer URL from the referrer's path (which we store as URL strings)
        let referrer_url = referrer.path().and_then(|p| {
            p.to_str().map(String::from)
        });

        // Resolve the specifier to an absolute URL
        let resolved_url = self.resolve_specifier(&specifier, referrer_url.as_deref())?;

        // Check cache first
        {
            let cache = self.module_cache.borrow();
            if let Some(cached) = cache.get(&resolved_url) {
                eprintln!("MODULE: Cache hit for {}", resolved_url);
                return Ok(cached.clone());
            }
        }

        // Fetch module source over HTTP
        let source_code = self.fetch_module_source(&resolved_url)?;

        // Parse the module — use the URL as the path so it becomes the referrer
        // for any nested imports from this module
        let path = PathBuf::from(&resolved_url);
        let source = Source::from_bytes(&source_code).with_path(&path);

        let module = {
            let mut ctx = context.borrow_mut();
            Module::parse(source, None, &mut ctx)?
        };

        // Cache the parsed module
        self.module_cache
            .borrow_mut()
            .insert(resolved_url.clone(), module.clone());

        eprintln!("MODULE: Successfully parsed and cached module: {}", resolved_url);

        Ok(module)
    }

    fn init_import_meta(
        self: Rc<Self>,
        import_meta: &boa_engine::JsObject,
        module: &Module,
        context: &mut Context,
    ) {
        // Set import.meta.url to the module's URL
        if let Some(path) = module.path() {
            if let Some(url_str) = path.to_str() {
                let _ = import_meta.set(
                    boa_engine::js_string!("url"),
                    JsString::from(url_str),
                    false,
                    context,
                );
            }
        }
    }
}
