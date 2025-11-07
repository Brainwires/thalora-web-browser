// Tool definition modules - each containing schema definitions for different tool categories
mod memory;
mod cdp;
mod scraping;
mod session;
mod browser;

// Re-export all tool definition functions
pub(crate) use memory::get_memory_tool_definitions;
pub(crate) use cdp::get_cdp_tool_definitions;
pub(crate) use scraping::{
    get_scraping_tool_definitions,
    get_search_tool_definitions,
    get_minimal_scraping_tool_definitions,
    get_minimal_search_tool_definitions,
};
pub(crate) use session::get_session_tool_definitions;
pub(crate) use browser::get_browser_automation_tool_definitions;
