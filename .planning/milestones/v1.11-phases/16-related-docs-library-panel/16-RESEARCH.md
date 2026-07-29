# Phase 16: Related-docs Library panel - Research

**Researched:** 2026-07-26
**Domain:** Tauri IPC + React Library selection / related-docs panel + focused WebDriver E2E
**Confidence:** HIGH (codebase-verified backend contract + UI seams); MEDIUM (E2E dual-fixture seeding details until implementer lands exact fixture text)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Panel placement
- **D-01:** Layout composition (side column vs under-row strip vs other) is **Claude's discretion** — prefer the cleanest fit with today's single-column Library glass panel; minimize layout churn.
- **D-02:** When **no source is selected**, the related panel is **fully hidden** (no placeholder chrome).
- **D-03:** Long related lists scroll **inside the panel** independently of the main source list.
- **D-04:** Panel appearance uses **light motion** (fade / short translate), honoring `prefers-reduced-motion` / existing Library motion patterns.

#### Selection model
- **D-05:** Selecting a source = click on the **row blank area** (not action buttons). Per-row buttons (summarize / tasks / retry / etc.) keep their own handlers and do not toggle selection.
- **D-06:** Non-`indexed` sources (pending / failed / etc.) **remain selectable**; the related panel shows an honest non-indexed empty/error state instead of silent ignore.
- **D-07:** Selected row uses **highlight border / background** consistent with existing cyan accent.
- **D-08:** Clicking the **same selected row again** clears selection and hides the related panel (toggle).

#### Row content & copy (Chinese UI)
- **D-09:** Each related row shows **title + kind label + snippet** — no similarity / RRF scores (aligns REL-04 + Phase 15 D-09).
- **D-10:** Empty (indexed seed, zero neighbors): **「暂无相关文档」**.
- **D-11:** Selected but not indexed: **「仅已索引来源可查看相关文档」**.
- **D-12:** Loading: **「加载相关文档…」**; error: **「相关文档加载失败」** (optional short error detail OK).

#### Navigate-on-click
- **D-13:** Clicking a related row **selects that neighbor**, **scrolls it into view** in the source list, and **refetches** related docs with the neighbor as the new seed (related-chain browsing).
- **D-14:** Scroll into view is **smooth** by default; under `prefers-reduced-motion`, use instant positioning.
- **D-15:** Rapid successive clicks: **last selection wins** — cancel/ignore in-flight fetches for superseded seeds.
- **D-16:** This phase ships a **focused E2E** journey (select → panel states → click related navigate) under `JARVIS_E2E=1` mocks, plus stable `data-testid`s. Phase 19 still owns citation/trust regression gate.

### Claude's Discretion
- Exact panel composition within D-01 (side vs under-row vs hybrid) as long as D-02–D-04 hold
- Whether non-indexed selection skips the IPC call entirely vs calls and maps errors — product copy is D-11 either way
- Exact `data-testid` names (kebab-case, E2E-stable)
- Serde / Tauri command naming for wrapping `related_sources` (keep thin shell, store sole SQLite owner)
- Vitest coverage shape alongside E2E

### Deferred Ideas (OUT OF SCOPE)
- Soft-exclude / demote WikiPage neighbors — **REL-F01** (roadmap future)
- MCP tools — **Phases 17–18**
- Citation / trust regression gate + full journey hardening — **Phase 19** (this phase still does focused related-docs E2E per D-16)
- Graph UI / co-citation visualization — out of milestone
- Affinity score threshold / exposing numeric overlap scores — Phase 15 D-06/D-09; do not add here
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REL-01 | User selecting an indexed Library source sees a related-docs panel with loading, empty, and error states | Selection state + panel UI; thin IPC to `related_sources`; Chinese copy D-10..D-12; `data-testid`s for loading/empty/error |
| REL-03 | User can click a related result to navigate/select that source in Library | Related-row click → set selection + `scrollIntoView` + refetch (D-13..D-15) |
| REL-04 | Each related result shows title, kind label, and short overlap snippet/reason (not raw scores) | Render `RelatedSource.{title,kind,snippet}` via `sourceKindLabel`; never surface scores (Phase 15 D-09 — field absent) |
</phase_requirements>

## Summary

Phase 15 already delivered `retriever::related_sources` + `RelatedSource { source_id, title, kind, snippet }` with no score field and no serde. Phase 16 is the **consumer half**: add Serialize + a thin Tauri command, wire Library selection + related panel (loading / empty / error), navigate-on-click with last-wins stale-response discard, and ship a focused WebDriver journey. No new npm/crates beyond `serde` on the `retriever` crate for the DTO.

Library today is a single-column glass panel with a scrollable `source-list` and per-row action buttons — **no selection model**. The cleanest layout (discretion D-01) is an **under-list related panel** that mounts only when a source is selected (D-02), with independent overflow scroll (D-03) and the same `motion` + `useReducedMotion` pattern already used by `LibraryView`.

**Critical planner gap:** E2E currently seeds **one** fixture (`e2e/fixtures/sample.md`). Related-chain navigate needs **≥2 indexed sources with FTS/vector overlap** under MockEmbedder. Wave 0 must extend E2E seeding (second overlapping markdown) before the navigate assertion can be green.

**Primary recommendation:** Add `Serialize` (+ `serde` dep) on `RelatedSource`, thin `list_related_sources` in `commands/library.rs`, selection + panel in `LibraryView` / `useLibrary`, Vitest for selection/copy states, `e2e/specs/related-docs.spec.ts` + dual fixture seed — no graph UI, no scores, no new frontend packages.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Overlap ranking | API / Backend (`retriever`) | Database via `store` only | Already GREEN in Phase 15; Phase 16 must not reimplement |
| Tauri IPC wrap | API / Backend (`src-tauri` commands) | — | Thin `async` command: store + embedder → `related_sources` → JSON |
| Source selection UI | Browser / Client (`LibraryView`) | Hook (`useLibrary`) | FE-only state; no schema change |
| Related panel render | Browser / Client | — | title + `sourceKindLabel(kind)` + snippet; loading/empty/error |
| Navigate / scroll / refetch | Browser / Client | API / Backend (refetch IPC) | Select neighbor id → scrollIntoView → invoke again |
| Last-wins cancel | Browser / Client | — | Ignore stale invoke results (generation counter); Tauri cannot abort in-flight Rust easily |
| E2E dual-source fixture | API / Backend (`e2e.rs` seed) | CDN/Static fixtures on disk | MockEmbedder + FTS needs shared keyword across ≥2 indexed files |
| Focused E2E journey | Browser / Client (WebDriver) | — | D-16; Phase 19 still owns trust/full-ui gate |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `retriever::related_sources` | workspace path crate | Overlap API | Phase 15 contract — reuse only [VERIFIED: crates/retriever/src/related.rs] |
| Tauri 2 commands | `tauri = "2"` | IPC | Existing thin-command pattern in `commands/library.rs` [VERIFIED: src-tauri/Cargo.toml] |
| React 19 + TypeScript | `react ^19.1.0`, `typescript ~5.8.3` | Library UI | Existing FE stack [VERIFIED: package.json] |
| `@tauri-apps/api` | `^2` | `invoke` wrappers | Existing `src/lib/tauri.ts` [VERIFIED: package.json] |
| `motion` (`motion/react`) | `^12.40.0` | Panel appear animation | Already used in `LibraryView` [VERIFIED: package.json + LibraryView.tsx] |
| `@tabler/icons-react` | `^3.44.0` | Kind icons (optional reuse) | Existing Library icons [VERIFIED: package.json] |
| Tailwind CSS v4 | `^4.3.0` | Glass / cyan highlight | Existing product UI [VERIFIED: package.json] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde` (retriever crate) | workspace `1` | Serialize `RelatedSource` for IPC | Required — Phase 15 deferred serde [VERIFIED: 15-02-SUMMARY.md] |
| Vitest 3 + Testing Library | `vitest ^3.2.4`, `@testing-library/react ^16.3.2` | Selection / copy / panel state unit tests | Alongside E2E [VERIFIED: package.json + LibraryView.test.tsx] |
| WebdriverIO 9 + `@wdio/tauri-service` | `^9.20.0` / `^1.1.0` | Focused related-docs E2E | D-16 / e2e-required [VERIFIED: package.json] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Under-list related panel | Side column | Side column forces Library layout rewrite — violates D-01 minimize churn |
| Generation counter for last-wins | `AbortController` | No AbortController usage in app today; Tauri invoke is not fetch-abortable for Rust work — counter is enough [VERIFIED: src/ grep] |
| Skip IPC for non-indexed | Call `related_sources` anyway | Backend may return `Ok([])` for blank seed text or neighbors; product needs D-11 distinct copy — **skip IPC** is cleaner (discretion) |
| New graph UI lib | — | Explicitly deferred / out of scope |

**Installation:**

```bash
# No new npm packages.
# Rust: add serde to crates/retriever/Cargo.toml only:
#   serde = { workspace = true }
```

**Version verification:** Versions above taken from workspace `package.json` / `Cargo.toml` / crate manifests on 2026-07-26. No registry installs required for this phase.

## Package Legitimacy Audit

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| — | — | — | — | — | — | **No new packages** |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** none  

Phase 16 uses only already-pinned workspace deps. Adding `serde` to `retriever` is a workspace-path dependency already approved for every other crate — not a new registry package. [VERIFIED: crates/retriever/Cargo.toml currently lacks serde; workspace pins serde elsewhere]

## Architecture Patterns

### System Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│ User: Library view                                                │
│  click row blank → select/toggle                                  │
│  click related row → select neighbor + scroll + refetch           │
└───────────────┬──────────────────────────────────────────────────┘
                │
                v
┌──────────────────────────────────────────────────────────────────┐
│ React: LibraryView + useLibrary                                   │
│  selectedSourceId: string | null                                  │
│  related: { status: idle|loading|ready|error, items[], detail? }  │
│  fetchSeq: number  (last-wins: ignore stale)                      │
│  render: panel hidden if null; else loading/empty/error/list      │
└───────────────┬──────────────────────────────────────────────────┘
                │ invoke("list_related_sources", { sourceId, topN? })
                v
┌──────────────────────────────────────────────────────────────────┐
│ Tauri: commands/library.rs (thin)                                 │
│  state.store + state.embedder() → retriever::related_sources      │
│  map Err → String                                                 │
└───────────────┬──────────────────────────────────────────────────┘
                │
                v
┌──────────────────────────────────────────────────────────────────┐
│ crates/retriever::related_sources (Phase 15 — DO NOT FORK)        │
│  get_source → seed_query → retrieve (hybrid) → rollup → top_n     │
│  RelatedSource { source_id, title, kind, snippet }  // no score   │
└───────────────┬──────────────────────────────────────────────────┘
                │
                v
┌──────────────────────────────────────────────────────────────────┐
│ crates/store (sole SQLite owner)                                  │
└──────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
crates/retriever/
  Cargo.toml                 # + serde workspace dep
  src/related.rs             # + Serialize on RelatedSource
src-tauri/src/
  commands/library.rs        # + list_related_sources
  lib.rs                     # register command in generate_handler!
  e2e.rs                     # + second overlapping fixture seed (Wave 0)
src/
  types/library.ts           # + RelatedSource interface
  lib/tauri.ts               # + listRelatedSources()
  hooks/useLibrary.ts        # optional: related fetch helpers OR keep in view
  views/LibraryView.tsx      # selection + panel + scroll
  views/LibraryView.test.tsx # selection / copy / panel states
e2e/
  fixtures/sample.md         # existing
  fixtures/related-neighbor.md  # NEW — share FTS keyword with sample
  specs/related-docs.spec.ts    # NEW focused journey
  helpers.ts                 # reuse openNav / clickViaDom
```

### Pattern 1: Thin Tauri command over crate API
**What:** Async command takes `source_id` (+ optional `top_n`), locks nothing beyond existing `AppState` accessors, calls `related_sources`, maps errors to `String`.  
**When to use:** All Library/insights IPC today (`summarize_source_cmd`, etc.).  
**Example:** See Code Examples — mirror `summarize_source_cmd` in `commands/library.rs`. [VERIFIED: src-tauri/src/commands/library.rs]

### Pattern 2: FE selection + generation counter (last-wins)
**What:** Keep `selectedSourceId` and monotonic `fetchSeq`. On each select/refetch, increment seq; when invoke resolves, apply only if `seq === fetchSeq`.  
**When to use:** D-15 rapid clicks / related-chain browsing.  
**Why not AbortController:** App has no AbortController pattern; Tauri invoke does not cancel Rust work. Ignoring stale JSON is sufficient. [VERIFIED: no AbortController in src/]

### Pattern 3: Non-indexed = skip IPC, show D-11
**What:** If `source.status !== "indexed"`, set panel to empty/non-indexed state with copy D-11 and **do not** call IPC.  
**When to use:** D-06 + discretion.  
**Why:** Backend `related_sources` does not encode “not indexed” as a dedicated error — blank title/summary returns `Ok([])`, which would collide with D-10 empty copy. [VERIFIED: related.rs seed_query → Ok([])]

### Pattern 4: E2E dual fixture for MockEmbedder overlap
**What:** Seed a second markdown that shares a distinctive token with `sample.md` (e.g. `xyzzy-plugh`) so FTS ranks a neighbor under Mock embedder.  
**When to use:** Navigate E2E (REL-03). Solo seed returns `[]` (Phase 15 `related_only_seed`). [VERIFIED: related.rs tests + e2e.rs single seed]

### Anti-Patterns to Avoid
- **Reimplementing overlap in FE or SQL:** Use Phase 15 API only.
- **Exposing scores / affinity %:** Field does not exist; do not add.
- **Side-column redesign of Library:** Violates D-01 minimize churn.
- **Placeholder panel when nothing selected:** Violates D-02.
- **Selecting via action buttons:** Violates D-05 — `stopPropagation` on button column.
- **Graph / force-directed UI packages:** Deferred / out of scope.
- **Opening SQLite outside `store`:** Project invariant.
- **Relying on Phase 19 for all E2E:** D-16 requires focused journey this phase.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Source overlap ranking | Custom cosine / SQL join | `retriever::related_sources` | Already tested hybrid path [VERIFIED: 15-02-SUMMARY] |
| Kind labels | Ad-hoc Chinese strings | `sourceKindLabel` | Centralized labels incl. `wiki_page` [VERIFIED: sourceDisplay.ts] |
| Motion / reduced-motion | Custom CSS timers | `motion` + `useReducedMotion` | Existing Library pattern [VERIFIED: LibraryView.tsx] |
| IPC invoke typing | Raw `invoke` in view | `src/lib/tauri.ts` wrapper | Project convention |
| E2E React clicks | `element.click()` alone | `clickViaDom` / helpers | Controlled React + WebDriver quirks [VERIFIED: e2e/helpers.ts] |
| Snippet truncation | FE re-truncate | Trust backend 160-char snippet | Phase 15 D-08 [VERIFIED: RELATED_SNIPPET_CHARS = 160] |

**Key insight:** Phase 16 is almost entirely **wiring + UX state**. The ranking algorithm is done; overbuilding UI or re-scoring will fight locked CONTEXT.

## Common Pitfalls

### Pitfall 1: Single E2E fixture → empty related forever
**What goes wrong:** Select indexed sample → panel always shows 「暂无相关文档」; navigate test cannot pass.  
**Why it happens:** `seed_e2e_fixture` indexes only `JARVIS_E2E_FIXTURE` [VERIFIED: e2e.rs].  
**How to avoid:** Wave 0 — add `related-neighbor.md` (shared keyword) and seed it in `seed_e2e_fixture` (or sibling seed).  
**Warning signs:** E2E empty assertion passes but click-related step never finds a row.

### Pitfall 2: Non-indexed IPC maps to wrong empty copy
**What goes wrong:** Pending/failed selection shows D-10 instead of D-11.  
**Why it happens:** `Ok([])` from empty seed query looks like “no neighbors.”  
**How to avoid:** Gate on `status === "indexed"` before invoke; use dedicated empty reason enum in FE state.  
**Warning signs:** Vitest/E2E sees 「暂无相关文档」 for pending source.

### Pitfall 3: Action buttons toggle selection
**What goes wrong:** Clicking 生成摘要 also selects/deselects.  
**Why it happens:** Click bubbles from button to row.  
**How to avoid:** Row blank handler on left content / `li` background; `stopPropagation` on action button container (D-05).  
**Warning signs:** Flaky E2E when clicking summarize.

### Pitfall 4: Stale related list after rapid navigate
**What goes wrong:** Click A→B→C; panel shows A's neighbors after C selected.  
**Why it happens:** No last-wins guard (D-15).  
**How to avoid:** Generation counter; ignore mismatched seq.  
**Warning signs:** Panel title/seed mismatch vs cyan-highlighted row.

### Pitfall 5: Serde / field naming mismatch
**What goes wrong:** FE reads `undefined` for `sourceId` / `source_id`.  
**Why it happens:** Store `Source` is snake_case; many shell DTOs use `rename_all = "camelCase"`; command **args** are camelCase by Tauri default. [CITED: https://v2.tauri.app/develop/calling-rust/]  
**How to avoid:** Pick one for `RelatedSource` payload (recommend **camelCase** Serialize to match `InsightsReport`) and mirror in TS; invoke args `{ sourceId, topN }`.  
**Warning signs:** Empty titles in panel despite 200 IPC.

### Pitfall 6: Missing command registration
**What goes wrong:** `invoke` fails with command not found.  
**Why it happens:** Added fn but forgot `generate_handler!` in `lib.rs`. [VERIFIED: src-tauri/src/lib.rs handler list]  
**How to avoid:** Register beside `list_sources` / `summarize_source_cmd`.

### Pitfall 7: Adding score UI “just for debug”
**What goes wrong:** Violates REL-04 / Phase 15 D-09.  
**How to avoid:** Render only title, kind label, snippet.

## Code Examples

### RelatedSource DTO (add serde — Phase 15 deferred)

```rust
// Source: crates/retriever/src/related.rs (current fields verified; Serialize to add)
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedSource {
    pub source_id: String,
    pub title: String,
    pub kind: SourceKind, // already Serialize snake_case enum
    pub snippet: String,
}
```

Also add to `crates/retriever/Cargo.toml`: `serde = { workspace = true }`. [ASSUMED: workspace serde features already include `derive` — verify against root Cargo.toml when implementing]

### Thin Tauri command (prescribed)

```rust
// Pattern from: src-tauri/src/commands/library.rs::summarize_source_cmd
#[tauri::command]
pub async fn list_related_sources(
    source_id: String,
    top_n: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<retriever::RelatedSource>, String> {
    retriever::related_sources(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &source_id,
        top_n,
    )
    .await
    .map_err(|e| e.to_string())
}
```

TS wrapper (args camelCase per Tauri default) [CITED: https://v2.tauri.app/develop/calling-rust/]:

```typescript
// src/lib/tauri.ts
export function listRelatedSources(sourceId: string, topN?: number | null) {
  return invoke<RelatedSource[]>("list_related_sources", {
    sourceId,
    topN: topN ?? null,
  });
}
```

### Library selection + panel states (prescribed shape)

```tsx
// Conceptual — match LibraryView motion pattern
const reduceMotion = useReducedMotion();
// selectedId: string | null
// relatedStatus: "loading" | "empty" | "nonindexed" | "error" | "ready"
{selectedId && (
  <motion.div
    data-testid="related-docs-panel"
    initial={reduceMotion ? false : { opacity: 0, y: 6 }}
    animate={{ opacity: 1, y: 0 }}
    className="… overflow-auto …" // D-03 independent scroll
  >
    {relatedStatus === "loading" && (
      <p data-testid="related-docs-loading">加载相关文档…</p>
    )}
    {relatedStatus === "nonindexed" && (
      <p data-testid="related-docs-empty">仅已索引来源可查看相关文档</p>
    )}
    {relatedStatus === "empty" && (
      <p data-testid="related-docs-empty">暂无相关文档</p>
    )}
    {relatedStatus === "error" && (
      <p data-testid="related-docs-error">相关文档加载失败</p>
    )}
    {relatedStatus === "ready" && (
      <ul data-testid="related-docs-list">
        {items.map((r) => (
          <li
            key={r.sourceId}
            data-testid={`related-docs-row-${r.sourceId}`}
            onClick={() => selectAndScroll(r.sourceId)}
          >
            <div>{r.title}</div>
            <div>{sourceKindLabel(r.kind)}</div>
            <div>{r.snippet}</div>
          </li>
        ))}
      </ul>
    )}
  </motion.div>
)}
```

### Scroll + reduced motion (D-14)

```typescript
rowEl?.scrollIntoView({
  behavior: reduceMotion ? "auto" : "smooth",
  block: "nearest",
});
```

### Recommended `data-testid`s (discretion — lock in plan)

| Element | testid |
|---------|--------|
| Panel root | `related-docs-panel` |
| Loading | `related-docs-loading` |
| Empty / non-indexed | `related-docs-empty` (distinguish via text assert) |
| Error | `related-docs-error` |
| List | `related-docs-list` |
| Related row | `related-docs-row-{sourceId}` |
| Source row (select/scroll target) | `source-row-{id}` |
| Selected marker | `aria-selected="true"` on source row **or** class + `data-selected="true"` |

### Existing Library motion pattern (reuse)

```102:108:src/views/LibraryView.tsx
    <motion.section
      key="library"
      className="glass-panel flex flex-1 flex-col gap-4 p-5"
      initial={reduceMotion ? false : { opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      exit={reduceMotion ? undefined : { opacity: 0, y: -6 }}
      transition={{ duration: 0.18 }}
```

### Phase 15 API signature (do not change semantics)

```56:64:crates/retriever/src/related.rs
/// Rank other indexed sources by hybrid retrieval overlap with the seed.
///
/// Reuses `retrieve` (vector+FTS+RRF). No score threshold (D-06) — weak hits may appear.
pub async fn related_sources(
    store: &Store,
    embedder: &dyn Embedder,
    source_id: &str,
    top_n: Option<usize>,
) -> Result<Vec<RelatedSource>> {
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No related-docs | `related_sources` crate API | Phase 15 (2026-07-25) | Phase 16 only wires UI/IPC |
| Library flat action list | Selection + related panel | Phase 16 (this) | FE state only |
| Graph / second vector DB | Hybrid retrieve rollup | v1.11 research | Do not introduce graph crates |

**Deprecated/outdated:**
- Frontend-only “related” heuristics — rejected by milestone research.
- Exposing raw RRF scores in UI — locked out (Phase 15 D-09 / REL-04).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Workspace `serde` already enables `derive` for path crates | Code Examples | Need explicit features in retriever Cargo.toml |
| A2 | Second fixture sharing `xyzzy-plugh` is enough for MockEmbedder related hits in E2E | Pitfall 1 / Validation | May need stronger shared body text or summary set like Phase 15 tests |
| A3 | Under-list panel is the lowest-churn layout vs under-row expand | Architecture Patterns | User may prefer under-row; still within D-01 discretion |

## Open Questions

1. **E2E empty vs navigate fixture strategy**
   - What we know: One seeded file today; solo seed → empty related [VERIFIED].
   - What's unclear: Whether to always seed neighbor in all E2E runs (affects library-stats counts in other specs) or gate on env.
   - Recommendation: Always seed a small second fixture; update any brittle “exactly N sources” assertions if present (wiki/full-ui mostly use “已索引” substring / presence checks — verify during implement).

2. **Where selection state lives**
   - What we know: `useLibrary` has no selection; `App.tsx` passes props into `LibraryView` [VERIFIED].
   - What's unclear: Keep selection entirely inside `LibraryView` (simpler) vs lift to `useLibrary`.
   - Recommendation: **Keep selection + related fetch state in `LibraryView`** (or a tiny `useRelatedDocs` hook colocated) to avoid bloating `App.tsx` / `useLibrary` action surface — unless scroll/refetch needs shared busy flag.

## Environment Availability

Step 2.6: **Partial** — phase is code/config; E2E needs existing Windows tooling already used by CI.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable / cargo | IPC + serde | ✓ (project toolchain) | stable MSRV 1.85 | — |
| Node 22 + npm | Vitest / E2E | ✓ (CI pin) | project | — |
| `tauri-driver` + msedgedriver | D-16 E2E | ✓ when local e2e set up | per e2e/wdio.conf | CI Windows job |
| Live LLM / Feishu | — | N/A | — | Must use `JARVIS_E2E=1` mocks |

**Missing dependencies with no fallback:** none for planning  
**Missing dependencies with fallback:** none

## Validation Architecture

> `workflow.nyquist_validation: true` in `.planning/config.json` [VERIFIED]

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Vitest 3.2.x (FE unit) + `cargo test` (Rust) + WebdriverIO 9 / Mocha (E2E) |
| Config file | `vite.config.ts` (Vitest); `e2e/wdio.conf.ts` (E2E) |
| Quick run command | `npx vitest run src/views/LibraryView.test.tsx` |
| Full suite command | `cargo test -p retriever` && `npm test` && `npm run test:e2e:local` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REL-01 | Select indexed → loading then ready/empty; panel hidden when none | unit (Vitest) | `npx vitest run src/views/LibraryView.test.tsx` | ❌ extend `LibraryView.test.tsx` |
| REL-01 | Non-indexed select → D-11 copy, no success list | unit (Vitest) | same | ❌ Wave 0 |
| REL-01 | IPC error → D-12 error testid | unit (Vitest mock invoke) | same / hook test | ❌ Wave 0 |
| REL-04 | Row shows title + kind label + snippet; no score | unit (Vitest) | same | ❌ Wave 0 |
| REL-03 | Click related → selection moves + scroll target present | unit (jsdom scrollIntoView mock) | same | ❌ Wave 0 |
| REL-01/03 | Focused journey under mocks | E2E | `npm run test:e2e:local` (or wdio filter related-docs) | ❌ `e2e/specs/related-docs.spec.ts` |
| REL-02 regression | Backend unchanged | unit/integration | `cargo test -p retriever related` | ✅ Phase 15 |
| TRUST-* | Full citation gate | E2E | Phase 19 | ⏭️ deferred |

### Sampling Rate
- **Per task commit:** `npx vitest run src/views/LibraryView.test.tsx` (+ `cargo test -p retriever` if serde touch)
- **Per wave merge:** `npm test` + `cargo test -p retriever`
- **Phase gate:** `npm run test:e2e:local` green for related-docs journey; existing wiki/qa not required green-blocked by Phase 19 but should not be knowingly broken

### Wave 0 Gaps
- [ ] `e2e/fixtures/related-neighbor.md` — shares keyword with `sample.md` (`xyzzy-plugh`)
- [ ] `src-tauri/src/e2e.rs` — seed second fixture in `seed_e2e_fixture`
- [ ] `e2e/specs/related-docs.spec.ts` — select → panel → click navigate
- [ ] Extend `LibraryView.test.tsx` — selection toggle, D-10/D-11/D-12, related row content
- [ ] `data-testid="source-row-{id}"` (or equivalent) for scroll/select E2E
- [ ] Update `.cursor/rules/e2e-required.mdc` spec map row for Related-docs → `related-docs.spec.ts` (or note in plan for Phase 19 if map update deferred — **prefer update this phase** per e2e-required)

## Security Domain

> Local desktop app; `security_enforcement` not disabled — include ASVS skim.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Single-user local app |
| V3 Session Management | no | — |
| V4 Access Control | no | No multi-user ACL |
| V5 Input Validation | yes | `source_id` string → `Store::get_source` NotFound → `String` error; `top_n` clamped in crate (`min(32)`) [VERIFIED: related.rs] |
| V6 Cryptography | no | — |

### Known Threat Patterns for Tauri + local KB

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unexpected `source_id` | Tampering | Store NotFound → IPC error string; FE shows D-12 |
| Path/SQLite escape via related IPC | Elevation | Command only calls crate API; store remains sole DB owner |
| XSS via snippet in panel | Tampering | React text nodes (no `dangerouslySetInnerHTML`) — keep snippet as text |

## Project Constraints (from .cursor/rules/)

| Rule | Directive for Phase 16 |
|------|------------------------|
| `e2e-required.mdc` | User-facing → add/extend `e2e/specs/*`, stable `data-testid`s, `JARVIS_E2E=1` mocks, `npm run test:e2e:local` green |
| `tdd-goal-driven.mdc` | Goal → failing tests → implement → refactor; bugfix = regression test |
| `frontend-taste.mdc` | Glass panel, cyan accent, loading/empty/error, honor reduced motion/transparency; read design-taste skill before UI |
| `jarvis-stack.mdc` | No stack change; thin Tauri commands; logic in crates; React/Vite/Tailwind/Motion/Tabler |
| `karpathy-guidelines.mdc` | Minimal surgical diff; no speculative abstractions; state assumptions |

## Sources

### Primary (HIGH confidence)
- `crates/retriever/src/related.rs` — DTO + `related_sources` behavior
- `15-02-SUMMARY.md` — serde deferred to Phase 16; REL-02 GREEN
- `src/views/LibraryView.tsx` / `useLibrary.ts` / `src/lib/tauri.ts` — FE seams
- `src-tauri/src/commands/library.rs` + `lib.rs` handler list — IPC pattern
- `src-tauri/src/e2e.rs` + `e2e/fixtures/sample.md` — single fixture gap
- `.planning/config.json` — nyquist + research enabled
- `.cursor/rules/*` — E2E/TDD/taste/stack

### Secondary (MEDIUM confidence)
- [CITED: https://v2.tauri.app/develop/calling-rust/] — camelCase command args
- `.planning/research/ARCHITECTURE.md` / `SUMMARY.md` — `list_related_sources` naming + panel shape

### Tertiary (LOW confidence)
- Exact second-fixture body text for reliable MockEmbedder neighbor ranking in packaged E2E (validate during Wave 0)

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — reuse existing pins; no new packages
- Architecture: **HIGH** — Phase 15 API + clear Library seams
- Pitfalls: **HIGH** — dual-fixture + non-indexed copy + last-wins verified as real gaps
- E2E fixture keyword strategy: **MEDIUM** — needs implement-time confirmation

**Research date:** 2026-07-26  
**Valid until:** 2026-08-25 (stable desktop stack; re-check if Library monolith moves)
