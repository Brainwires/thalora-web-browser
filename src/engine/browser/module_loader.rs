//! HTTP-based ES Module Loader for the browser engine.
//!
//! Implements Boa's `ModuleLoader` trait to fetch ES modules over HTTP,
//! enabling `<script type="module">` and dynamic `import()` to work with
//! real web URLs.
//!
//! URL resolution follows the WHATWG URL spec — does NOT use Boa's
//! `resolve_module_specifier()` which assumes filesystem paths.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use thalora_browser_apis::boa_engine::{
    Context, JsError, JsNativeError, JsResult, JsString, Source, js_string,
    module::{Module, ModuleLoader, ModuleRequest, Referrer},
    object::JsObject,
};

/// HTTP-based module loader that fetches ES modules from URLs.
///
/// Resolves module specifiers against the current page URL and caches
/// fetched modules by their resolved URL.
#[derive(Debug)]
pub struct HttpModuleLoader {
    /// Current page URL used as base for relative module resolution.
    base_url: RefCell<String>,
    /// Module cache keyed by resolved URL string.
    cache: RefCell<HashMap<String, Module>>,
}

impl HttpModuleLoader {
    /// Create a new HTTP module loader with the given base URL.
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: RefCell::new(base_url.to_string()),
            cache: RefCell::new(HashMap::new()),
        }
    }

    /// Update the base URL (called when page navigates).
    pub fn set_base_url(&self, url: &str) {
        *self.base_url.borrow_mut() = url.to_string();
    }

    /// Resolve a module specifier to an absolute URL (public for testing).
    pub fn resolve_url_pub(
        &self,
        specifier: &str,
        referrer_path: Option<&Path>,
    ) -> Result<String, String> {
        self.resolve_url(specifier, referrer_path)
            .map_err(|e| format!("{}", e))
    }

    /// Resolve a module specifier to an absolute URL.
    ///
    /// - Absolute URLs (`http://`, `https://`) → used as-is
    /// - Relative (`./`, `../`) → resolved against referrer or base URL
    /// - Root-relative (`/`) → resolved against base URL origin
    /// - Bare specifiers (`lodash`, `react`) → tried via esm.sh CDN
    fn resolve_url(&self, specifier: &str, referrer_path: Option<&Path>) -> JsResult<String> {
        // Absolute URL — use directly
        if specifier.starts_with("http://") || specifier.starts_with("https://") {
            return Ok(specifier.to_string());
        }

        // Determine base for resolution: prefer referrer URL, fall back to page base
        let base_str = if let Some(path) = referrer_path {
            let p = path.to_string_lossy();
            if p.starts_with("http://") || p.starts_with("https://") {
                p.to_string()
            } else {
                self.base_url.borrow().clone()
            }
        } else {
            self.base_url.borrow().clone()
        };

        // Relative specifier (./foo, ../bar)
        if specifier.starts_with("./") || specifier.starts_with("../") {
            let base = url::Url::parse(&base_str).map_err(|e| {
                JsNativeError::typ().with_message(format!("Invalid base URL '{}': {}", base_str, e))
            })?;
            let resolved = base.join(specifier).map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("Failed to resolve '{}': {}", specifier, e))
            })?;
            return Ok(resolved.to_string());
        }

        // Root-relative specifier (/foo/bar.js)
        if specifier.starts_with('/') {
            let base = url::Url::parse(&base_str).map_err(|e| {
                JsNativeError::typ().with_message(format!("Invalid base URL '{}': {}", base_str, e))
            })?;
            let origin = base.origin().unicode_serialization();
            return Ok(format!("{}{}", origin, specifier));
        }

        // Bare specifier — try esm.sh CDN as a fallback for npm-style imports
        Ok(format!("https://esm.sh/{}", specifier))
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

        // Resolve to absolute URL
        let resolved_url = self.resolve_url(&specifier, referrer.path())?;

        // Check cache
        if let Some(module) = self.cache.borrow().get(&resolved_url) {
            return Ok(module.clone());
        }

        // HTTP fetch — create client and fetch module source
        // The ModuleLoader trait is async, so we can await directly.
        // However, Boa runs futures via its own single-threaded executor (run_jobs),
        // which doesn't have a tokio runtime. Use reqwest::blocking instead.
        let source_text = {
            let client = reqwest::blocking::Client::builder()
                .user_agent(thalora_constants::USER_AGENT)
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| {
                    JsNativeError::typ()
                        .with_message(format!("Failed to create HTTP client: {}", e))
                })?;

            let response = client.get(&resolved_url).send().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("Failed to fetch module '{}': {}", resolved_url, e))
            })?;

            if !response.status().is_success() {
                return Err(JsNativeError::typ()
                    .with_message(format!(
                        "Module fetch failed with status {}: {}",
                        response.status(),
                        resolved_url
                    ))
                    .into());
            }

            response.text().map_err(|e| {
                JsNativeError::typ().with_message(format!(
                    "Failed to read module body '{}': {}",
                    resolved_url, e
                ))
            })?
        };

        // Check if this is a JSON module (via import attribute)
        let is_json = request
            .get_attribute("type")
            .map(|v| v.to_std_string_escaped() == "json")
            .unwrap_or(false)
            || resolved_url.ends_with(".json");

        // Parse the module
        let module = if is_json {
            // JSON module
            let json_str = JsString::from(source_text.as_str());
            Module::parse_json(json_str, &mut context.borrow_mut()).map_err(|e| {
                JsNativeError::syntax().with_message(format!(
                    "JSON module parse error for '{}': {}",
                    resolved_url, e
                ))
            })?
        } else {
            // JavaScript module — use URL-as-PathBuf for referrer resolution
            let path = PathBuf::from(&resolved_url);
            let src = Source::from_bytes(source_text.as_bytes()).with_path(&path);
            Module::parse(src, None, &mut context.borrow_mut()).map_err(|e| {
                JsNativeError::syntax()
                    .with_message(format!("Module parse error for '{}': {}", resolved_url, e))
            })?
        };

        // Cache by URL
        self.cache
            .borrow_mut()
            .insert(resolved_url.clone(), module.clone());

        Ok(module)
    }

    fn init_import_meta(
        self: Rc<Self>,
        import_meta: &JsObject,
        module: &Module,
        context: &mut Context,
    ) {
        // Set import.meta.url to the module's resolved URL
        if let Some(path) = module.path() {
            let url = path.to_string_lossy().to_string();
            let _ = import_meta.set(
                js_string!("url"),
                JsString::from(url.as_str()),
                false,
                context,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_absolute_url() {
        let loader = HttpModuleLoader::new("https://example.com/page");
        let result = loader.resolve_url("https://cdn.example.com/lib.js", None);
        assert_eq!(result.unwrap(), "https://cdn.example.com/lib.js");
    }

    #[test]
    fn test_resolve_relative_url() {
        let loader = HttpModuleLoader::new("https://example.com/app/index.html");
        let result = loader.resolve_url("./utils.js", None);
        assert_eq!(result.unwrap(), "https://example.com/app/utils.js");
    }

    #[test]
    fn test_resolve_parent_relative_url() {
        let loader = HttpModuleLoader::new("https://example.com/app/deep/index.html");
        let result = loader.resolve_url("../shared.js", None);
        assert_eq!(result.unwrap(), "https://example.com/app/shared.js");
    }

    #[test]
    fn test_resolve_root_relative_url() {
        let loader = HttpModuleLoader::new("https://example.com/app/page");
        let result = loader.resolve_url("/static/module.js", None);
        assert_eq!(result.unwrap(), "https://example.com/static/module.js");
    }

    #[test]
    fn test_resolve_bare_specifier() {
        let loader = HttpModuleLoader::new("https://example.com/page");
        let result = loader.resolve_url("lodash-es", None);
        assert_eq!(result.unwrap(), "https://esm.sh/lodash-es");
    }

    #[test]
    fn test_resolve_with_referrer_path() {
        let loader = HttpModuleLoader::new("https://example.com/page");
        let referrer = PathBuf::from("https://cdn.example.com/lib/index.js");
        let result = loader.resolve_url("./helper.js", Some(&referrer));
        assert_eq!(result.unwrap(), "https://cdn.example.com/lib/helper.js");
    }

    #[test]
    fn test_set_base_url() {
        let loader = HttpModuleLoader::new("https://old.example.com/page");
        loader.set_base_url("https://new.example.com/app");
        let result = loader.resolve_url("./module.js", None);
        assert_eq!(result.unwrap(), "https://new.example.com/module.js");
    }
}
