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
| **Quick run command** | `cargo test -p insights -- --test-threads=1 wiki` |
| **Full suite command** | `cargo test -p insights -p llm -- --test-threads=1` |
| **Estimated runtime** | ~45 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p insights -- --test-threads=1 wiki`
- **After every plan wave:** Run `cargo test -p insights -p llm -- --test-threads=1`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 09-01-01 | 01 | 1 | WIKI-03 | T-09-03 | InvalidWikiJson + Mock 「笔记编译」 → valid wiki JSON | unit / tokio | `cargo test -p llm -- --exact --test-threads=1 mock_wiki_compile_json` | ❌ W0 | ⬜ pending |
| 09-01-02 | 01 | 1 | WIKI-03 | T-09-01 | Mock + analyze → populated analysis; summary untouched | unit / tokio | `cargo test -p insights -- --exact --test-threads=1 analyze_parses_mock_json analyze_does_not_touch_source_summary` | ❌ W0 | ⬜ pending |
| 09-02-01 | 02 | 2 | WIKI-03 | T-09-07 | Empty Ok; fence/chatter extract; truncated/schema → InvalidWikiJson | unit | `cargo test -p insights -- --exact --test-threads=1 parse_wiki_allows_empty_struct parse_wiki_accepts_fenced_or_chatter parse_wiki_rejects_truncated_object parse_wiki_rejects_schema` | ❌ W0 | ⬜ pending |
| 09-02-02 | 02 | 2 | WIKI-05 | T-09-05 | write helper creates index.md + sources tree under tempfile | unit | `cargo test -p insights -- --exact --test-threads=1 write_wiki_pages_to_dir_writes_tree` | ❌ W0 | ⬜ pending |
| 09-02-03 | 02 | 2 | WIKI-05 | T-09-06 | Three garbage fixtures → InvalidWikiJson; zero files | unit / tokio | `cargo test -p insights -- --exact --test-threads=1 wiki_parse_fail_prose_writes_zero_files wiki_parse_fail_truncated_writes_zero_files wiki_parse_fail_schema_writes_zero_files` | ❌ W0 | ⬜ pending |

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
