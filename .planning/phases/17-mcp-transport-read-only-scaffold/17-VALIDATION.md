---
phase: 17
slug: mcp-transport-read-only-scaffold
status: draft
nyquist_compliant: false
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
| 17-01-01 | 01 | 1 | MCP-03 | — | Path resolution; missing DB fails | unit | `cargo test -p mcp paths` | ❌ W0 | ⬜ pending |
| 17-01-02 | 01 | 1 | MCP-04 | T-17-SC | Exact tool allowlist | integration | `cargo test -p mcp allowlist` | ❌ W0 | ⬜ pending |
| 17-02-01 | 02 | 2 | MCP-04 | T-17-01 | Stub handlers non-mutating | integration | `cargo test -p mcp stub_smoke` | ❌ W0 | ⬜ pending |
| 17-02-02 | 02 | 2 | D-08 | — | WAL on Store::open | unit | `cargo test -p store wal` | ❌ W0 | ⬜ pending |
| 17-03-01 | 03 | 3 | MCP-03 | — | docs/mcp.md + README | other | path exists / grep | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/mcp/` scaffold + workspace member + `rmcp` 2.2.0 pin
- [ ] `paths.rs` tests with `tempfile`
- [ ] Allowlist test for `{search, list_sources}` only
- [ ] Stub handler smoke (no Store mutation)
- [ ] WAL test in `store`
- [ ] `docs/mcp.md` + README pointer

Existing infrastructure: workspace `cargo test` / `tempfile` already used elsewhere — no new test framework install.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Cursor mcp.json smoke | MCP-03 | Host-specific | Optional: point Cursor at debug `jarvis-mcp` with `JARVIS_DATA_DIR` — not CI-gated |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
