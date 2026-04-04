// Tool definition modules - each containing schema definitions for different tool categories
mod accessibility;
mod advanced;
mod brainclaw;
mod browser;
mod cdp;
mod memory;
mod scraping;
mod session;
#[cfg(feature = "wasm-debug")]
mod wasm_debug;

// Re-export all tool definition functions
pub(crate) use accessibility::get_accessibility_tool_definitions;
pub(crate) use advanced::get_advanced_tool_definitions;
pub(crate) use brainclaw::get_brainclaw_alias_tool_definitions;
pub(crate) use browser::get_browser_automation_tool_definitions;
pub(crate) use cdp::get_cdp_tool_definitions;
pub(crate) use memory::get_memory_tool_definitions;
pub(crate) use scraping::{
    get_minimal_scraping_tool_definitions, get_minimal_search_tool_definitions,
    get_scraping_tool_definitions, get_search_tool_definitions,
};
pub(crate) use session::get_session_tool_definitions;
#[cfg(feature = "wasm-debug")]
pub(crate) use wasm_debug::get_wasm_debug_tool_definitions;
