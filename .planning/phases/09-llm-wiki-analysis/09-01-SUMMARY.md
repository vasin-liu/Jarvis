---
phase: 09-llm-wiki-analysis
plan: "01"
subsystem: insights
tags: [wiki, llm, mock, serde, insights]

requires:
  - phase: 08-deterministic-markdown-renderer
    provides: WikiAnalysis / WikiEntity / WikiConcept types and render_wiki_pages
provides:
  - InsightsError::InvalidWikiJson
  - MockChatModel 「笔记编译」 keyword happy-path JSON
  - analyze_source_for_wiki + private parse_wiki_analysis / extract_first_json_object
affects: [09-02, 10-wiki-persist]

tech-stack:
  added: []
  patterns: [keyword Mock branch before 摘要/任务/记忆, fail-typed InvalidWikiJson]

key-files:
  created: []
  modified:
    - crates/insights/src/error.rs
    - crates/insights/src/wiki.rs
    - crates/insights/src/lib.rs
    - crates/llm/src/mock.rs

key-decisions:
  - "Mock trigger is system.contains(笔记编译) before 记忆/任务/摘要 branches (D-01, D-02)"
  - "Wiki summary independent of sources.summary — no set_source_summary (D-07)"
  - "Parse: fence strip → first brace object → strict serde (D-09, D-10, D-12)"

patterns-established:
  - "Pattern: Chinese product-phrase Mock branch returns minimal valid wiki JSON"
  - "Pattern: analyze_source_for_wiki mirrors tasks Indexed/EmptySource/truncate without store writes"

requirements-completed: [WIKI-03]

coverage:
  - id: D1
    description: MockChatModel system prompt containing 笔记编译 returns minimal valid wiki JSON
    requirement: WIKI-03
    verification:
      - kind: unit
        ref: cargo test -p llm mock_wiki_compile_json
        status: pass
    human_judgment: false
  - id: D2
    description: analyze_source_for_wiki with Mock yields populated WikiAnalysis; sources.summary untouched
    requirement: WIKI-03
    verification:
      - kind: unit
        ref: cargo test -p insights analyze_
        status: pass
    human_judgment: false

duration: 25min
completed: 2026-07-19
status: complete
---

# Phase 09 Plan 01: Analyze foundation Summary

**Mock 「笔记编译」 → `analyze_source_for_wiki` returns populated `WikiAnalysis` without touching `sources.summary`.**

## Performance

- **Duration:** ~25 min
- **Tasks:** 2/2
- **Files modified:** 4

## Accomplishments

- Added `InsightsError::InvalidWikiJson` (D-12)
- Extended `MockChatModel` with 「笔记编译」 happy-path JSON before 记忆/任务/摘要 (D-01…D-03)
- Implemented `analyze_source_for_wiki` with Chinese bare-JSON prompt, 12k truncate, private JSON extract/parse (D-05…D-10)
- Locked happy path with `mock_wiki_compile_json`, `analyze_parses_mock_json`, `analyze_does_not_touch_source_summary`

## Self-Check: PASSED

- `InvalidWikiJson` present in error.rs
- `笔记编译` branch in mock.rs before 任务提取
- `analyze_source_for_wiki` exported from lib.rs
- No `set_source_summary` in wiki.rs
- `cargo test -p insights -p llm -- --test-threads=1` green for lib tests
