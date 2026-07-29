# Phase 16: Related-docs Library panel - Pattern Map

**Mapped:** 2026-07-28
**Files analyzed:** 14 new/modified (+ 1 optional hook)
**Analogs found:** 12 / 14 exact or role-match; 2 partial (generation counter, stopPropagation)

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/retriever/Cargo.toml` | config | transform | `crates/store/Cargo.toml` (serde workspace dep) | exact |
| `crates/retriever/src/related.rs` | model | transform | `src-tauri/src/insights_ops.rs` (`InsightsReport` Serialize) | exact |
| `src-tauri/src/commands/library.rs` | controller | request-response | same file `summarize_source_cmd` | exact |
| `src-tauri/src/commands/mod.rs` | config | — | same file `library` re-exports | exact |
| `src-tauri/src/lib.rs` | config | — | same file `generate_handler!` block | exact |
| `src-tauri/src/e2e.rs` | config / test helper | file-I/O + batch | same file `seed_e2e_fixture` | exact |
| `src/types/library.ts` | model | — | same file `InsightsReport` interface | exact |
| `src/lib/tauri.ts` | utility / service | request-response | same file `summarizeSource` | exact |
| `src/views/LibraryView.tsx` | component | request-response + event-driven | `MemoryView.tsx` + existing `LibraryView` | exact |
| `src/views/LibraryView.test.tsx` | test | — | same file wiki compile/export tests | exact |
| `e2e/fixtures/related-neighbor.md` | config / fixture | file-I/O | `e2e/fixtures/sample.md` | exact |
| `e2e/specs/related-docs.spec.ts` | test | request-response (UI journey) | `e2e/specs/memory.spec.ts` | exact |
| `e2e/helpers.ts` | utility | — | same file (reuse only; no change required) | exact |
| `.cursor/rules/e2e-required.mdc` | config | — | same file spec map table | exact |
| `src/hooks/useRelatedDocs.ts` *(optional)* | hook | request-response | `src/hooks/useLibrary.ts` | partial — selection may stay in view |

## Pattern Assignments

### `crates/retriever/Cargo.toml` (config)

**Analog:** workspace `Cargo.toml` + peer crates using `serde`

**Workspace dep pattern** (root `Cargo.toml` already pins `serde`):

```toml
serde = { workspace = true }
```

Add under `[dependencies]` beside `thiserror`. No new registry packages.

---

### `crates/retriever/src/related.rs` (model, transform)

**Analog:** `src-tauri/src/insights_ops.rs` (camelCase IPC DTO) + `crates/store/src/types.rs` (`SourceKind` already serializes)

**Serialize on DTO** (mirror `InsightsReport`):

```6:12:src-tauri/src/insights_ops.rs
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InsightsReport {
    pub summarized: usize,
    pub tasks_extracted: usize,
    pub failed: usize,
}
```

**Apply to `RelatedSource`** — add `use serde::Serialize;`, derive `Serialize`, `#[serde(rename_all = "camelCase")]` on struct at lines 10–16. Do **not** change `related_sources` semantics (Phase 15 contract).

**Dual-fixture overlap pattern for E2E fixture design** (from integration tests):

```192:223:crates/retriever/src/related.rs
                "# Neighbor Alpha\n\nBody shares {OVERLAP_TOKEN} for FTS overlap.\n"
// ...
            .set_source_summary(&seed_id, &format!("Summary mentions {OVERLAP_TOKEN}"))
```

Reuse `xyzzy-plugh` from `sample.md` in neighbor body + set seed summary mentioning it after index (or mirror test token strategy).

---

### `src-tauri/src/commands/library.rs` (controller, request-response)

**Analog:** `summarize_source_cmd` in same file

**Thin async command + embedder access** (lines 30–42):

```30:42:src-tauri/src/commands/library.rs
#[tauri::command]
pub async fn summarize_source_cmd(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    insights::summarize_source(
        state.store.as_ref(),
        state.chat().as_ref(),
        &source_id,
    )
    .await
    .map_err(|e| e.to_string())
}
```

**New command shape:** `list_related_sources(source_id, top_n: Option<usize>, state)` → `retriever::related_sources(state.store.as_ref(), state.embedder().as_ref(), &source_id, top_n).await.map_err(|e| e.to_string())`. `retriever` already in `src-tauri/Cargo.toml`.

---

### `src-tauri/src/commands/mod.rs` (config)

**Analog:** existing `library` re-export block (lines 26–29)

```26:29:src-tauri/src/commands/mod.rs
pub use library::{
    delete_task, extract_tasks_cmd, list_sources, list_tasks, remove_source, run_insights_all_cmd,
    source_count, summarize_source_cmd, update_task_status,
};
```

Add `list_related_sources` to the `pub use library::{ ... }` list.

---

### `src-tauri/src/lib.rs` (config)

**Analog:** `generate_handler!` list beside `summarize_source_cmd`

**Import + register** (lines 20–23 import pattern; lines 95–96 handler slot):

```95:96:src-tauri/src/lib.rs
            summarize_source_cmd,
            extract_tasks_cmd,
```

Insert `list_related_sources` adjacent to other library commands. Import from `commands` module alongside `summarize_source_cmd`.

---

### `src-tauri/src/e2e.rs` (config, file-I/O + batch)

**Analog:** `seed_e2e_fixture` (lines 30–51)

```30:51:src-tauri/src/e2e.rs
pub fn seed_e2e_fixture(state: &AppState) -> Result<(), String> {
    let fixture = std::env::var("JARVIS_E2E_FIXTURE")
        .map_err(|_| "JARVIS_E2E_FIXTURE is not set".to_string())?;
    if !Path::new(&fixture).is_file() {
        return Err(format!("e2e fixture missing: {fixture}"));
    }

    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(index_path(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &fixture,
    ))
    .map_err(|e| e.to_string())?;
    // ...
    Ok(())
}
```

**Extend:** After primary fixture index, also `index_path` on `e2e/fixtures/related-neighbor.md` (path relative to repo or env). Optionally `set_source_summary` on seed id with shared keyword so MockEmbedder + FTS rank neighbor — mirror `related.rs` test setup.

---

### `src/types/library.ts` (model)

**Analog:** `InsightsReport` in same file (camelCase fields)

```19:23:src/types/library.ts
export interface InsightsReport {
  summarized: number;
  tasksExtracted: number;
  failed: number;
}
```

**Add:**

```typescript
export interface RelatedSource {
  sourceId: string;
  title: string;
  kind: string;
  snippet: string;
}
```

---

### `src/lib/tauri.ts` (utility, request-response)

**Analog:** `summarizeSource` (lines 102–104)

```102:104:src/lib/tauri.ts
export function summarizeSource(sourceId: string) {
  return invoke<string>("summarize_source_cmd", { sourceId });
}
```

**Add import** `RelatedSource` from `../types/library`. **Wrapper:**

```typescript
export function listRelatedSources(sourceId: string, topN?: number | null) {
  return invoke<RelatedSource[]>("list_related_sources", {
    sourceId,
    topN: topN ?? null,
  });
}
```

Command args camelCase per Tauri 2 convention (matches `{ sourceId }` above).

---

### `src/views/LibraryView.tsx` (component, request-response + event-driven)

**Analogs:**
- Layout/motion: existing `LibraryView` (lines 97–108, 110–119, 185–274)
- Empty copy tone: `MemoryView` list empty (line 82)
- Loading copy tone: `SettingsView` config gate (line 150)
- Selection highlight: `src/index.css` `.nav-btn-active` tokens

**Motion + glass shell** (reuse):

```102:108:src/views/LibraryView.tsx
    <motion.section
      key="library"
      className="glass-panel flex flex-1 flex-col gap-4 p-5"
      initial={reduceMotion ? false : { opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      exit={reduceMotion ? undefined : { opacity: 0, y: -6 }}
      transition={{ duration: 0.18 }}
```

**Source list + per-row testids** (extend `<li>`):

```189:193:src/views/LibraryView.tsx
          {sources.map((s) => (
            <li
              key={s.id}
              className="flex items-start justify-between gap-4 px-4 py-3 text-sm"
            >
```

Add: `data-testid={`source-row-${s.id}`}`, `aria-selected={selectedId === s.id}`, `data-selected={selectedId === s.id || undefined}`, cyan highlight classes from `.nav-btn-active` (`bg-cyan-500/15 ring-1 ring-cyan-400/30`), row-blank `onClick` for toggle select (D-08).

**Action button isolation (D-05)** — wrap button column:

```tsx
<div className="flex shrink-0 flex-col gap-2" onClick={(e) => e.stopPropagation()}>
```

No existing `stopPropagation` in codebase; this is the prescribed React pattern.

**Empty state copy** (MemoryView pattern):

```81:82:src/views/MemoryView.tsx
        {memories.length === 0 ? (
          <li className="px-4 py-3 text-sm text-zinc-500">暂无记忆。</li>
```

**Loading gate** (SettingsView pattern):

```138:151:src/views/SettingsView.tsx
  if (!config) {
    return (
      <motion.section
        // ...
        <p className="text-sm text-zinc-500">加载中…</p>
```

**Related panel** (conditional sibling under `source-list`, per 16-UI-SPEC):
- Mount only when `selectedSourceId !== null` (D-02)
- Container: `rounded-xl border border-white/10 bg-zinc-950/40 p-4 max-h-48 overflow-auto`
- `motion.div` with `initial={reduceMotion ? false : { opacity: 0, y: 6 }}`, `data-testid="related-docs-panel"`, `role="region"` + `aria-label="相关文档"`
- States + testids from UI-SPEC: `related-docs-loading`, `related-docs-empty`, `related-docs-error`, `related-docs-list`, `related-docs-row-{sourceId}`
- Row content: `{r.title}`, `{sourceKindLabel(r.kind)}`, `{r.snippet}` — import from `../lib/sourceDisplay`

**Kind label reuse:**

```1:4:src/lib/sourceDisplay.ts
export function sourceKindLabel(kind: string): string {
  switch (kind) {
    case "local_file":
      return "本地文件";
```

**Fetch orchestration (last-wins, D-15)** — no codebase analog; use RESEARCH Pattern 2:

```typescript
const fetchSeq = useRef(0);
// on select: const seq = ++fetchSeq.current; ... invoke ... if (seq !== fetchSeq.current) return;
```

**Non-indexed gate (D-06/D-11):** if `status !== "indexed"`, skip `listRelatedSources`, set `relatedStatus = "nonindexed"`.

**Navigate (D-13/D-14):** on related row click → `setSelectedSourceId(id)` → `document.querySelector(`[data-testid="source-row-${id}"]`)?.scrollIntoView({ behavior: reduceMotion ? "auto" : "smooth", block: "nearest" })` → refetch.

---

### `src/views/LibraryView.test.tsx` (test)

**Analog:** existing wiki compile/export describes in same file

**Vitest + jsdom + motion mock** (lines 1–20):

```1:20:src/views/LibraryView.test.tsx
/**
 * @vitest-environment jsdom
 */
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
// ...
vi.mock("motion/react", () => ({
  motion: {
    section: ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => (
      <section {...props}>{children}</section>
    ),
  },
  useReducedMotion: () => true,
}));
```

**Fixture source objects** (lines 22–31):

```22:31:src/views/LibraryView.test.tsx
const indexedLocal = {
  id: "src-1",
  kind: "local_file",
  uri: "file:///tmp/a.md",
  title: "a.md",
  status: "indexed",
  indexed_at: 1,
  error: null,
  summary: null,
};
```

**Click assertion pattern** (lines 134–138):

```134:138:src/views/LibraryView.test.tsx
  it("clicking 生成笔记 calls onCompileWiki with source id", () => {
    render(<LibraryView {...baseProps} />);
    fireEvent.click(screen.getByTestId("wiki-compile-src-1"));
    expect(baseProps.onCompileWiki).toHaveBeenCalledWith("src-1");
  });
```

**Extend with new `describe("LibraryView related docs")`:**
- Mock `../lib/tauri` → `listRelatedSources: vi.fn()` (pattern from `useLibrary.test.ts` lines 10–45)
- Panel hidden when no selection
- Select indexed → loading then list/empty testids
- Select pending → `related-docs-empty` text 「仅已索引来源可查看相关文档」
- Select indexed, mock `[]` → 「暂无相关文档」
- Mock error → `related-docs-error`
- Ready row shows title + kind label + snippet; no score element
- Toggle: second click same row hides panel
- Action button click does not select (click `summarize-*`, panel stays hidden)
- Optional: mock `scrollIntoView` on `Element.prototype`

---

### `e2e/fixtures/related-neighbor.md` (fixture, file-I/O)

**Analog:** `e2e/fixtures/sample.md`

```1:7:e2e/fixtures/sample.md
# Jarvis E2E Fixture

This document is seeded automatically when `JARVIS_E2E=1`.

The secret keyword for automated tests is **xyzzy-plugh**.

Jarvis indexes this file so WebDriver can ask a question and receive a Mock answer with citations.
```

**New file:** Include `xyzzy-plugh` in body so FTS overlap with seed fixture. Keep short; distinct title for E2E text asserts.

---

### `e2e/specs/related-docs.spec.ts` (test, UI journey)

**Analog:** `e2e/specs/memory.spec.ts` (focused CRUD journey)

**Nav + testid waits** (memory.spec.ts lines 7–21):

```7:21:e2e/specs/memory.spec.ts
  it("adds, edits, and forgets a manual memory", async () => {
    const unique = `E2E-${Date.now()}`;
    // ...
    await openNav("memory");

    const input = await $('[data-testid="new-memory-input"]');
    await input.setValue(memoryText);
    await $('[data-testid="add-memory-submit"]').click();

    const list = await $('[data-testid="memory-list"]');
    await browser.waitUntil(
      async () => (await list.getText()).includes(unique),
      { timeout: 30_000, timeoutMsg: "memory not listed after add" },
    );
```

**Library nav pattern** (wiki.spec.ts lines 52–67):

```52:67:e2e/specs/wiki.spec.ts
    await openNav("library", '[data-testid="library-stats"]');
    await browser.waitUntil(
      async () =>
        (await $('[data-testid="library-stats"]').getText()).match(
          /已索引 [1-9]/,
        ) !== null,
      {
        timeout: 60_000,
        timeoutMsg: "seeded fixture not indexed yet",
      },
    );
```

**Journey outline:**
1. `openNav("library", '[data-testid="library-stats"]')` — wait indexed ≥ 2 after dual seed
2. `clickViaDom('[data-testid^="source-row-"]')` on first indexed source (helpers line 160–165)
3. Wait `related-docs-panel` displayed → `related-docs-loading` gone → `related-docs-list` or assert rows
4. `clickViaDom('[data-testid^="related-docs-row-"]')` → assert target `source-row-*` has `aria-selected="true"` / `data-selected="true"`
5. Panel refetches (still visible; list may change)
6. Optional: deselect via second click same row → panel absent

Use `clickViaDom` not raw WebDriver click for React handlers:

```160:165:e2e/helpers.ts
/** Click via DOM — WebDriver elementClick often misses React onClick handlers. */
export async function clickViaDom(selector: string) {
  await browser.execute((sel) => {
    (document.querySelector(sel) as HTMLElement | null)?.click();
  }, selector);
}
```

---

### `e2e/helpers.ts` (utility — reuse only)

**Analog:** self — import `openNav`, `clickViaDom`, `waitForDomText` in new spec. No file change required unless shared helper for related-panel wait emerges.

---

### `.cursor/rules/e2e-required.mdc` (config)

**Analog:** spec map table (lines 26–38)

Add row: `Related-docs | related-docs.spec.ts`. Per e2e-required policy, optionally extend `full-ui.spec.ts` only if related-docs becomes primary journey (Phase 16 = focused spec only per D-16).

---

## Shared Patterns

### Thin Tauri IPC (store + provider → crate → String errors)

**Source:** `src-tauri/src/commands/library.rs`  
**Apply to:** `list_related_sources`

```17:19:src-tauri/src/commands/library.rs
pub fn list_sources(state: State<'_, AppState>) -> Result<Vec<Source>, String> {
    state.store.list_sources().map_err(|e| e.to_string())
}
```

Async variant uses `state.embedder().as_ref()` like `summarize_source_cmd` uses `state.chat()`.

### IPC TypeScript wrappers

**Source:** `src/lib/tauri.ts`  
**Apply to:** `listRelatedSources`

- Import types from `../types/library`
- `invoke<ReturnType>("snake_case_cmd", { camelCaseArgs })`
- Args: `{ sourceId }`, optional `{ topN: topN ?? null }`

### camelCase serde DTOs across IPC boundary

**Source:** `src-tauri/src/insights_ops.rs` + `src/types/library.ts` `InsightsReport`  
**Apply to:** `RelatedSource` Rust + TS interfaces

### Glass panel + motion + reduced motion

**Source:** `LibraryView.tsx`, `MemoryView.tsx`, `SettingsView.tsx`  
**Apply to:** related panel mount animation; `useReducedMotion()` for panel + scroll

### Chinese empty / loading copy

**Source:** `LibraryView` 「暂无来源。」, `SettingsView` 「加载中…」, `MemoryView` 「暂无记忆。」  
**Apply to:** locked strings D-10..D-12 in related panel

### Selection highlight (cyan accent)

**Source:** `src/index.css`

```37:38:src/index.css
  .nav-btn-active {
    @apply bg-cyan-500/15 text-cyan-200 ring-1 ring-cyan-400/30;
```

Apply equivalent Tailwind classes on selected `source-row-*`.

### Vitest: mock `../lib/tauri`

**Source:** `src/hooks/useLibrary.test.ts` (lines 10–45)  
**Apply to:** `LibraryView.test.tsx` related-docs tests when invoking `listRelatedSources`

### E2E: Library navigation + indexed wait

**Source:** `e2e/specs/wiki.spec.ts`, `e2e/specs/full-ui.spec.ts`  
**Apply to:** all related-docs spec setup — `openNav("library", ...)`, wait for `library-stats` 已索引

### Kind display

**Source:** `src/lib/sourceDisplay.ts` → `sourceKindLabel`  
**Apply to:** related row kind line (REL-04); never hardcode kind strings

---

## No Analog Found

| File / Pattern | Role | Data Flow | Reason |
|----------------|------|-----------|--------|
| Last-wins `fetchSeq` guard | hook logic | request-response | No generation counter / stale-invoke ignore in `src/` today — implement per RESEARCH Pattern 2 |
| Row `stopPropagation` for action buttons | interaction | event-driven | No existing stopPropagation in codebase — standard React event isolation |
| `src/hooks/useRelatedDocs.ts` *(if extracted)* | hook | request-response | Optional; RESEARCH recommends colocating in `LibraryView` unless view bloats |

---

## Metadata

**Analog search scope:** `crates/retriever/`, `src-tauri/src/commands/`, `src-tauri/src/e2e.rs`, `src/views/`, `src/lib/`, `src/hooks/*.test.ts`, `e2e/specs/`, `e2e/fixtures/`, `src/index.css`  
**Files scanned:** ~20  
**Pattern extraction date:** 2026-07-28

## PATTERN MAPPING COMPLETE

**Phase:** 16 - Related-docs Library panel  
**Files classified:** 14 (+ 1 optional)  
**Analogs found:** 12 / 14 exact or role-match; 2 partial (generation counter, stopPropagation)

### Coverage
- Files with exact analog: 12
- Files with role-match analog: 2 (`LibraryView` selection UX, optional hook)
- Files with no analog: 2 patterns (not files) — last-wins guard, stopPropagation

### Key Patterns Identified
- Thin async Tauri commands delegate to workspace crates and map errors with `.map_err(|e| e.to_string())`
- IPC DTOs use `#[serde(rename_all = "camelCase")]` on Rust + matching TS interfaces; invoke args camelCase
- Library UI uses `motion` + `useReducedMotion`, glass-panel Tailwind, and `data-testid` kebab-case
- E2E uses `openNav`, `clickViaDom`, dual fixture seed, and `JARVIS_E2E=1` Mock providers
- Related overlap logic stays in `retriever::related_sources` — Phase 16 is wiring + UX only

### File Created
`D:\Work\99_Code\02_Rust\Jarvis\.planning\phases\16-related-docs-library-panel\16-PATTERNS.md`

### Ready for Planning
Pattern mapping complete. Planner can now reference analog patterns in PLAN.md files.
