use rmcp::{
    ErrorData,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    schemars, tool, tool_router,
};
use serde::Deserialize;
use std::sync::Arc;
use store::Store;

use embedder::Embedder;
use retriever::{list_indexed_sources, search_kb};

/// MCP server: read-only KB tools via shared `kb_readonly` helpers.
#[derive(Clone)]
pub struct JarvisMcp {
    store: Arc<Store>,
    embedder: Arc<dyn Embedder>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchParams {
    pub query: String,
    #[serde(default)]
    pub limit: Option<u32>,
}

fn json_text(value: serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(value.to_string())])
}

fn json_error(value: serde_json::Value) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(value.to_string())])
}

#[tool_router(server_handler, vis = "pub")]
impl JarvisMcp {
    pub fn new(store: Arc<Store>, embedder: Arc<dyn Embedder>) -> Self {
        Self { store, embedder }
    }

    #[tool(
        description = "Hybrid search over the Jarvis knowledge base (vector + FTS + RRF)",
        annotations(read_only_hint = true)
    )]
    pub async fn search(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match search_kb(
            self.store.as_ref(),
            self.embedder.as_ref(),
            &params.query,
            params.limit.map(|n| n as usize),
        )
        .await
        {
            Ok(hits) => {
                let results: Vec<serde_json::Value> = hits
                    .into_iter()
                    .map(|h| {
                        serde_json::json!({
                            "chunk_id": h.chunk_id,
                            "source_id": h.source_id,
                            "title": h.title,
                            "loc": h.loc,
                            "excerpt": h.excerpt,
                        })
                    })
                    .collect();
                Ok(json_text(serde_json::json!({ "results": results })))
            }
            Err(err) => {
                let code = match &err {
                    retriever::RetrieveError::EmptyQuery => "empty_query",
                    _ => "search_failed",
                };
                Ok(json_error(serde_json::json!({
                    "error": code,
                    "message": err.to_string(),
                })))
            }
        }
    }

    #[tool(
        description = "List Indexed sources in the Jarvis knowledge base (bounded payload)",
        annotations(read_only_hint = true)
    )]
    pub async fn list_sources(&self) -> Result<CallToolResult, ErrorData> {
        match list_indexed_sources(self.store.as_ref()) {
            Ok(list) => {
                let sources: Vec<serde_json::Value> = list
                    .sources
                    .into_iter()
                    .map(|s| {
                        serde_json::json!({
                            "id": s.id,
                            "title": s.title,
                            "kind": s.kind,
                        })
                    })
                    .collect();
                Ok(json_text(serde_json::json!({
                    "sources": sources,
                    "truncated": list.truncated,
                    "total_indexed": list.total_indexed,
                })))
            }
            Err(err) => Ok(json_error(serde_json::json!({
                "error": "list_sources_failed",
                "message": err.to_string(),
            }))),
        }
    }
}
