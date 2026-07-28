# 18-03 Summary — MCP real handlers + embedder

**Status:** complete
## Shipped
- `JarvisMcp::new(store, embedder)`; JSON search/list; `build_embedder` fail-closed in main
- `tests/tools_kb.rs` + allowlist preserved
## Verify
- `cargo test -p mcp` — allowlist + tools_kb + paths green
