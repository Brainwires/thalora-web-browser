// Tool definition modules - each containing schema definitions for different tool categories
mod accessibility;
mod brainclaw;
mod browser;
mod cdp;
mod scraping;
mod session;
mod vault;

// Re-export all tool definition functions
pub(crate) use accessibility::get_accessibility_tool_definitions;
pub(crate) use brainclaw::get_brainclaw_alias_tool_definitions;
pub(crate) use browser::get_browser_automation_tool_definitions;
pub(crate) use cdp::get_cdp_tool_definitions;
pub(crate) use scraping::{
    get_minimal_scraping_tool_definitions, get_minimal_search_tool_definitions,
    get_scraping_tool_definitions, get_search_tool_definitions,
};
pub(crate) use session::get_session_tool_definitions;
pub(crate) use vault::get_vault_tool_definitions;
