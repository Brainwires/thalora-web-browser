#![cfg(feature = "http-transport")]

use std::cell::RefCell;

use rmcp::{
    RoleServer, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResult, Implementation, ListToolsResult,
        PaginatedRequestParams, ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    Error as McpError,
};

use crate::engine::EngineConfig;
use crate::protocols::mcp_server::core::McpServer;

/// rmcp `ServerHandler` wrapper for `McpServer`.
///
/// `RefCell` provides interior mutability so rmcp's `&self` trait methods can
/// drive `McpServer`'s `&mut self` internals.  This is safe because rmcp's
/// `"local"` feature uses `spawn_local`, which guarantees serial execution —
/// only one call runs at a time per session instance.
pub struct McpServerService(RefCell<McpServer>);

impl McpServerService {
    pub fn new(server: McpServer) -> Self {
        Self(RefCell::new(server))
    }

    pub fn with_engine(config: EngineConfig) -> Self {
        Self::new(McpServer::new_with_engine(config))
    }
}

impl ServerHandler for McpServerService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::default())
            .with_server_info(Implementation::new("thalora-mcp-server", env!("CARGO_PKG_VERSION")))
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let defs = self.0.borrow().get_tool_definitions();
        let tools = defs
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let name = request.name.to_string();
        let arguments = request
            .arguments
            .map(serde_json::Value::Object)
            .unwrap_or_default();
        let result = self.0.borrow_mut().call_tool(name, arguments).await;
        Ok(result.into())
    }
}
