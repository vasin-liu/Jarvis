---
phase: 17
slug: mcp-transport-read-only-scaffold
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-28
---

# Phase 17 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (+ `tempfile`) |
| **Config file** | `crates/mcp/Cargo.toml`; store inline `#[cfg(test)]` |
| **Quick run command** | `cargo test -p mcp` |
| **Full suite command** | `cargo test -p mcp && cargo test -p store` |
| **Estimated runtime** | ~30–90 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p mcp` (or focused store test if only store changed)
- **After every plan wave:** Run `cargo test -p mcp && cargo test -p store`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 17-01-01 | 01 | 1 | MCP-03 | T-17-01 | Path resolution; missing DB fails | unit | `cargo test -p mcp paths` | ❌ W0 | ⬜ pending |
| 17-01-02 | 01 | 1 | MCP-03 | T-17-02 | `--db` parse + fail-closed preflight | unit | `cargo test -p mcp` | ❌ W0 | ⬜ pending |
| 17-02-01 | 02 | 1 | MCP-03 | T-17-04 | WAL on Store::open | unit | `cargo test -p store wal` | ❌ W0 | ⬜ pending |
| 17-02-02 | 02 | 1 | D-08 | T-17-05 | Full store suite green | unit | `cargo test -p store` | ✅ | ⬜ pending |
| 17-03-01 | 03 | 2 | MCP-04 | T-17-06 | Exact tool allowlist | integration | `cargo test -p mcp allowlist` | ❌ W0 | ⬜ pending |
| 17-03-02 | 03 | 2 | MCP-04 | T-17-07 | Stub exports + full mcp suite | integration | `cargo test -p mcp` | ❌ W0 | ⬜ pending |
| 17-04-01 | 04 | 3 | MCP-03 | T-17-09 | stdio main + Store::open after preflight | other | `cargo build -p mcp && cargo test -p mcp` | ❌ W0 | ⬜ pending |
| 17-04-02 | 04 | 3 | MCP-03 | T-17-11 | docs/mcp.md + README + phase gate | other | `cargo test -p mcp && cargo test -p store` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/mcp/` scaffold + workspace member + `rmcp` 2.2.0 pin (Plan 17-01)
- [ ] `paths.rs` tests with `tempfile` (Plan 17-01)
- [ ] Allowlist test for `{search, list_sources}` only (Plan 17-03)
- [ ] Stub handler smoke (no Store mutation) (Plan 17-03)
- [ ] WAL test in `store` (Plan 17-02)
- [ ] `docs/mcp.md` + README pointer (Plan 17-04)

Existing infrastructure: workspace `cargo test` / `tempfile` already used elsewhere — no new test framework install.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Cursor mcp.json smoke | MCP-03 | Host-specific | Optional: point Cursor at debug `jarvis-mcp` with `JARVIS_DATA_DIR` — not CI-gated |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** plans mapped 2026-07-28 — execute to flip per-task status
