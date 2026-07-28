use std::collections::HashSet;

use mcp::{JarvisMcp, SearchParams};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ContentBlock;

#[test]
fn tool_allowlist_is_exactly_search_and_list_sources() {
    let tools = JarvisMcp::tool_router().list_all();
    let names: HashSet<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    assert_eq!(
        names,
        HashSet::from(["search", "list_sources"]),
        "MCP-04 allowlist must be exactly {{search, list_sources}}, got {names:?}"
    );
    assert_eq!(tools.len(), 2);
}

fn first_text(result: &rmcp::model::CallToolResult) -> String {
    match &result.content[0] {
        ContentBlock::Text(t) => t.text.clone(),
        other => panic!("expected text block, got {other:?}"),
    }
}

#[tokio::test]
async fn stub_search_returns_not_implemented_json() {
    let srv = JarvisMcp::new(None);
    let result = srv
        .search(Parameters(SearchParams {
            query: "hello".into(),
            limit: None,
        }))
        .await
        .expect("stub must succeed");
    let v: serde_json::Value =
        serde_json::from_str(&first_text(&result)).expect("parseable JSON");
    assert_eq!(v["status"], "not_implemented");
    assert_eq!(v["phase"], 18);
    assert_eq!(v["tool"], "search");
}

#[tokio::test]
async fn stub_list_sources_returns_not_implemented_json() {
    let srv = JarvisMcp::new(None);
    let result = srv.list_sources().await.expect("stub must succeed");
    let v: serde_json::Value =
        serde_json::from_str(&first_text(&result)).expect("parseable JSON");
    assert_eq!(v["status"], "not_implemented");
    assert_eq!(v["phase"], 18);
    assert_eq!(v["tool"], "list_sources");
}
