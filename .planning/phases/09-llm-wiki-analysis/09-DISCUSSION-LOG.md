# Phase 09: LLM wiki analysis - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-19
**Phase:** 09-LLM wiki analysis
**Areas discussed:** Mock reply strategy, Prompt contract / analysis depth, Parse strictness, Fail-closed proof

---

## Mock reply strategy

### Q1: Deterministic wiki JSON for tests

| Option | Description | Selected |
|--------|-------------|----------|
| Keyword heuristic | Mirror 摘要/任务提取; no queue API | ✓ |
| Queue-based override | `MockChatModel` with configurable replies | |
| Both | Keyword default + optional queue | |
| You decide | | |

**User's choice:** Keyword heuristic  
**Notes:** Align with existing insights Mock pattern.

### Q2: Trigger phrase

| Option | Description | Selected |
|--------|-------------|----------|
| Chinese product phrase | e.g. 「笔记编译」 | ✓ |
| English technical token | e.g. `wiki_analysis` | |
| Composite schema hints | JSON + entities/concepts cues | |
| You decide | | |

**User's choice:** Chinese product phrase  

### Q3: Fixed JSON payload

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal valid sample | summary + 1 entity + empty concepts | ✓ |
| Full dense sample | many entities/concepts | |
| Switchable heuristic variants | | |
| You decide | | |

**User's choice:** Minimal valid sample  

### Q4: Bad-JSON fixtures

| Option | Description | Selected |
|--------|-------------|----------|
| Test-injected double | Wrapper ChatModel in tests | ✓ |
| Mock bad-reply branch | e.g. `WIKI_BAD_JSON` marker | |
| Pure parse unit tests only | No ChatModel on fail path | |
| You decide | | |

**User's choice:** Test-injected double  

---

## Prompt contract / analysis depth

### Q1: Prompt language

| Option | Description | Selected |
|--------|-------------|----------|
| Chinese + bare JSON | English field names | ✓ |
| Full English schema contract | | |
| Bilingual with English example | | |
| You decide | | |

**User's choice:** Chinese + bare JSON  

### Q2: Extraction depth

| Option | Description | Selected |
|--------|-------------|----------|
| Sparse quality-first (~5–8) | Empty arrays OK | ✓ |
| Extract as many as possible | | |
| No quantity guidance | | |
| You decide | | |

**User's choice:** Sparse quality-first  

### Q3: Relation to sources.summary

| Option | Description | Selected |
|--------|-------------|----------|
| Independent wiki summary | No read/write of store summary | ✓ |
| Reuse existing summary as context | | |
| Skip LLM summary when present | | |
| You decide | | |

**User's choice:** Independent wiki summary  

### Q4: User message contents

| Option | Description | Selected |
|--------|-------------|----------|
| Title + body | Same as summarize/tasks; 12k truncate | ✓ |
| Also kind/uri | | |
| Body only | | |
| You decide | | |

**User's choice:** Title + body  

---

## Parse strictness

### Q1: JSON extraction

| Option | Description | Selected |
|--------|-------------|----------|
| Strip fence + first `{…}` | Per wiki plan / PITFALLS | ✓ |
| Fence strip only | Match tasks parser | |
| Whole reply must be pure JSON | | |
| You decide | | |

**User's choice:** Strip fence + first object  

### Q2: Missing fields / types

| Option | Description | Selected |
|--------|-------------|----------|
| Strict serde | Empty arrays OK when present | ✓ |
| Lenient defaults | | |
| Semi-lenient | | |
| You decide | | |

**User's choice:** Strict serde  

### Q3: Empty semantic result

| Option | Description | Selected |
|--------|-------------|----------|
| Structural success | `""` summary + empty lists OK | ✓ |
| Require non-empty summary | | |
| Require ≥1 entity/concept | | |
| You decide | | |

**User's choice:** Structural success  

### Q4: Error type

| Option | Description | Selected |
|--------|-------------|----------|
| `InsightsError::InvalidWikiJson` | Symmetric with tasks | ✓ |
| Unified `Parse` | | |
| Reuse `InvalidTasksJson` | | |
| You decide | | |

**User's choice:** InvalidWikiJson  

---

## Fail-closed proof

### Q1: How to prove zero files this phase

| Option | Description | Selected |
|--------|-------------|----------|
| Thin compile stub + gate | Analyze Ok → write tempfile | ✓ |
| Defer zero-file test to Phase 10 | | |
| Injected Writer callback | | |
| You decide | | |

**User's choice:** Thin compile stub + gate  

### Q2: What the stub writes

| Option | Description | Selected |
|--------|-------------|----------|
| Full render tree + index.md | No index/embed | ✓ |
| Sentinel file only | | |
| Full Phase 10 pipeline | | |
| You decide | | |

**User's choice:** Full render tree  

### Q3: API visibility

| Option | Description | Selected |
|--------|-------------|----------|
| Public write-to-dir helper | Caller analyzes first | ✓ |
| Single dry compile entrypoint | | |
| `#[cfg(test)]` only | | |
| You decide | | |

**User's choice:** Public write helper  

### Q4: Bad fixtures to cover

| Option | Description | Selected |
|--------|-------------|----------|
| Three fixtures | Prose / truncated / schema-invalid | ✓ |
| One fixture only | | |
| Also fenced valid JSON positive | | |
| You decide | | |

**User's choice:** Three fixtures  

---

## Claude's Discretion

- Exact system-prompt wording (must include trigger + bare-JSON + sparse guidance)
- Exact Mock minimal JSON literals
- Parse helper module layout
- Write-helper function name
- Test file placement conventions

## Deferred Ideas

- Programmable Mock reply queues → Phase 13 if needed
- Persist + index WikiPage → Phase 10
- UI / zip / E2E → Phases 11–13
