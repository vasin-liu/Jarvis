---
phase: 17-mcp-transport-read-only-scaffold
verified: 2026-07-28T12:15:00Z
status: passed
score: 4/4 must-haves verified
behavior_unverified: 0
overrides_applied: 0
---

# Phase 17: MCP transport + read-only scaffold — Verification Report

**Phase Goal:** Ship a console `jarvis-mcp` stdio binary with fail-closed KB path resolution, WAL-ready Store, and exactly two stub tools  
**Verified:** 2026-07-28T12:15:00Z  
**Status:** passed  
**Requirements:** MCP-03, MCP-04

## Goal Achievement

### ROADMAP Success Criteria

| # | Truth | Status | Evidence |
| --- | ------- | ---------- | -------------- |
| 1 | Workspace ships console binary `jarvis-mcp` (no Windows GUI subsystem) with `rmcp` 2.2.0 | ✅ VERIFIED | `crates/mcp` + `[[bin]] jarvis-mcp`; no `windows_subsystem`; workspace pin `rmcp = 2.2.0` |
| 2 | DB path precedence `--db` > `JARVIS_DATA_DIR` > default AppData; missing DB fails closed | ✅ VERIFIED | `paths.rs` + `tests/paths.rs` (5); main exits 1 before `Store::open` |
| 3 | File-backed Store enables WAL for GUI+MCP coexistence | ✅ VERIFIED | `PRAGMA journal_mode=WAL` in `Store::open`; `store_open_enables_wal` green |
| 4 | MCP tools allowlist is exactly `{search, list_sources}` stubs (Phase 18 for real KB) | ✅ VERIFIED | `allowlist.rs` 3 tests; stub JSON `not_implemented` / phase 18 |

**Score:** 4/4 truths verified

### Requirement Coverage

| REQ | Status | Plans |
|-----|--------|-------|
| MCP-03 | ✅ | 17-01 scaffold/paths, 17-02 WAL, 17-04 stdio + docs |
| MCP-04 | ✅ | 17-03 stub tools + allowlist |

### Required Artifacts

| Artifact | Status | Details |
| -------- | ------ | ------- |
| `crates/mcp/` + workspace member | ✅ | Console bin `jarvis-mcp` |
| `Cargo.toml` `rmcp` 2.2.0 | ✅ | server + transport-io + macros |
| `crates/mcp/src/paths.rs` | ✅ | D-05 / D-06 |
| `crates/store/src/store.rs` WAL | ✅ | D-08 |
| `crates/mcp/src/server.rs` | ✅ | JarvisMcp stubs |
| `crates/mcp/src/main.rs` | ✅ | stdio serve |
| `docs/mcp.md` + README pointer | ✅ | D-07 |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Store WAL | `cargo test -p store wal` | 1 passed | ✅ PASS |
| Store suite | `cargo test -p store` | green | ✅ PASS |
| MCP paths | `cargo test -p mcp --test paths` | 5 passed | ✅ PASS |
| MCP allowlist | `cargo test -p mcp --test allowlist` | 3 passed | ✅ PASS |
| MCP full | `cargo test -p mcp` | all green | ✅ PASS |
| MCP build | `cargo build -p mcp` | jarvis-mcp binary | ✅ PASS |

### Notes

- No WebDriver E2E this phase (D-09) — cargo gates only.
- Real hybrid `search` / `list_sources` deferred to Phase 18.
