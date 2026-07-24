# Phase 13: E2E + citation trust - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-24
**Phase:** 13-E2E + citation trust
**Areas discussed:** Wiki journey depth, Export under WebDriver, Citation trust proof, Regression placement

---

## Wiki journey depth

| Option | Description | Selected |
|--------|-------------|----------|
| Happy-path only | enable → compile → 笔记页 → export | |
| Happy-path + default-off | also assert controls absent when disabled | ✓ |
| Also empty-export | enable then export before compile shows empty copy | |

**User's choice:** Happy-path + default-off

| Option | Description | Selected |
|--------|-------------|----------|
| 笔记页 label | at least one Library row with「笔记页」 | ✓ |
| Label + no compile on wiki rows | also assert wiki rows lack compile button | |
| You decide | planner picks selectors | |

**User's choice:** 笔记页 label

| Option | Description | Selected |
|--------|-------------|----------|
| Real UI only | Settings toggle + save | |
| E2E seed enabled | start with wiki.enabled=true | |
| Hybrid | assert off then UI enable then compile/export | ✓ |

**User's choice:** Hybrid

| Option | Description | Selected |
|--------|-------------|----------|
| Fixture by name | bind sample.md display name | |
| First eligible compile button | first non-wiki indexed `wiki-compile-*` | ✓ |
| You decide | | |

**User's choice:** First eligible compile button

---

## Export under WebDriver

| Option | Description | Selected |
|--------|-------------|----------|
| E2E fixed-path bypass | skip Save dialog; write temp zip | ✓ |
| UI-only success notice | no disk assert | |
| Pre-inject destPath invoke | call export IPC with path, may skip button | |

**User's choice:** E2E fixed-path bypass

| Option | Description | Selected |
|--------|-------------|----------|
| Success notice only | | |
| Notice + file exists size > 0 | | ✓ |
| Also unzip contents | index.md + .obsidian | |

**User's choice:** Notice + file exists size > 0

| Option | Description | Selected |
|--------|-------------|----------|
| FE hook `__JARVIS_E2E_WIKI_EXPORT_PATH__` | skip save() | ✓ |
| Dedicated E2E Tauri command | | |
| You decide | | |

**User's choice:** FE hook

| Option | Description | Selected |
|--------|-------------|----------|
| Fixed TEMP `jarvis-e2e-wiki.zip` | | ✓ |
| Unique timestamp name | | |
| You decide | | |

**User's choice:** Fixed TEMP path

---

## Citation trust proof

| Option | Description | Selected |
|--------|-------------|----------|
| E2E primary | wiki.spec after compile; qa stays default-off | ✓ |
| E2E + Rust integration | also rag test with wiki indexed | |
| Rust + existing qa only | no wiki-on E2E citation assert | |

**User's choice:** E2E primary

| Option | Description | Selected |
|--------|-------------|----------|
| Add `data-source-uri` | assert URI not wiki:// | ✓ |
| Title/excerpt proxy | no Chat UI change | |
| You decide | | |

**User's choice:** `data-source-uri`

| Option | Description | Selected |
|--------|-------------|----------|
| At least one non-wiki:// | wiki citations allowed alongside | ✓ |
| No all-wiki set | equivalent framing | |
| Ban any wiki citation | too strict | |

**User's choice:** At least one non-wiki://

| Option | Description | Selected |
|--------|-------------|----------|
| `What is xyzzy-plugh?` | | ✓ |
| Chinese regression question | | |
| You decide | | |

**User's choice:** xyzzy-plugh

---

## Regression placement

| Option | Description | Selected |
|--------|-------------|----------|
| wiki.spec start + light full-ui | | ✓ |
| wiki.spec only | | |
| full-ui only for off / positive in wiki.spec | weaker hybrid | |

**User's choice:** wiki.spec + light full-ui

| Option | Description | Selected |
|--------|-------------|----------|
| Leave qa.spec unchanged | | ✓ |
| Add data-source-uri smoke in qa | | |
| You decide | | |

**User's choice:** Leave qa.spec unchanged

| Option | Description | Selected |
|--------|-------------|----------|
| Update e2e-required.mdc + e2e/README.md | | ✓ |
| e2e-required only | | |
| Planner discretion | | |

**User's choice:** Both docs

| Option | Description | Selected |
|--------|-------------|----------|
| full-ui negative only | positive stays in wiki.spec | ✓ |
| Small positive slice in full-ui | | |
| You decide | | |

**User's choice:** Negative only in full-ui

---

## Claude's Discretion

- Helper wiring for export path hook / zip size checks
- Exact full-ui Settings vs Library default-off selectors
- Mock / e2e.rs tweaks if compile JSON needs isolation
- Optional Vitest for `data-source-uri`; optional Rust rag+wiki test

## Deferred Ideas

- Empty-export E2E (Phase 12 already covers)
- Unzip content asserts in E2E
- Mandatory Rust citation integration with wiki on
