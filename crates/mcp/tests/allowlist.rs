use std::collections::HashSet;

use mcp::JarvisMcp;

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
