# Phase 09: LLM wiki analysis - Research

**Researched:** 2026-07-19  
**Domain:** LLM → structured `WikiAnalysis` (ChatModel + strict JSON parse) with fail-closed write gate  
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Mock reply strategy
- **D-01:** Extend existing **keyword-heuristic** `MockChatModel` (same pattern as 摘要 / 任务提取). Do **not** add a queue-based `MockChatModel::new(vec![…])` API in this phase.
- **D-02:** Trigger on a **Chinese product phrase** in the system prompt (e.g. 「笔记编译」).
- **D-03:** On trigger, return a **minimal valid** wiki JSON: non-empty `summary`, one entity, empty `concepts` array.
- **D-04:** Bad-JSON / fail-path tests use a **test double / wrapper** `ChatModel` that returns garbage; production `MockChatModel` stays happy-path only.

#### Prompt contract / analysis depth
- **D-05:** System prompt in **Chinese**, requiring **bare JSON only** (no markdown fences in the contract text). Field names remain English to match `WikiAnalysis` serde.
- **D-06:** Guide **sparse, quality-first** extraction (roughly ≤5–8 entities and concepts each); empty `entities` / `concepts` arrays are allowed.
- **D-07:** Wiki `summary` is **independent** of `sources.summary` — do not read or write the store summary field; decouple from `summarize_source`.
- **D-08:** User message = **title + body** only (same shape as summarize/tasks). Body from `source_chunk_text` via `truncate_chars` (**12_000**).

#### Parse strictness
- **D-09:** Extract JSON by **stripping markdown fences**, then taking the **first `{…}` object**.
- **D-10:** **Strict** `serde` deserialize into `WikiAnalysis` — missing fields or wrong types → error. Empty arrays are fine when present.
- **D-11:** **Structurally valid = `Ok`**, even if `summary` is `""` and both lists are empty (renderer already supports empty entities).
- **D-12:** Add **`InsightsError::InvalidWikiJson`** (symmetric with `InvalidTasksJson`); include extract/serde reason in the message.

#### Fail-closed proof
- **D-13:** Ship a **thin public write helper** that writes `render_wiki_pages` output (sources/entities/concepts tree + `index.md`) to a directory. Callers **analyze first**; only on `Ok` call the writer (gate). No `WikiPage` indexing / embed in this phase.
- **D-14:** Fail-closed tests cover **three fixtures**: (a) prose/non-JSON, (b) truncated `{…`, (c) JSON object with missing/wrong-typed fields. Each asserts `InvalidWikiJson` and **zero files** under the tempfile wiki root.

### Claude's Discretion
- Exact Chinese system-prompt wording (must include the 「笔记编译」 trigger and bare-JSON + sparse-extraction guidance).
- Exact minimal Mock JSON string literals (names/blurbs).
- Whether parse helpers live as private fns in `wiki.rs` vs a small submodule.
- Whether the write helper is named `write_wiki_pages_to_dir` / `try_write_wiki_compile_result` / similar — as long as D-13 holds.
- Test placement: inline `#[cfg(test)]` vs `crates/insights/tests/` — follow crate conventions.

### Deferred Ideas (OUT OF SCOPE)
- Queue-based / programmable `MockChatModel` replies — not needed for v1.10 Phase 09; revisit if E2E mock collisions force it (Phase 13)
- Full `{app_data}/wiki/` persist + `WikiPage` indexing — Phase 10
- `auto_on_insights` wiring — still Future / later
- Library compile UX — Phase 11
- Obsidian zip — Phase 12
- E2E wiki journey — Phase 13

None — discussion stayed within phase scope (no folded todos)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WIKI-03 (analyze half) | Compile indexed source into Markdown wiki pages (summary + entities + concepts) — LLM analysis half | `analyze_source_for_wiki` → `WikiAnalysis`; Mock keyword 「笔记编译」; Chinese bare-JSON prompt; reuse Phase 08 types + `render_wiki_pages` |
| WIKI-05 | On LLM JSON parse failure, system does **not** write a partial wiki tree (fail closed) | Parse → `InvalidWikiJson` **before** any FS write; public write helper called only on `Ok`; three garbage fixtures + tempfile zero-file assertions |
</phase_requirements>

## Summary

Phase 09 delivers the **analyze half** of WIKI-03 plus WIKI-05 fail-closed proof inside `crates/insights` (+ a one-branch extension of `MockChatModel`). Given an indexed source, `analyze_source_for_wiki` loads chunk text, calls `ChatModel` with a Chinese bare-JSON system prompt containing 「笔记编译」, and strict-deserializes into the existing `WikiAnalysis` types from Phase 08. [CITED: `09-CONTEXT.md`; VERIFIED: `crates/insights/src/wiki.rs` already defines `WikiAnalysis` / `render_wiki_pages`]

Discussion **overrides** the older plan Task 3 sketch: use **keyword-heuristic Mock** (not `MockChatModel::new(vec![…])`), typed **`InsightsError::InvalidWikiJson`** (not a generic `Parse`), and a **thin public directory writer** so tests can prove parse failure never starts a write. Persist under `{app_data}/wiki/` + `WikiPage` indexing remains Phase 10. [CITED: `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` Task 3; `09-CONTEXT.md` D-01…D-14]

Stack is almost entirely existing: `llm::ChatModel`, `serde_json`, `thiserror`, `store` load patterns from `summarize`/`tasks`, Phase 08 renderer. Only new dependency for this phase is **`tempfile` as a `dev-dependency`** on `insights` (already used across the workspace as `"3"`). [VERIFIED: `crates/insights/Cargo.toml` lacks tempfile; other crates pin `tempfile = "3"`; `cargo search tempfile` → 3.27.0]

**Primary recommendation:** Mirror `extract_tasks_from_source` load→LLM→parse flow in `wiki.rs`, but (1) never touch `sources.summary`, (2) parse with fence-strip + first-`{…}` + strict `WikiAnalysis`, (3) gate disk writes at the call site after `Ok`, and (4) extend `MockChatModel` with an early 「笔记编译」 branch returning minimal valid wiki JSON.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Load indexed source + chunk text | Library / domain (`insights` + `store`) | — | Same pattern as summarize/tasks; Store owns SQLite |
| LLM complete for wiki JSON | Library / domain (`insights` via `ChatModel`) | Provider impls in `llm` | Trait injection; Mock in unit tests |
| Mock wiki JSON replies | Library (`llm::MockChatModel`) | — | Keyword heuristic; happy-path only (D-01…D-03) |
| JSON extract + deserialize | Library / domain (`insights`) | — | Domain parse errors → `InvalidWikiJson` |
| Deterministic Markdown render | Library (`render_wiki_pages`) | — | Already shipped Phase 08; reuse, don't rewrite |
| Write render tree to directory | Library / domain (`insights` FS helper) | — | Thin helper for fail-closed proof + Phase 10 shape |
| Persist `{app_data}/wiki/` + index WikiPage | Deferred Phase 10 | — | Out of scope |
| Library / Settings / E2E | Deferred 11–13 | — | Not user-facing this phase |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust `stable` | MSRV 1.85 | Implementation language | Workspace pin [VERIFIED: `Cargo.toml`, `rust-toolchain.toml`] |
| `llm` (path) | workspace | `ChatModel`, `Message`, `Role`, `MockChatModel` | Existing insights dep [VERIFIED: `crates/insights/Cargo.toml`] |
| `store` (path) | workspace | `get_source`, `source_chunk_text`, `IndexStatus` | Existing load path [VERIFIED: `summarize.rs` / `tasks.rs`] |
| `serde` / `serde_json` | workspace `"1"` | Strict deserialize into `WikiAnalysis` | Already on insights; types already derive Deserialize [VERIFIED: `wiki.rs`, `Cargo.toml`] |
| `thiserror` | workspace `"1"` | `InsightsError::InvalidWikiJson` | Precedent `InvalidTasksJson` [VERIFIED: `error.rs`] |
| Phase 08 wiki types + `render_wiki_pages` | in-tree | Analysis target + write payload | Already complete [VERIFIED: `wiki.rs`] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tempfile` | `"3"` (crates.io **3.27.0**) | Temp wiki root in fail-closed tests | **dev-dependency only** on `insights` [VERIFIED: `cargo search tempfile`; legitimacy OK] |
| `tokio` | workspace | `#[tokio::test]` for analyze | Already insights `[dev-dependencies]` [VERIFIED] |
| `async-trait` | via `llm` | `ChatModel` trait | No direct insights dep needed [VERIFIED: `crates/llm`] |
| `std::fs` | std | Create dirs + write `.md` files | Write helper only — no new crate |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Keyword Mock branch | `MockChatModel::new(vec![…])` queue | **Rejected** — locked D-01; plan Task 3 example is outdated |
| `InvalidWikiJson` | Generic `InsightsError::Parse` | **Rejected** — locked D-12; keep symmetry with tasks |
| Soft/partial parse | Best-effort fill + write | **Rejected** — WIKI-05 / Pitfall 5 |
| Couple to `summarize_source` | Reuse `sources.summary` as wiki summary | **Rejected** — locked D-07 |
| Combined analyze+write API that writes on soft fail | Single “compile” fn with soft errors | **Rejected** — D-13 gate is analyze-then-write |

**Installation:**

```toml
# crates/insights/Cargo.toml — add under [dev-dependencies]
tempfile = "3"
```

No new production registry packages. Optional discretionary: `InsightsError::Io(#[from] std::io::Error)` for the write helper (no new crate).

**Version verification:** `tempfile = "3.27.0"` via `cargo search` (2026-07-19). Match sibling crates' `"3"` pin — do not introduce a workspace pin unless planner prefers consistency cleanup outside this phase. [VERIFIED]

## Package Legitimacy Audit

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| `tempfile` | crates | since 2015 | ~11.9M/wk | github.com/Stebalien/tempfile | OK | Approved — add as insights **dev-dep** only |
| `serde_json` | crates | since 2015 | ~18.4M/wk | github.com/serde-rs/json | OK | Already in use — no install |
| `thiserror` | crates | since 2019 | ~23.0M/wk | github.com/dtolnay/thiserror | OK | Already in use — no install |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** none  

*Do not add queue-mock crates, JSON repair crates, or `serde_yaml`.*

## Architecture Patterns

### System Architecture Diagram

```
Indexed source_id
        │
        ▼
┌───────────────────────┐
│ Store.get_source      │──NotIndexed / EmptySource──► Err
│ source_chunk_text     │
│ truncate_chars(12000) │
└───────────┬───────────┘
            ▼
┌───────────────────────┐
│ ChatModel.complete    │  system: 中文 + 「笔记编译」+ bare JSON
│ (Mock / real)         │  user: 标题 + 正文
└───────────┬───────────┘
            ▼
┌───────────────────────┐
│ strip fences          │
│ first {…} object      │──extract/serde fail──► InvalidWikiJson
│ serde WikiAnalysis    │                         (NO FS CALLS)
└───────────┬───────────┘
            │ Ok(WikiAnalysis)
            ▼
┌───────────────────────┐     Phase 09 tests / Phase 10 callers
│ render_wiki_pages     │◄──── only after Ok
└───────────┬───────────┘
            ▼
┌───────────────────────┐
│ write_*_to_dir        │──► wiki_root/{sources,entities,concepts}/*.md
│ (thin public helper)  │    + wiki_root/index.md
└───────────────────────┘
            │
            └── Phase 10: index WikiPage / content_hash (OUT OF SCOPE)
```

### Recommended Project Structure

```
crates/insights/
├── Cargo.toml                 # + tempfile = "3" under [dev-dependencies]
├── src/
│   ├── lib.rs                 # re-export analyze + write helper
│   ├── wiki.rs                # + analyze_source_for_wiki, parse_*, write_*_to_dir, tests
│   ├── error.rs               # + InvalidWikiJson (+ optional Io)
│   ├── summarize.rs           # pattern reference only (do not call)
│   └── tasks.rs               # parse_task_drafts fence pattern reference
crates/llm/
└── src/mock.rs                # + 「笔记编译」 keyword branch (before 摘要/任务提取)
```

[VERIFIED: no `crates/insights/tests/` today — prefer inline `#[cfg(test)]` like Phase 08 / summarize / tasks]

### Pattern 1: Insights load → LLM → parse (no store summary side effect)

**What:** Copy the Indexed / empty / truncate / message shape from `summarize_source` / `extract_tasks_from_source`, but **omit** `set_source_summary` and any task table writes.  
**When to use:** `analyze_source_for_wiki` only.  
**Source:** [VERIFIED: `summarize.rs`, `tasks.rs`; CITED: D-07]

```rust
// Source: crates/insights/src/tasks.rs + plan Task 3 [VERIFIED / CITED]
pub async fn analyze_source_for_wiki(
    store: &Store,
    chat: &dyn ChatModel,
    source_id: &str,
) -> Result<WikiAnalysis> {
    let source = store.get_source(source_id)?;
    if source.status != IndexStatus::Indexed {
        return Err(InsightsError::NotIndexed(source_id.to_string()));
    }
    let text = store.source_chunk_text(source_id)?;
    if text.trim().is_empty() {
        return Err(InsightsError::EmptySource(source_id.to_string()));
    }
    let excerpt = crate::truncate_chars(&text, 12_000);
    // Chinese system prompt MUST contain 「笔记编译」 (D-02, D-05, D-06)
    let messages = vec![/* system + user title/body */];
    let raw = chat.complete(&messages).await?;
    parse_wiki_analysis(&raw)
}
```

### Pattern 2: Fence strip + first JSON object + strict serde

**What:** Extend tasks' fence strip with brace-balanced first-object extraction, then `serde_json::from_str::<WikiAnalysis>`.  
**When to use:** All wiki LLM replies (models often wrap JSON in ``` fences despite instructions).  
**Source:** [CITED: D-09…D-12; VERIFIED: `tasks.rs` `parse_task_drafts`]

```rust
// Source: adapt crates/insights/src/tasks.rs::parse_task_drafts [VERIFIED]
fn parse_wiki_analysis(raw: &str) -> Result<WikiAnalysis> {
    let json = extract_first_json_object(raw)
        .map_err(|e| InsightsError::InvalidWikiJson(e))?;
    serde_json::from_str(json)
        .map_err(|e| InsightsError::InvalidWikiJson(e.to_string()))
}
```

Brace matching: find first `{`, then scan with depth counter; skip content inside JSON strings (`"` with `\` escapes) so braces in string values do not terminate early. Truncated `{…` (no close) → extract error → `InvalidWikiJson`. [ASSUMED: string-aware scanner detail — standard approach; lock with D-14 fixture (b)]

### Pattern 3: Fail-closed gate at call site

**What:** Never call the write helper unless `analyze_source_for_wiki` returned `Ok`. Writer accepts only `WikiCompileResult` (already rendered).  
**When to use:** Unit tests proving WIKI-05; Phase 10 `compile_wiki_for_source` will reuse the same order.  
**Source:** [CITED: D-13, D-14; PITFALLS Pitfall 5]

```rust
// Recommended test orchestration (not required as public API)
let dir = tempfile::tempdir().unwrap();
let wiki = dir.path().join("wiki");
match analyze_source_for_wiki(&store, &bad_chat, "s1").await {
    Ok(analysis) => {
        let source = store.get_source("s1").unwrap();
        let compiled = render_wiki_pages(&analysis, &source.uri, &source.title);
        write_wiki_pages_to_dir(&compiled, &wiki).unwrap();
    }
    Err(e) => {
        assert!(matches!(e, InsightsError::InvalidWikiJson(_)));
        assert!(!wiki.exists() || walk_is_empty(&wiki));
    }
}
```

### Pattern 4: Keyword Mock before colliding branches

**What:** In `build_answer`, check `system.contains("笔记编译")` **before** `任务提取` / `摘要` (and before any branch that keys on generic “JSON”). Return minimal valid object per D-03.  
**When to use:** Production `MockChatModel` only.  
**Source:** [VERIFIED: `crates/llm/src/mock.rs` branch order; CITED: D-01…D-03]

```rust
// Source: crates/llm/src/mock.rs [VERIFIED pattern]
if system.contains("笔记编译") {
    return Ok(
        r#"{"summary":"要点摘要","entities":[{"name":"示例实体","blurb":"说明"}],"concepts":[]}"#
            .to_string(),
    );
}
```

### Pattern 5: Garbage ChatModel test double

**What:** Small `struct` implementing `ChatModel` that ignores messages and returns a fixed bad string — used only in insights tests (D-04). Do **not** change production Mock to return garbage.  
**When to use:** D-14 fixtures (a)(b)(c).  
**Source:** [CITED: D-04; VERIFIED: `ChatModel` trait in `crates/llm/src/lib.rs`]

### Anti-Patterns to Avoid

- **Write-then-parse / soft-fail write:** Violates WIKI-05 — always parse complete `WikiAnalysis` first. [CITED: PITFALLS §5]
- **Calling `summarize_source` or `set_source_summary`:** Couples wiki to Library summary (D-07).
- **Queue Mock API:** Locked out for Phase 09 (D-01); defer to Phase 13 if E2E collisions appear.
- **Opening SQLite outside `store`:** Architecture invariant. [CITED: PITFALLS §8]
- **Indexing `WikiPage` / embeds in this phase:** Phase 10 only.
- **Trusting LLM for filesystem paths:** Slugs come from Phase 08 renderer, not model. [CITED: PITFALLS security]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON deserialize | Custom field parsers | `serde_json` + existing `WikiAnalysis` derives | Strict missing-field errors for free (D-10) |
| Chat provider | New HTTP client | Existing `ChatModel` trait | Mock/E2E already wired |
| Markdown page assembly | Re-implement render | `render_wiki_pages` | Phase 08 already green |
| Temp dirs in tests | Ad-hoc `%TEMP%` paths | `tempfile = "3"` | Workspace standard; auto-cleanup |
| Typed errors | `String` / `anyhow` | `thiserror` `InvalidWikiJson` | Matches `InvalidTasksJson` |

**Key insight:** The hard product rule is **ordering** (parse complete → then write), not a clever JSON repairer. Prefer fail-closed over resilient repair. [CITED: PITFALLS §5; FEATURES anti-features]

## Common Pitfalls

### Pitfall 1: Partial wiki write on parse failure
**What goes wrong:** Prose / truncated JSON still creates `index.md` or entity files.  
**Why it happens:** Write loop starts before full deserialize; soft-fail continues.  
**How to avoid:** D-13 gate; D-14 three fixtures assert zero files under tempfile root.  
**Warning signs:** Tests only cover happy-path Mock JSON. [CITED: PITFALLS §5; ROADMAP success #3]

### Pitfall 2: Mock branch order collision
**What goes wrong:** Wiki prompt also matches `摘要` or `任务提取` / generic JSON-array memory branch → wrong Mock reply → false parse failures.  
**Why it happens:** `build_answer` checks 摘要/任务提取 before wiki trigger.  
**How to avoid:** Insert 「笔记编译」 check **above** those branches; keep trigger phrase unique in system prompt.  
**Warning signs:** `analyze_parses_mock_json` gets Chinese prose or a JSON array. [VERIFIED: `mock.rs` order]

### Pitfall 3: Coupling wiki summary to `sources.summary`
**What goes wrong:** Compile mutates Library summary or reads stale summarize output.  
**Why it happens:** Copy-paste from `summarize_source`.  
**How to avoid:** D-07 — no `set_source_summary`; assert store summary unchanged in a unit test (recommended).  
**Warning signs:** `get_source().summary` changes after analyze. [CITED: D-07]

### Pitfall 4: Fence-only parse without first-object extract
**What goes wrong:** Model returns preamble + `{…}` + trailing prose; fence strip alone fails; or truncated `{` accepted poorly.  
**Why it happens:** Tasks parser only strips fences for arrays (`parse_task_drafts`).  
**How to avoid:** D-09 first `{…}` with brace depth; fixture (b) truncated.  
**Warning signs:** Valid JSON after chatter still errors. [CITED: D-09; VERIFIED: tasks vs wiki gap]

### Pitfall 5: Lenient serde / optional fields
**What goes wrong:** Missing `entities` silently becomes `[]` via `#[serde(default)]` on the struct — hides bad model output.  
**Why it happens:** Tempting to add defaults “for robustness.”  
**How to avoid:** Keep `WikiAnalysis` fields required (as today); empty arrays must be **present** in JSON; D-10/D-11. Fixture (c) missing/wrong-typed fields → error.  
**Warning signs:** `#[serde(default)]` added to `WikiAnalysis`. [VERIFIED: `wiki.rs` has no default on those fields]

### Pitfall 6: Writer called with unsafe slug paths
**What goes wrong:** Join user/LLM strings into paths.  
**Why it happens:** Skipping renderer and writing from raw analysis names.  
**How to avoid:** Writer only consumes `WikiCompileResult.pages[].slug` (already slugified). Create parent dirs for `sources/`, `entities/`, `concepts/`.  
**Warning signs:** `Path::new(&entity.name)` in writer. [CITED: Phase 08 slug rules]

### Pitfall 7: Scope creep into Phase 10
**What goes wrong:** Phase 09 also indexes `WikiPage` or writes under real app_data.  
**Why it happens:** Plan Task 4 looks adjacent.  
**How to avoid:** Writer is tempfile-oriented API; no `Embedder`, no `index_document`.  
**Warning signs:** New deps on `indexer` / `embedder` in insights for this phase. [CITED: CONTEXT out of scope]

## Code Examples

### Recommended public surface

```rust
// Source: plan Task 3 + CONTEXT D-12/D-13 overrides [CITED]
pub async fn analyze_source_for_wiki(
    store: &Store,
    chat: &dyn ChatModel,
    source_id: &str,
) -> Result<WikiAnalysis>;

/// Writes `{slug}.md` for each page and `index.md` under `wiki_root`.
/// Creates parent directories as needed. Does not index or embed.
pub fn write_wiki_pages_to_dir(
    compiled: &WikiCompileResult,
    wiki_root: &std::path::Path,
) -> Result<()>;
```

Name of write helper is discretionary (`write_wiki_pages_to_dir` recommended for clarity).

### System prompt contract (discretionary wording; locked requirements)

Must include:

1. Phrase **「笔记编译」** (Mock trigger D-02)  
2. **Bare JSON only** / no markdown fences (D-05)  
3. Schema with English keys: `summary`, `entities[{name,blurb}]`, `concepts[{name,blurb}]`  
4. Sparse guidance ≤5–8 entities/concepts; empty arrays allowed (D-06)

User message shape (locked D-08):

```text
标题：{title}

正文：
{excerpt}
```

### Error variant

```rust
// Source: crates/insights/src/error.rs pattern [VERIFIED]
#[error("failed to parse wiki json: {0}")]
InvalidWikiJson(String),
```

### Bad-chat test double sketch

```rust
// Source: ChatModel trait [VERIFIED: crates/llm/src/lib.rs]
struct FixedReplyChat(&'static str);

#[async_trait::async_trait]
impl ChatModel for FixedReplyChat {
    fn id(&self) -> &str { "test-fixed" }
    async fn complete(&self, _: &[Message]) -> llm::Result<String> {
        Ok(self.0.to_string())
    }
    async fn complete_stream(
        &self,
        messages: &[Message],
        on_token: &mut (dyn FnMut(String) + Send),
    ) -> llm::Result<String> {
        let answer = self.complete(messages).await?;
        on_token(answer.clone());
        Ok(answer)
    }
}
```

### Write helper shape

```rust
// Recommended behavior [ASSUMED naming; D-13 locked]
pub fn write_wiki_pages_to_dir(
    compiled: &WikiCompileResult,
    wiki_root: &std::path::Path,
) -> Result<()> {
    std::fs::create_dir_all(wiki_root)?;
    for page in &compiled.pages {
        let path = wiki_root.join(format!("{}.md", page.slug));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, &page.body_markdown)?;
    }
    std::fs::write(wiki_root.join("index.md"), &compiled.index_markdown)?;
    Ok(())
}
```

Map `std::io::Error` via `InsightsError::Io` (recommended) or `.map_err(|e| …)` — discretionary.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Plan Task 3: `MockChatModel::new(vec![json])` | Keyword heuristic 「笔记编译」 (D-01…D-03) | 2026-07-19 discuss | Matches existing Mock; no Mock API redesign |
| Plan Task 3: `InsightsError::Parse` | `InvalidWikiJson` (D-12) | 2026-07-19 discuss | Symmetric with tasks; clearer diagnostics |
| Parse-only without FS proof | Public write helper + tempfile zero-file tests (D-13/D-14) | 2026-07-19 discuss | WIKI-05 verifiable before Phase 10 |
| Fence-strip array parse (tasks) | Fence + first `{…}` object (wiki) | Phase 09 | Handles chatter/fences around objects |

**Deprecated/outdated:**
- Plan Task 3 queue-constructor example — superseded by CONTEXT D-01  
- Soft-fail / partial wiki writes — never acceptable for WIKI-05

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | String-aware brace scanner is the intended “first `{…}`” algorithm | Pattern 2 | Truncated/string-brace edge cases differ — covered by D-14 fixtures |
| A2 | Write helper returns `insights::Result<()>` via new `InsightsError::Io` | Code Examples | Planner may prefer `std::io::Result` — low risk |
| A3 | Recommended Mock JSON literals / prompt copy are discretionary as long as D-02…D-06 hold | Code Examples | Wording tweaks only |
| A4 | Asserting `sources.summary` unchanged is a useful extra test but not a ROADMAP criterion | Pitfall 3 | Optional coverage gap |

**If this table is empty:** N/A — assumptions listed above are low-risk discretion under CONTEXT.

## Open Questions (RESOLVED)

1. **Should Phase 09 export a combined `analyze_then_write` helper?** — **RESOLVED**
   - What we know: D-13 requires analyze-first gate; write helper is public; Phase 10 will orchestrate full compile.
   - What's unclear: Whether a thin combined helper reduces test duplication.
   - Recommendation: **No public combined helper** in Phase 09 — keep gate explicit in tests; Phase 10 owns `compile_wiki_for_source`.
   - **Plan answer (09-02):** No public combined helper; interfaces/context and Task 2 action forbid `analyze_then_write`.

2. **IO error variant vs `io::Result` on writer** — **RESOLVED**
   - What we know: Current `InsightsError` has no IO arm [VERIFIED: `error.rs`].
   - Recommendation: Add `Io(#[from] std::io::Error)` for consistency with `Llm`/`Store` froms — discretionary, not a discuss blocker.
   - **Plan answer (09-02):** Prefer `InsightsError::Io(#[from] std::io::Error)` for the writer (Task 2 GREEN; optional artifact in must_haves).

3. **Happy-path write test in Phase 09?** — **RESOLVED**
   - What we know: ROADMAP success criteria emphasize Mock parse + fail-closed zero files; happy write is Phase 10's main job.
   - Recommendation: Include **one** small happy-path test that analyze → render → write creates `index.md` under tempfile (proves helper works) without indexing — strengthens D-13 without stealing Phase 10 scope.
   - **Plan answer (09-02 Task 2):** One happy write-tree test — `write_wiki_pages_to_dir_writes_tree` — asserts `index.md` + at least one `sources/*.md` under tempfile.

## Environment Availability

Phase depends on local Rust toolchain + existing crates only (no live LLM, Feishu, or zip).

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` / `rustc` | unit/integration tests | ✓ | rustc 1.97.0 / cargo 1.97.0 | — |
| `llm::MockChatModel` | happy-path analyze | ✓ | in-tree | — |
| `tempfile` crate | fail-closed FS tests | ✓ (registry) | 3.27.0 | add as insights dev-dep |
| Live Ollama / cloud LLM | — | N/A | — | **Must not use** in CI/tests |

**Missing dependencies with no fallback:** none  
**Missing dependencies with fallback:** none  

Step 2.6: No external services required beyond cargo registry for `tempfile` download at build time.

## Validation Architecture

> `workflow.nyquist_validation: true` in `.planning/config.json` [VERIFIED]

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (`#[cfg(test)]` + `#[tokio::test]`) |
| Config file | none — Cargo workspace default |
| Quick run command | `cargo test -p insights wiki -- --test-threads=1` |
| Full suite command | `cargo test -p insights -p llm -- --test-threads=1` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WIKI-03 | MockChatModel + 「笔记编译」 → populated summary + ≥1 entity + concepts (may be empty) | unit / tokio | `cargo test -p insights analyze_parses_mock_json -- --exact` | ❌ Wave 0 |
| WIKI-03 | Mock branch returns valid wiki JSON when system contains 「笔记编译」 | unit / tokio | `cargo test -p llm mock_wiki_compile_json -- --exact` (or insights-only if preferred) | ❌ Wave 0 |
| WIKI-05 | Prose/non-JSON → `InvalidWikiJson`; **zero** files under tempfile wiki root | unit / tokio | `cargo test -p insights wiki_parse_fail_prose_writes_zero_files -- --exact` | ❌ Wave 0 |
| WIKI-05 | Truncated `{…` → `InvalidWikiJson`; zero files | unit / tokio | `cargo test -p insights wiki_parse_fail_truncated_writes_zero_files -- --exact` | ❌ Wave 0 |
| WIKI-05 | Missing/wrong-typed fields → `InvalidWikiJson`; zero files | unit / tokio | `cargo test -p insights wiki_parse_fail_schema_writes_zero_files -- --exact` | ❌ Wave 0 |
| WIKI-03 / D-11 | Structurally valid empty analysis JSON (`""` + `[]` + `[]`) → `Ok` | unit | `cargo test -p insights parse_wiki_allows_empty_struct — --exact` | ❌ Wave 0 |
| WIKI-03 / D-07 | Analyze does not set `sources.summary` | unit / tokio | `cargo test -p insights analyze_does_not_touch_source_summary -- --exact` | ❌ Wave 0 (recommended) |
| — | Happy write helper creates `index.md` (no index/embed) | unit / tokio | `cargo test -p insights write_wiki_pages_to_dir_writes_tree -- --exact` | ❌ Wave 0 (recommended) |
| — | E2E / UI | — | N/A this phase | skip (Phase 13) |

### Sampling Rate

- **Per task commit:** `cargo test -p insights wiki -- --test-threads=1`
- **Per wave merge:** `cargo test -p insights -p llm -- --test-threads=1`
- **Phase gate:** Full commands green; manual check that fail-path tests never call writer / wiki root file count is 0

### Wave 0 Gaps

- [ ] `analyze_source_for_wiki` + `parse_wiki_analysis` / `extract_first_json_object` in `crates/insights/src/wiki.rs` with ROADMAP tests
- [ ] `InsightsError::InvalidWikiJson` in `error.rs`
- [ ] Public `write_wiki_pages_to_dir` (or equivalent) + re-export from `lib.rs`
- [ ] `MockChatModel` 「笔记编译」 branch in `crates/llm/src/mock.rs` (+ unit test)
- [ ] `FixedReplyChat` (or similar) test double for D-14 fixtures
- [ ] `tempfile = "3"` under `crates/insights` `[dev-dependencies]`
- [ ] Framework install: none beyond `cargo` (already available)

*(No E2E Wave 0 — Phase 09 is library-only; user-facing E2E is Phase 13 / WIKI-09. `e2e-required.mdc` does not apply until UI.)*

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Strict `WikiAnalysis` serde; fence/object extract; reject bad JSON before FS |
| V6 Cryptography | no (this phase) | Slug hashing already in Phase 08 renderer |

### Known Threat Patterns for LLM wiki analysis

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Prompt injection / hostile source text | Tampering | Existing 12k truncate; model output never trusted for paths; parse schema-gated |
| LLM returns path-like entity names | Tampering | Paths only from Phase 08 slugify via `WikiCompileResult` |
| Partial write leaves corrupt vault | Tampering / Denial | Fail-closed gate (WIKI-05); zero files on parse error |
| Accidental live LLM in CI | Information Disclosure / cost | Mock only; no network in unit tests [CITED: PITFALLS §9] |
| Secrets in source sent to cloud chat | Information Disclosure | Pre-existing insights risk; same truncate; no new cloud path |

`security_enforcement`: treat as enabled (absent override in config) — validation is schema + fail-closed FS, not auth.

## Project Constraints (from .cursor/rules/)

| Rule | Directive for Phase 09 |
|------|------------------------|
| `tdd-goal-driven.mdc` | Goal + acceptance first; **failing tests first**; unit/integration with `MockChatModel` / test doubles; run `cargo test -p insights` (and `-p llm` for Mock branch) before done |
| `karpathy-guidelines.mdc` | No speculative features (no queue Mock, no Phase 10 index); surgical diffs; honor locked CONTEXT |
| `jarvis-stack.mdc` | Rust stable / MSRV 1.85; logic in library crates; no pre-release crates; pin tempfile deliberately |
| `e2e-required.mdc` | User-facing features need E2E — **Phase 09 is not user-facing UI**; no E2E this phase; Phase 13 covers wiki journey |
| `frontend-taste.mdc` | N/A — no `src/**` UI work |

## Handoff Notes (planner awareness)

| Later phase | Expects from Phase 09 |
|-------------|----------------------|
| 10 | `analyze_source_for_wiki` + `write_wiki_pages_to_dir` (or rename) + `render_wiki_pages` orchestration; add `content_hash`, `wiki://` index, WikiPage skip |
| 13 | Mock keyword 「笔记编译」 returns wiki JSON for compile path; may need dedicated E2E mock sequencing if collisions appear |

## Sources

### Primary (HIGH confidence)

- [VERIFIED] `crates/insights/src/{wiki,summarize,tasks,error,lib}.rs` — types, render, load/parse patterns
- [VERIFIED] `crates/llm/src/{lib,mock}.rs` — `ChatModel` + keyword Mock
- [VERIFIED] `crates/insights/Cargo.toml` — deps; missing tempfile
- [VERIFIED] `cargo search tempfile` → 3.27.0; package-legitimacy OK for tempfile/serde_json/thiserror (2026-07-19)
- [VERIFIED] `.planning/config.json` — `nyquist_validation: true`
- [CITED] `.planning/phases/09-llm-wiki-analysis/09-CONTEXT.md` — D-01…D-14
- [CITED] `.planning/ROADMAP.md` Phase 09 success criteria
- [CITED] `.planning/REQUIREMENTS.md` — WIKI-03, WIKI-05
- [CITED] `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` Task 3 (with discuss overrides)
- [CITED] `.planning/research/{SUMMARY,FEATURES,STACK,PITFALLS}.md` — Phase 09 deliverables / Pitfalls 5 & 9
- [CITED] `.planning/phases/08-deterministic-markdown-renderer/08-CONTEXT.md` — renderer contract consumed here

### Secondary (MEDIUM confidence)

- [CITED] Sibling crates' `tempfile = "3"` pins (`store`, `indexer`, …) as install style precedent
- [ASSUMED] JSON string-aware brace scanner details (standard practice; enforced by fixtures)

### Tertiary (LOW confidence)

- None material — phase is brownfield and decision-locked

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — existing crates verified; tempfile legitimacy OK
- Architecture: **HIGH** — CONTEXT locks APIs, Mock strategy, fail-closed gate; Phase 08 types already in tree
- Pitfalls: **HIGH** — milestone PITFALLS §5/§9 + discuss overrides align with ROADMAP tests

**Research date:** 2026-07-19  
**Valid until:** 2026-08-18 (stable brownfield; re-check only if `MockChatModel` API is redesigned or `WikiAnalysis` shape changes)
