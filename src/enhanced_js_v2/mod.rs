pub mod console;
pub mod es6_polyfills;
pub mod es2017_polyfills;
pub mod es2018_polyfills;
pub mod es2019_polyfills;
pub mod es2020_polyfills;
pub mod es2021_polyfills;
pub mod es2022_polyfills;
pub mod es2023_polyfills;
pub mod es2024_polyfills;
pub mod es2025_experimental;
pub mod syntax_transformer;
pub mod timers;
pub mod web_apis;
pub mod engine;

pub use engine::EnhancedJavaScriptEngine;
pub use syntax_transformer::SyntaxTransformer;

use boa_engine::{Context, JsResult, Source};

/// Setup all enhanced JavaScript globals with complete ES2025+ support
pub fn setup_enhanced_globals(context: &mut Context) -> JsResult<()> {
    // First, establish global object reference for compatibility
    context.eval(Source::from_bytes(r#"
        if (typeof global === 'undefined') {
            var global = this;
        }
        if (typeof globalThis === 'undefined') {
            var globalThis = this;
        }
    "#))?;

    // Core polyfills
    console::setup_console(context)?;
    es6_polyfills::setup_es6_polyfills(context)?;

    // ES2017+ polyfills
    es2017_polyfills::setup_es2017_polyfills(context)?;
    es2018_polyfills::setup_es2018_polyfills(context)?;
    es2019_polyfills::setup_es2019_polyfills(context)?;
    es2020_polyfills::setup_es2020_polyfills(context)?;
    es2021_polyfills::setup_es2021_polyfills(context)?;
    es2022_polyfills::setup_es2022_polyfills(context)?;

    // Latest ES2023+ polyfills
    es2023_polyfills::setup_es2023_polyfills(context)?;
    es2024_polyfills::setup_es2024_polyfills(context)?;
    es2025_experimental::setup_es2025_experimental(context)?;

    // Web APIs and timers
    timers::setup_timers(context)?;
    web_apis::setup_web_apis(context)?;

    Ok(())
}