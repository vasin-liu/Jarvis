---
phase: 10
slug: persist-index
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-21
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` + `#[tokio::test]` + `tempfile` |
| **Config file** | none (crate-local `#[cfg(test)]` / optional `crates/insights/tests/`) |
| **Quick run command** | `cargo test -p insights compile_wiki -- --test-threads=1` |
| **Full suite command** | `cargo test -p insights -p indexer -p memory -- --test-threads=1` |
| **Estimated runtime** | ~60–120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p insights compile_wiki -- --test-threads=1`
- **After every plan wave:** Run `cargo test -p insights -p indexer -p memory -- --test-threads=1`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds
- **E2E:** Not required this phase (no UI) — deferred to Phase 13

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 10-01-01 | 01 | 0 | WIKI-04 | T-10-01 | Reject `..` in slug paths | integration scaffold | `cargo test -p insights compile_wiki` | ❌ W0 | ⬜ pending |
| 10-01-02 | 01 | 1 | WIKI-04 | — | Writes tree + scan `index.md` | integration | `cargo test -p insights compile_writes_files_and_indexes -- --exact` | ❌ W0 | ⬜ pending |
| 10-01-03 | 01 | 1 | WIKI-04 | — | `wiki://{slug}` + `WikiPage` | integration | same | ❌ W0 | ⬜ pending |
| 10-01-04 | 01 | 1 | WIKI-04 | — | Hash skip on re-compile | integration | `cargo test -p insights compile_idempotent_hash_skip` | ❌ W0 | ⬜ pending |
| 10-01-05 | 01 | 1 | WIKI-04 / D-16 | T-10-02 | Reject WikiPage input | unit/integration | `cargo test -p insights compile_rejects_wiki_page_input` | ❌ W0 | ⬜ pending |
| 10-01-06 | 01 | 1 | D-15 | — | Reject when disabled | unit/integration | `cargo test -p insights compile_rejects_when_disabled` | ❌ W0 | ⬜ pending |
| 10-02-01 | 02 | 1 | D-01…D-05 | — | Scan-rebuild lex sort | unit | `cargo test -p insights rebuild_index_md_` | ❌ W0 | ⬜ pending |
| 10-02-02 | 02 | 1 | D-06…D-09 | — | Stale cleanup + user-edit skip | integration | `cargo test -p insights compile_cleanup_and_user_edit_` | ❌ W0 | ⬜ pending |
| 10-02-03 | 02 | 1 | D-10 | — | No `content_hash` in frontmatter | unit | extend `frontmatter_omits_content_hash` / compile assert | ✅/❌ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Integration test scaffold: tempfile Store (dim=4) + seed Indexed local source + MockChatModel + MockEmbedder + `compile_wiki_for_source`
- [ ] Unit tests for `rebuild_index_md_from_disk` / frontmatter peek helpers
- [ ] User-edit + stale cleanup fixtures (pre-seed markdown files)
- [ ] Framework install: none — add insights path deps only (`indexer`/`ingest`/`embedder`/`chunker` as needed)

*Existing Phase 09 analyze/write/fail-closed tests remain; update if writer signature changes.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| — | — | All phase behaviors have automated verification (E2E deferred to Phase 13) | — |

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
