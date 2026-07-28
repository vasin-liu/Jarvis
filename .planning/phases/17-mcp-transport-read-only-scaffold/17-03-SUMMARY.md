# 17-03 Summary — Stub tools + allowlist

**Status:** complete
**Plan:** 17-03

## What shipped
- `JarvisMcp` with `#[tool_router(server_handler)]` — tools exactly `search` + `list_sources`
- Stub handlers return `not_implemented` / phase 18 JSON (no Store mutation)
- `crates/mcp/tests/allowlist.rs` enforces MCP-04 allowlist + stub smoke

## Verify
- `cargo test -p mcp --test allowlist` — 3 passed

## Decisions
- D-03, D-04, D-09
