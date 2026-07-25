---
phase: 09-llm-wiki-analysis
plan: "02"
subsystem: insights
tags: [wiki, parse, tempfile, fail-closed, insights]

requires:
  - phase: 09-llm-wiki-analysis
    provides: analyze_source_for_wiki, InvalidWikiJson, Mock 笔记编译 branch
provides:
  - write_wiki_pages_to_dir
  - parse strictness D-09..D-11 tests
  - FixedReplyChat + three fail-closed zero-file proofs (WIKI-05)
affects: [10-wiki-persist]

tech-stack:
  added: [tempfile@3 (dev), async-trait (dev)]
  patterns: [analyze-first write-only-on-Ok gate, FixedReplyChat garbage double]

key-files:
  created: []
  modified:
    - crates/insights/Cargo.toml
    - crates/insights/src/error.rs
    - crates/insights/src/wiki.rs
    - crates/insights/src/lib.rs
    - Cargo.lock

key-decisions:
  - "InsightsError::Io for writer I/O"
  - "Fail-closed gate stays at call site — no public analyze_then_write"
  - "Production MockChatModel unchanged for garbage (D-04)"

patterns-established:
  - "Pattern: write_wiki_pages_to_dir joins only draft.slug + index.md"
  - "Pattern: FixedReplyChat fixtures assert InvalidWikiJson + zero files"

requirements-completed: [WIKI-05, WIKI-03]

coverage:
  - id: D1
    description: Parse strictness — empty Ok; fence/chatter; truncated/schema InvalidWikiJson
    requirement: WIKI-03
    verification:
      - kind: unit
        ref: cargo test -p insights parse_wiki_
        status: pass
    human_judgment: false
  - id: D2
    description: write_wiki_pages_to_dir writes index.md + sources/*.md under tempfile
    requirement: WIKI-05
    verification:
      - kind: unit
        ref: cargo test -p insights write_wiki_pages_to_dir_writes_tree
        status: pass
    human_judgment: false
  - id: D3
    description: Three garbage fixtures → InvalidWikiJson and zero wiki files
    requirement: WIKI-05
    verification:
      - kind: unit
        ref: cargo test -p insights wiki_parse_fail_
        status: pass
    human_judgment: false

duration: 20min
completed: 2026-07-19
status: complete
---

# Phase 09 Plan 02: Fail-closed write Summary

**Parse hardened; `write_wiki_pages_to_dir` ships; three garbage ChatModel fixtures prove zero files on InvalidWikiJson (WIKI-05).**

## Performance

- **Duration:** ~20 min
- **Tasks:** 3/3
- **Files modified:** 5

## Accomplishments

- Locked parse D-09…D-11 (empty structural Ok; fence/chatter; truncated/schema → InvalidWikiJson)
- Public `write_wiki_pages_to_dir` + `InsightsError::Io` + tempfile/async-trait dev-deps
- `FixedReplyChat` + prose/truncated/schema fail-closed tests (D-04, D-14)

## Self-Check: PASSED

- `write_wiki_pages_to_dir` exported
- Three `wiki_parse_fail_*_writes_zero_files` green
- `cargo test -p insights -p llm --lib` green (31 + 4)
