# 17-01 Summary — MCP crate scaffold + path resolution

**Status:** complete
**Plan:** 17-01

## What shipped
- Workspace member `crates/mcp` + bin `jarvis-mcp` (console, no GUI subsystem)
- `rmcp` pinned **2.2.0** (server + transport-io + macros); identity verified via `cargo info`
- `paths.rs`: `default_app_data_dir`, `resolve_db_path`, `ensure_db_exists`, `parse_db_flag`
- `main.rs`: resolve + fail-closed preflight (no Store/stdio yet)

## Verify
- `cargo test -p mcp --test paths` — 5 passed
- `cargo test -p mcp` path-related unit tests — passed

## Decisions
- D-01, D-02, D-05, D-06
