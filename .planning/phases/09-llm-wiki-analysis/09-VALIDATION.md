---
phase: 09
slug: llm-wiki-analysis
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-19
---

# Phase 09 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (`#[cfg(test)]` + `#[tokio::test]`) |
| **Config file** | none — Cargo workspace default |
| **Quick run command** | `cargo test -p insights wiki -- --test-threads=1` |
| **Full suite command** | `cargo test -p insights -p llm -- --test-threads=1` |
| **Estimated runtime** | ~45 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p insights wiki -- --test-threads=1`
- **After every plan wave:** Run `cargo test -p insights -p llm -- --test-threads=1`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 09-01-01 | 01 | 1 | WIKI-03 | T-09-01 | InvalidWikiJson + Mock 「笔记编译」 wired | unit / build | `cargo test -p insights -p llm --no-run` | ❌ W0 | ⬜ pending |
| 09-01-02 | 01 | 1 | WIKI-03 | T-09-01 | Mock + analyze → populated analysis | unit / tokio | `cargo test -p insights analyze_parses_mock_json -- --exact --test-threads=1` | ❌ W0 | ⬜ pending |
| 09-02-01 | 02 | 2 | WIKI-05 | T-09-02 | Prose → InvalidWikiJson; zero files | unit / tokio | `cargo test -p insights wiki_parse_fail_prose_writes_zero_files -- --exact --test-threads=1` | ❌ W0 | ⬜ pending |
| 09-02-02 | 02 | 2 | WIKI-05 | T-09-02 | Truncated → InvalidWikiJson; zero files | unit / tokio | `cargo test -p insights wiki_parse_fail_truncated_writes_zero_files -- --exact --test-threads=1` | ❌ W0 | ⬜ pending |
| 09-02-03 | 02 | 2 | WIKI-05 | T-09-02 | Schema → InvalidWikiJson; zero files | unit / tokio | `cargo test -p insights wiki_parse_fail_schema_writes_zero_files -- --exact --test-threads=1` | ❌ W0 | ⬜ pending |
| 09-02-04 | 02 | 2 | WIKI-03 | T-09-01 | Empty structural JSON → Ok; write helper tree | unit / tokio | `cargo test -p insights parse_wiki_allows_empty_struct write_wiki_pages_to_dir_writes_tree -- --exact --test-threads=1` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `analyze_source_for_wiki` + JSON extract/parse helpers in `crates/insights/src/wiki.rs` with ROADMAP tests
- [ ] `InsightsError::InvalidWikiJson` in `crates/insights/src/error.rs`
- [ ] Public `write_wiki_pages_to_dir` (or equivalent) + re-export from `lib.rs`
- [ ] `MockChatModel` 「笔记编译」 branch in `crates/llm/src/mock.rs` (+ unit test)
- [ ] `FixedReplyChat` (or similar) test double for D-14 fixtures
- [ ] `tempfile = "3"` under `crates/insights` `[dev-dependencies]`
- [ ] Framework install: none beyond `cargo` (already available)

*Existing infrastructure covers the test runner; Wave 0 adds analyze/parse/write + Mock branch test surface. No E2E Wave 0 — Phase 09 is library-only.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| — | — | — | All phase behaviors have automated verification |

*All ROADMAP success criteria have automated unit verification planned.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
