use rmcp::{
    ErrorData,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    schemars, tool, tool_router,
};
use serde::Deserialize;
use std::sync::Arc;
use store::Store;

/// Phase 17 MCP server: exactly two read-only stub tools (real KB wiring in Phase 18).
#[derive(Clone)]
pub struct JarvisMcp {
    /// Reserved for Phase 18; unused by stubs (D-09).
    #[allow(dead_code)]
    store: Option<Arc<Store>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchParams {
    pub query: String,
    #[serde(default)]
    pub limit: Option<u32>,
}

fn stub_result(tool: &str) -> CallToolResult {
    let body = serde_json::json!({
        "status": "not_implemented",
        "phase": 18,
        "tool": tool,
        "message": "KB-backed MCP tools land in Phase 18"
    });
    CallToolResult::success(vec![ContentBlock::text(body.to_string())])
}

#[tool_router(server_handler, vis = "pub")]
impl JarvisMcp {
    pub fn new(store: Option<Arc<Store>>) -> Self {
        Self { store }
    }

    #[tool(
        description = "Search the Jarvis knowledge base (stub — Phase 18)",
        annotations(read_only_hint = true)
    )]
    pub async fn search(
        &self,
        Parameters(_params): Parameters<SearchParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(stub_result("search"))
    }

    #[tool(
        description = "List indexed sources in the Jarvis knowledge base (stub — Phase 18)",
        annotations(read_only_hint = true)
    )]
    pub async fn list_sources(&self) -> Result<CallToolResult, ErrorData> {
        Ok(stub_result("list_sources"))
    }
}
