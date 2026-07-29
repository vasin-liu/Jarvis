---
phase: 15
slug: overlap-scoring-api
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-25
---

# Phase 15 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` + `tokio` async tests; `tempfile` DB; `MockEmbedder` |
| **Config file** | crate-local `#[cfg(test)]` / workspace Cargo |
| **Quick run command** | `cargo test -p retriever related -- --test-threads=4` |
| **Full suite command** | `cargo test -p retriever` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p retriever related -- --test-threads=4`
- **After every plan wave:** Run `cargo test -p retriever`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 15-01-01 | 01 | 1 | REL-02 | T-15-01 | Parameterized `get_source`; clamp `top_n` | integration | `cargo test -p retriever related_excludes_seed` | ❌ W0 | ⬜ pending |
| 15-01-02 | 01 | 1 | REL-02 | — | Unique neighbors by `source_id` | integration | `cargo test -p retriever related_rolls_up_by_source` | ❌ W0 | ⬜ pending |
| 15-01-03 | 01 | 1 | REL-02 | T-15-02 | Cap/`top_n` clamp; no unbounded retrieve | unit/integration | `cargo test -p retriever related_respects_top_n` | ❌ W0 | ⬜ pending |
| 15-01-04 | 01 | 1 | REL-02 | — | Empty seed text → `Ok([])` | unit/integration | `cargo test -p retriever related_empty_seed_text` | ❌ W0 | ⬜ pending |
| 15-01-05 | 01 | 1 | REL-02 | — | Only seed → `Ok([])` | integration | `cargo test -p retriever related_only_seed` | ❌ W0 | ⬜ pending |
| 15-01-06 | 01 | 1 | REL-02 | — | Prefers summary over title | integration | `cargo test -p retriever related_prefers_summary` | ❌ W0 | ⬜ pending |
| 15-01-07 | 01 | 1 | REL-02 | T-15-03 | DTO fields + truncated snippet; no raw score | unit | `cargo test -p retriever related_snippet_truncate` | ❌ W0 | ⬜ pending |
| 15-01-08 | 01 | 1 | REL-02 | — | No kind filter (Wiki ok) | integration | `cargo test -p retriever related_allows_wiki_neighbor` | ❌ W0 | ⬜ pending |
| 15-01-09 | 01 | 1 | REL-02 | T-15-01 | Missing `source_id` → error | integration | `cargo test -p retriever related_missing_source` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/retriever/src/related.rs` (+ `#[cfg(test)]` module) — REL-02 behaviors above
- [ ] Fixture helpers: ≥2 indexed markdown files with shared distinctive terms; 1 disjoint-vocab control
- [ ] Export `related_sources` / `RelatedSource` from `lib.rs`
- [ ] Framework install: none — existing `cargo test -p retriever` infrastructure

*Existing infrastructure covers the test runner; Wave 0 is new related-* test stubs + module.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| — | — | — | All phase behaviors have automated verification. |

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
