# Phase 17 Discussion Log

**Date:** 2026-07-28  
**Mode:** `--auto` (all gray areas; recommended defaults)

## Areas covered

1. Packaging & binary (`crates/mcp` / `jarvis-mcp`)
2. Read-only tool surface stubs
3. Data-dir / `--db` resolution
4. Documentation placement
5. WAL on `Store::open`
6. Testing bar (cargo-only)

## Auto selections

See `17-CONTEXT.md` `<specifics>` for the full `[auto]` audit trail (D-01..D-09).

## Not discussed (locked upstream)

- `rmcp` 2.2.0 pin / no 3.x beta (REQUIREMENTS Out of Scope)
- No MCP write tools (MCP-04 / milestone constraints)
- `store` sole SQLite owner
- Hybrid search implementation → Phase 18
