---
phase: 10
slug: persist-index
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-21
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Task IDs and waves aligned to `10-01-PLAN.md` / `10-02-PLAN.md` (revision 1).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` + `#[tokio::test]` + `tempfile` |
| **Config file** | none (crate-local `#[cfg(test)]` / optional `crates/insights/tests/`) |
| **Quick run command** | `cargo test -p insights compile_ -- --test-threads=1` |
| **Full suite command** | `cargo test -p insights -p indexer -p memory -- --test-threads=1` |
| **Estimated runtime (per-task)** | ≤30 seconds (filtered insights tests) |
| **Estimated runtime (wave/phase gate)** | ~60–120 seconds (full insights+indexer+memory) |

---

## Sampling Rate

- **After every task commit:** Filtered insights tests from that task’s `<automated>` (Nyquist ≤30s)
- **After every plan wave merge:** `cargo test -p insights -p indexer -- --test-threads=1` (Plan 01); Plan 02 wave/phase gate adds `-p memory`
- **Before `/gsd-verify-work`:** Full suite `cargo test -p insights -p indexer -p memory -- --test-threads=1` must be green
- **Max feedback latency (task):** 30 seconds
- **Max feedback latency (wave/phase gate):** 120 seconds
- **E2E:** Not required this phase (no UI) — deferred to Phase 13

---

## Per-Task Verification Map

Aligned to PLAN task names and wave labels:

| Task ID | Plan | Wave | PLAN task | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-----------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 10-01-01 | 01 | 0 | Task 1: Wave 0 scaffolds | WIKI-04 / D-15/D-16 | T-10-02 | Deps+errors+four tests; happy-path/idempotent RED | structural + RED-aware | `rg` deps/errors/tests + gate tests exit 0 + `(! cargo test … compile_writes_files_and_indexes)` + `(! … compile_idempotent_hash_skip)` | ❌ W0 | ⬜ pending |
| 10-01-02 | 01 | 1 | Task 2: GREEN compile | WIKI-04 | T-10-01, T-10-03 | Gates + write + scan `index.md` + `wiki://` + hash skip | integration | `cargo test -p insights -- --exact --test-threads=1 compile_rejects_when_disabled compile_rejects_wiki_page_input compile_writes_files_and_indexes compile_idempotent_hash_skip` | ❌ W0 | ⬜ pending |
| 10-01-03 | 01 | 1 | Task 3: Thin Tauri cmd | WIKI-04 / D-14/D-17 | T-10-04 | `compile_wiki_cmd` registered; no progress emit | structural + `cargo check` + filtered tests | `rg` wiki cmd + `cargo check -p tauri-app` + four compile_* tests | ❌ W0 | ⬜ pending |
| 10-02-01 | 02 | 2 | Task 1: rebuild units | D-01…D-05, D-13 | T-10-01 | Scan-rebuild multi-dir / lex / empty omit / no root clutter | unit | `cargo test -p insights rebuild_index_md_ -- --test-threads=1` | ❌ W0 | ⬜ pending |
| 10-02-02 | 02 | 2 | Task 2: stale cleanup | D-06…D-08 | T-10-06, T-10-08 | Stale generated FS+Store delete; cleaned counted | integration | `cargo test -p insights -- --test-threads=1 compile_cleanup_and_user_edit_ compile_removes_stale` | ❌ W0 | ⬜ pending |
| 10-02-03 | 02 | 2 | Task 3: user-edit + D-10 | D-09/D-10/D-12 | T-10-07 | User-edit skip+index; no digest in frontmatter | integration (filtered) | `cargo test -p insights -- --test-threads=1 compile_cleanup_and_user_edit_ frontmatter_omits_content_hash` | ❌ W0 | ⬜ pending |

**Wave/phase gates (not per-task `<automated>`):**

| Gate | When | Command |
|------|------|---------|
| Plan 01 wave merge | After 10-01-03 | `cargo test -p insights -p indexer -- --test-threads=1` |
| Plan 02 / phase gate | After 10-02-03 | `cargo test -p insights -p indexer -p memory -- --test-threads=1` |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Integration test scaffold: tempfile Store (dim=4) + seed Indexed local source + MockChatModel + MockEmbedder + `compile_wiki_for_source` (Task 10-01-01; happy-path/idempotent remain RED until 10-01-02)
- [ ] Unit tests for `rebuild_index_md_from_disk` / frontmatter peek helpers (Plan 02 Task 1)
- [ ] User-edit + stale cleanup fixtures (pre-seed markdown files) (Plan 02 Tasks 2–3)
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

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency ≤30s per task; wave/phase gate may be ≤120s
- [x] `nyquist_compliant: true` set in frontmatter (task map aligned to PLAN waves 0/1/2)

**Approval:** pending revision-1 alignment
