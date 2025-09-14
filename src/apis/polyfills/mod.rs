// JavaScript polyfills organized by ES version
pub mod console;
pub mod timers;
pub mod web_apis;
pub mod syntax_transformer;

// ES version polyfills
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

use anyhow::Result;
use boa_engine::Context;

/// Setup all JavaScript polyfills in the browser context
pub fn setup_all_polyfills(context: &mut Context) -> Result<()> {
    // Core JavaScript enhancements
    console::setup_console(context)?;
    timers::setup_timers(context)?;
    web_apis::setup_web_api_globals(context)?;

    // ES version features
    es6_polyfills::setup_es6_features(context)?;
    es2017_polyfills::setup_es2017_features(context)?;
    es2018_polyfills::setup_es2018_features(context)?;
    es2019_polyfills::setup_es2019_features(context)?;
    es2020_polyfills::setup_es2020_features(context)?;
    es2021_polyfills::setup_es2021_features(context)?;
    es2022_polyfills::setup_es2022_features(context)?;
    es2023_polyfills::setup_es2023_features(context)?;
    es2024_polyfills::setup_es2024_features(context)?;
    es2025_experimental::setup_es2025_features(context)?;

    Ok(())
}