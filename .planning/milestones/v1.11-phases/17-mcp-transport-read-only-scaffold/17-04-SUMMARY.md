# 17-04 Summary — stdio main + docs

**Status:** complete
**Plan:** 17-04

## What shipped
- `jarvis-mcp` async main: path resolve → fail-closed → `load_config` → `Store::open` → `rmcp` stdio serve
- `docs/mcp.md` (Cursor + Claude Desktop + path precedence)
- README pointer to `docs/mcp.md`

## Verify
- `cargo build -p mcp` — ok
- `cargo test -p mcp` — all passed

## Decisions
- D-01, D-05, D-06, D-07
