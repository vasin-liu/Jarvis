# Phase 09: LLM wiki analysis - Context

**Gathered:** 2026-07-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver `analyze_source_for_wiki` in `crates/insights`: load an indexed source, call `ChatModel`, parse model text into `WikiAnalysis`. Bad/non-JSON output returns a typed error and must never start a wiki write. Include a thin public write-to-dir helper (render tree only) so tests can prove fail-closed (zero files under a tempfile wiki root on parse failure).

**Requirements:** WIKI-03 (analyze half), WIKI-05  
**Success criteria (ROADMAP):** Mock valid JSON → populated analysis; invalid output → typed parse error; parse failure writes **zero** files under tempfile wiki root.

**Out of scope this phase:** Persist under `{app_data}/wiki/` + index as `WikiPage` (Phase 10); Library/Settings UX (Phase 11); Obsidian zip (Phase 12); E2E journey (Phase 13).

</domain>

<decisions>
## Implementation Decisions

### Mock reply strategy
- **D-01:** Extend existing **keyword-heuristic** `MockChatModel` (same pattern as 摘要 / 任务提取). Do **not** add a queue-based `MockChatModel::new(vec![…])` API in this phase.
- **D-02:** Trigger on a **Chinese product phrase** in the system prompt (e.g. 「笔记编译」).
- **D-03:** On trigger, return a **minimal valid** wiki JSON: non-empty `summary`, one entity, empty `concepts` array.
- **D-04:** Bad-JSON / fail-path tests use a **test double / wrapper** `ChatModel` that returns garbage; production `MockChatModel` stays happy-path only.

### Prompt contract / analysis depth
- **D-05:** System prompt in **Chinese**, requiring **bare JSON only** (no markdown fences in the contract text). Field names remain English to match `WikiAnalysis` serde.
- **D-06:** Guide **sparse, quality-first** extraction (roughly ≤5–8 entities and concepts each); empty `entities` / `concepts` arrays are allowed.
- **D-07:** Wiki `summary` is **independent** of `sources.summary` — do not read or write the store summary field; decouple from `summarize_source`.
- **D-08:** User message = **title + body** only (same shape as summarize/tasks). Body from `source_chunk_text` via `truncate_chars` (**12_000**).

### Parse strictness
- **D-09:** Extract JSON by **stripping markdown fences**, then taking the **first `{…}` object**.
- **D-10:** **Strict** `serde` deserialize into `WikiAnalysis` — missing fields or wrong types → error. Empty arrays are fine when present.
- **D-11:** **Structurally valid = `Ok`**, even if `summary` is `""` and both lists are empty (renderer already supports empty entities).
- **D-12:** Add **`InsightsError::InvalidWikiJson`** (symmetric with `InvalidTasksJson`); include extract/serde reason in the message.

### Fail-closed proof
- **D-13:** Ship a **thin public write helper** that writes `render_wiki_pages` output (sources/entities/concepts tree + `index.md`) to a directory. Callers **analyze first**; only on `Ok` call the writer (gate). No `WikiPage` indexing / embed in this phase.
- **D-14:** Fail-closed tests cover **three fixtures**: (a) prose/non-JSON, (b) truncated `{…`, (c) JSON object with missing/wrong-typed fields. Each asserts `InvalidWikiJson` and **zero files** under the tempfile wiki root.

### Claude's Discretion
- Exact Chinese system-prompt wording (must include the 「笔记编译」 trigger and bare-JSON + sparse-extraction guidance).
- Exact minimal Mock JSON string literals (names/blurbs).
- Whether parse helpers live as private fns in `wiki.rs` vs a small submodule.
- Whether the write helper is named `write_wiki_pages_to_dir` / `try_write_wiki_compile_result` / similar — as long as D-13 holds.
- Test placement: inline `#[cfg(test)]` vs `crates/insights/tests/` — follow crate conventions.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone / phase
- `.planning/ROADMAP.md` — Phase 09 goal, success criteria, WIKI-03 (analyze) / WIKI-05
- `.planning/REQUIREMENTS.md` — WIKI-03, WIKI-05
- `.planning/PROJECT.md` — v1.10 wiki goals; local-first; Mock in tests
- `.planning/STATE.md` — current phase position

### Research
- `.planning/research/SUMMARY.md` — Phase 09 deliverables; fail-closed on parse
- `.planning/research/FEATURES.md` — LLM → structured analysis; MockChatModel
- `.planning/research/STACK.md` — ChatModel / Mock; no live LLM in CI
- `.planning/research/PITFALLS.md` — Pitfall 5 (partial write on parse fail); Pitfall 9 (Mock required)

### Implementation plan
- `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` — Task 3 interfaces (`analyze_source_for_wiki`, prompt schema). Apply discussion overrides: keyword Mock (not queue ctor), `InvalidWikiJson` (not generic `Parse`), public write helper for fail-closed proof.

### Prior phases
- `.planning/phases/08-deterministic-markdown-renderer/08-CONTEXT.md` — `WikiAnalysis` shape, `render_wiki_pages`, tree layout, wikilinks
- `.planning/phases/07-wiki-kind-config/07-CONTEXT.md` — `WikiConfig` / `WikiPage` kind (no re-litigate)

### Code integration points
- `crates/insights/src/wiki.rs` — types + `render_wiki_pages`; add analyze + write helper
- `crates/insights/src/summarize.rs` / `tasks.rs` — prompt + truncate + store load patterns
- `crates/insights/src/error.rs` — add `InvalidWikiJson`
- `crates/insights/src/lib.rs` — `truncate_chars`, re-exports
- `crates/llm/src/mock.rs` — keyword branch for 「笔记编译」

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `WikiAnalysis` / `WikiEntity` / `WikiConcept` / `render_wiki_pages` — already in `wiki.rs` (Phase 08)
- `truncate_chars` + `MAX_SOURCE_CHARS = 12_000` pattern from summarize/tasks
- `MockChatModel` heuristic branches for 摘要 / 任务提取 — extend with wiki trigger
- `parse_task_drafts` fence-strip — wiki parse goes further (first `{…}` + stricter schema)

### Established Patterns
- Insights: load source → require `Indexed` → chunk text → LLM → parse/persist
- Errors: `thiserror` variants per domain (`InvalidTasksJson` precedent)
- Tests: `Store::open_in_memory` + sample indexed source + `#[tokio::test]`

### Integration Points
- Phase 10 will call analyze → write under `{app_data}/wiki/` → index `wiki://{slug}`
- Phase 13 E2E will rely on Mock keyword returning wiki JSON (dedicated sequences later)
- No Tauri command required in Phase 09 unless planner finds a thin glue need for tests only

</code_context>

<specifics>
## Specific Ideas

- Prefer product-language trigger 「笔记编译」 so Mock stays consistent with Chinese insights prompts.
- Fail-closed proof uses real render tree (not a sentinel file) so the gate matches Phase 10 write shape.
- Do not couple wiki compile to `summarize_source` / `sources.summary` in this milestone slice.

</specifics>

<deferred>
## Deferred Ideas

- Queue-based / programmable `MockChatModel` replies — not needed for v1.10 Phase 09; revisit if E2E mock collisions force it (Phase 13)
- Full `{app_data}/wiki/` persist + `WikiPage` indexing — Phase 10
- `auto_on_insights` wiring — still Future / later
- Library compile UX — Phase 11
- Obsidian zip — Phase 12
- E2E wiki journey — Phase 13

None — discussion stayed within phase scope (no folded todos)

</deferred>

---

*Phase: 09-LLM wiki analysis*
*Context gathered: 2026-07-19*
