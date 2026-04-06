use crate::protocols::browser_tools::BrowserTools;
use crate::protocols::cdp::CdpServer;
use crate::protocols::mcp::McpResponse;
use serde_json::Value;
use std::rc::Rc;

mod debugger;
mod dom;
mod network;
mod page;
mod runtime;

use debugger::DebuggerTools;
use dom::DomTools;
use network::NetworkTools;
use page::PageTools;
use runtime::RuntimeTools;

/// Chrome DevTools Protocol tools implementation
/// Provides comprehensive CDP domain support for browser debugging and automation
pub struct CdpTools {
    runtime: RuntimeTools,
    debugger: DebuggerTools,
    dom: DomTools,
    network: NetworkTools,
    page: PageTools,
}

impl Default for CdpTools {
    fn default() -> Self {
        Self::new()
    }
}

impl CdpTools {
    /// Create a new CdpTools with its own BrowserTools instance (standalone mode)
    pub fn new() -> Self {
        Self::with_browser_tools(Rc::new(BrowserTools::new()))
    }

    /// Create a new CdpTools sharing an existing BrowserTools instance
    pub fn with_browser_tools(browser_tools: Rc<BrowserTools>) -> Self {
        Self {
            runtime: RuntimeTools::new(browser_tools),
            debugger: DebuggerTools::new(),
            dom: DomTools::new(),
            network: NetworkTools::new(),
            page: PageTools::new(),
        }
    }

    // Runtime domain methods
    pub async fn enable_runtime(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        self.runtime.enable_runtime(args, cdp_server).await
    }

    pub async fn evaluate_javascript(
        &mut self,
        args: Value,
        cdp_server: &mut CdpServer,
    ) -> McpResponse {
        self.runtime.evaluate_javascript(args, cdp_server).await
    }

    pub async fn get_console_messages(
        &mut self,
        args: Value,
        cdp_server: &mut CdpServer,
    ) -> McpResponse {
        self.runtime.get_console_messages(args, cdp_server).await
    }

    // Debugger domain methods
    pub async fn enable_debugger(
        &mut self,
        args: Value,
        cdp_server: &mut CdpServer,
    ) -> McpResponse {
        self.debugger.enable_debugger(args, cdp_server).await
    }

    pub async fn set_breakpoint(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        self.debugger.set_breakpoint(args, cdp_server).await
    }

    // DOM domain methods
    pub async fn enable_dom(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        self.dom.enable_dom(args, cdp_server).await
    }

    pub async fn get_document(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        self.dom.get_document(args, cdp_server).await
    }

    pub async fn query_selector(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        self.dom.query_selector(args, cdp_server).await
    }

    pub async fn get_attributes(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        self.dom.get_attributes(args, cdp_server).await
    }

    pub async fn get_computed_style(
        &mut self,
        args: Value,
        cdp_server: &mut CdpServer,
    ) -> McpResponse {
        self.dom.get_computed_style(args, cdp_server).await
    }

    // Network domain methods
    pub async fn enable_network(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        self.network.enable_network(args, cdp_server).await
    }

    pub async fn get_response_body(
        &mut self,
        args: Value,
        cdp_server: &mut CdpServer,
    ) -> McpResponse {
        self.network.get_response_body(args, cdp_server).await
    }

    pub async fn get_cookies(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        self.network.get_cookies(args, cdp_server).await
    }

    pub async fn set_cookie(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        self.network.set_cookie(args, cdp_server).await
    }

    // Page domain methods
    pub async fn take_screenshot(
        &mut self,
        args: Value,
        cdp_server: &mut CdpServer,
    ) -> McpResponse {
        self.page.take_screenshot(args, cdp_server).await
    }

    pub async fn reload_page(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        self.page.reload_page(args, cdp_server).await
    }
}
