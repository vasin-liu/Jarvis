---
phase: 18
slug: mcp-tools-search-list-sources
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-28
---

# Phase 18 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Nyquist: every plan task has `<automated>` verify; Wave 0 covers MISSING test files.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (+ `tempfile`, `MockEmbedder`) |
| **Config file** | workspace `Cargo.toml` / crate manifests |
| **Quick run command** | `cargo test -p retriever kb_readonly` |
| **Full suite command** | `cargo test -p retriever -p mcp -p agent` |
| **Estimated runtime** | ~60–120 seconds |

---

## Sampling Rate

- **After every task commit:** Focused crate test for the plan under edit (`retriever` / `agent` / `mcp`)
- **After every plan wave:** `cargo test -p retriever -p mcp -p agent`
- **Before `/gsd-verify-work`:** Full three-crate suite green; `docs/mcp.md` updated
- **Max feedback latency:** 120 seconds
- **WebDriver / E2E:** Not required this phase (CONTEXT D-12 → Phase 19)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 18-01-01 | 01 | 1 | MCP-01, MCP-02 | T-18-01..04 | Helper contracts (clamp, excerpt, Indexed, cap, no uri) | unit | `cargo test -p retriever kb_readonly` | ❌ W0 | ⬜ pending |
| 18-01-02 | 01 | 1 | MCP-01, MCP-02 | T-18-01..04 | GREEN kb_readonly + full retriever | unit | `cargo test -p retriever` | ❌ W0 | ⬜ pending |
| 18-02-01 | 02 | 2 | MCP-01, MCP-02 | T-18-05..07 | Agent arms call helpers | unit/integration | `cargo test -p agent` | ✅ / extend | ⬜ pending |
| 18-02-02 | 02 | 2 | MCP-01, MCP-02 | T-18-06 | No forked retrieve in knowledge arms | unit | `cargo test -p agent` | ✅ | ⬜ pending |
| 18-03-01 | 03 | 3 | MCP-01, MCP-02 | T-18-08..12 | RED tools_kb + allowlist keep | integration | `cargo test -p mcp` | ❌ W0 replace | ⬜ pending |
| 18-03-02 | 03 | 3 | MCP-01, MCP-02 | T-18-08..12 | GREEN handlers + fail-closed embedder | integration | `cargo test -p mcp` | ❌ W0 | ⬜ pending |
| 18-04-01 | 04 | 4 | MCP-01, MCP-02 | T-18-13..14 | Docs + phase gate suite | other | `cargo test -p retriever -p mcp -p agent` | ✅ docs | ⬜ pending |
| 18-04-02 | 04 | 4 | MCP-01, MCP-02 | T-18-13 | Docs match handlers | other | `cargo test -p mcp` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/retriever/src/kb_readonly.rs` + unit tests (Plan 18-01) — MISSING today
- [ ] Re-export from `crates/retriever/src/lib.rs` (Plan 18-01)
- [ ] Agent thin-wrapper coverage / smoke via `cargo test -p agent` (Plan 18-02)
- [ ] Replace Phase 17 stub body asserts; add `crates/mcp/tests/tools_kb.rs` (or equivalent) with tempfile + MockEmbedder (Plan 18-03)
- [ ] Keep `tool_allowlist_is_exactly_search_and_list_sources` (MCP-04 continuity)
- [ ] Update `docs/mcp.md` real tool docs (Plan 18-04)
- [ ] Framework install: none — cargo already present

Existing infrastructure: `MockEmbedder`, `tempfile`, indexer fixtures in retriever tests, Phase 17 mcp allowlist harness.

---

## Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | Covered By |
|--------|----------|-----------|-------------------|------------|
| MCP-01 | Same hybrid path as agent; MCP search JSON | unit + integration | `cargo test -p retriever` / `-p mcp` / `-p agent` | 18-01, 18-02, 18-03 |
| MCP-02 | Indexed inventory + caps + no paths | unit + integration | `cargo test -p retriever` / `-p mcp` | 18-01, 18-03 |
| MCP-04 continuity | Exact `{search, list_sources}` | unit | `cargo test -p mcp allowlist` | 18-03 (keep) |
| TRUST / E2E | WebDriver MCP | — | — | Deferred Phase 19 (D-12) |

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live Cursor/Claude Desktop search | MCP-01 | Host-specific | Optional smoke after build; not CI-gated this phase |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency ≤ 120s target
- [x] `nyquist_compliant: true` set in frontmatter
- [x] No WebDriver required for Phase 18 (explicit D-12 deferral)

**Approval:** plans mapped 2026-07-28 — execute to flip per-task status
