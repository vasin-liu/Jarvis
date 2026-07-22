# Phase 11: Library / Settings UI — Research

**Researched:** 2026-07-22  
**Domain:** React/Tauri FE wiring for WIKI-06 (feature-flagged Settings toggle + Library「生成笔记」)  
**Confidence:** HIGH (codebase-verified integration seams; no new packages)

> Note: Researcher subagent was interrupted twice on Windows; this RESEARCH.md was completed by the plan-phase orchestrator from direct codebase reads so planning can proceed.

---

## User Constraints

*(Copied verbatim from `11-CONTEXT.md` — planner MUST honor.)*

### Settings placement & fields
- **D-01:** Add an **independent AccordionSection** titled **「Wiki 笔记」** (`data-testid="settings-section-wiki"`), sibling to Lark / Agent sections — not nested under「自动洞察」.
- **D-02:** Expose **only** `wiki.enabled` in UI. Do **not** show `auto_on_insights` (avoids implying auto-compile; field stays in config for later).
- **D-03:** Toggle updates local Settings `config` state; persists only when user clicks existing **「保存配置」** (same as most Settings fields — not immediate `handleSaveConfig`).
- **D-04:** When `wiki.enabled` is false after save, **keep `wiki_page` sources visible** in Library (label「笔记页」); only hide wiki compile controls. Closing the feature ≠ deleting content.

### Library「生成笔记」eligibility
- **D-05:** Show「生成笔记」only when **`config.wiki.enabled === true`** and source **`status === "indexed"`** (same gate as 生成摘要 / 提取任务).
- **D-06:** **Never show** the button for `kind === "wiki_page"` (align with Phase 10 hard-reject of WikiPage compile inputs).
- **D-07:** For all **other** kinds (`local_file`, Lark, Cursor, Memory, …), show the button when D-05 holds — no kind allowlist narrower than “non–wiki_page”.
- **D-08:** Place the button in the **same per-source action group** as「生成摘要」/「提取任务」(same column, ghost `text-xs` buttons).

### Compile feedback
- **D-09:** Use existing **global `busy`** — disable Library action buttons while compile runs (same as summarize/extract).
- **D-10:** On failure, surface via existing **Library/App error path** (`reportError` / shared error bar) — no per-button inline error UI.
- **D-11:** On success, **silent refresh** (`refreshSources`) — no toast; new「笔记页」rows are the feedback. Discard/ignore `WikiCompileSummary` for UI display in this phase (return value unused in UX).
- **D-12:** Button remains available for **re-compile** of the same source (backend idempotency / stale cleanup / hash skip). Do not hide after first successful compile.

### Copy & testids
- **D-13:** Settings checkbox label: **「启用 Wiki 笔记层」**.
- **D-14:** Library button label: **「生成笔记」**.
- **D-15:** Accordion title: **「Wiki 笔记」**.
- **D-16:** Stable selectors: `settings-section-wiki`, `wiki-enabled-toggle`, `wiki-compile-{id}` (mirror `summarize-{id}`).

### Claude's Discretion
- Exact AccordionSection helper text / description under the checkbox (keep minimal; Chinese product tone).
- Whether compile invoke lives in `useLibrary` vs thin App handler — follow existing summarize/extract pattern.
- FE config typing: ensure nested `wiki.enabled` round-trips through existing flat↔nested helpers (`src/types/config.ts`) without inventing a second shape.
- Vitest placement (LibraryView props test vs hook test) — follow Phase plan / existing `useLibrary` / view test style.
- Whether Phase 11 adds a minimal E2E assertion or only Vitest (ROADMAP Phase 13 owns full wiki journey; project E2E rule still applies if planner judges UI shippable without Phase 13 — prefer Vitest here + Phase 13 E2E unless planner finds a cheap smoke add).

### Deferred Ideas
- Obsidian zip export button / IPC UI — Phase 12
- Full `wiki.spec.ts` enable → compile → list → export — Phase 13
- Wire `auto_on_insights` to compile on index — Future / post-MVP (Phase 07 D-09)
- Wiki chip / icon beside「笔记页」— optional polish, not required for WIKI-06
- Bulk「全部生成笔记」— out of phase scope
- Success toast with `WikiCompileSummary` counts — explicitly rejected for this phase

---

## Standard Stack

| Layer | Use | Confidence |
|-------|-----|------------|
| React 19 + TypeScript | Views + hooks | HIGH `[VERIFIED: codebase]` |
| Vite 7 + Vitest 3 | Unit tests (`jsdom`) | HIGH `[VERIFIED: codebase]` |
| `@tauri-apps/api` `invoke` | Via `src/lib/tauri.ts` wrappers | HIGH `[VERIFIED: codebase]` |
| Tailwind v4 + Motion | Existing Settings/Library styling | HIGH `[VERIFIED: codebase]` |
| Existing Tauri cmd | `compile_wiki_cmd(source_id)` already registered | HIGH `[VERIFIED: codebase]` |

**Do not add** new npm/Rust deps. No new Tauri commands. No schema migrations.

---

## Architecture Patterns

### 1. Thin view + App busy wrapper (summarize/extract pattern) — **USE THIS**

`[VERIFIED: codebase]`

- `useLibrary` exposes domain helpers (`summarizeSource`, `extractTasks`) that call `src/lib/tauri.ts`, then `refreshSources`, and `reportError` on failure. Hook does **not** own `busy`.
- `App.tsx` wraps each call: `setErr(null); setBusy(true); try { await lib… } finally { setBusy(false) }`.
- `LibraryView` receives `busy`, `config`, and `onSummarizeSource` / `onExtractTasks` props; renders gated buttons.

**Recommendation (Discretion):** Mirror exactly — add `compileWiki` in `useLibrary` + `compileWiki` wrapper in `tauri.ts` + `handleCompileWiki` in `App` with busy; pass `onCompileWiki` into `LibraryView`. Do not put `setBusy` inside the hook.

### 2. Settings: local `setConfig` + bottom「保存配置」

`[VERIFIED: codebase]`

- `SettingsView` uses `useJarvisConfig()` → `config` / `setConfig` / `handleSaveConfig`.
- Checkboxes under「自动洞察」mutate via `setConfig({ ...config, field: checked })` only; persist on primary button click (`handleSaveConfig(config)`).
- Accordion sections use local `AccordionSection` with `testId` + `title` (e.g. `settings-section-lark`).

**Recommendation:** New sibling `AccordionSection testId="settings-section-wiki" title="Wiki 笔记"` placed near Lark/Agent (after sync section or before Lark — either is fine; prefer **after「同步与监听」/ before Lark** for discoverability). Checkbox only mutates `wiki.enabled` via nested spread:

```ts
setConfig({
  ...config,
  wiki: { enabled: e.target.checked, auto_on_insights: config.wiki?.auto_on_insights ?? false },
});
```

Preserve `auto_on_insights` when toggling enabled so save does not wipe the dormant field.

### 3. Feature-flag visibility in Library

`[VERIFIED: codebase]` — `LibraryView` already receives `config: AppConfig | null` but currently only uses `cursor_projects_root`. Gate compile button with:

`config?.wiki?.enabled === true && s.status === "indexed" && s.kind !== "wiki_page"`

Treat missing `wiki` as disabled (matches default-off / optional nested field).

### 4. IPC already done (Phase 10)

`[VERIFIED: codebase]` — `src-tauri/src/commands/wiki.rs`:

- Command name: `compile_wiki_cmd`
- Arg: `source_id: String` → FE serde camelCase `{ sourceId }`
- Returns `WikiCompileSummary` (ignore in UX per D-11)
- Backend hard-gates `cfg.wiki.enabled` and WikiPage inputs — UI gates are UX-only, not a substitute for backend checks

FE wrapper:

```ts
export function compileWiki(sourceId: string) {
  return invoke<WikiCompileSummary>("compile_wiki_cmd", { sourceId });
}
```

Add a minimal `WikiCompileSummary` type in `src/types/` (or inline in tauri.ts) matching Rust fields — only needed for typing; UI discards value.

---

## Don't Hand-Roll

| Problem | Don't | Do instead |
|---------|-------|------------|
| Persist wiki toggle | Immediate save / new IPC | Existing「保存配置」+ `handleSaveConfig` |
| Compile progress UI | New toast / per-row spinner | Global `busy` + silent `refreshSources` |
| Config shape | Second wiki type / flatten keys | Existing `WikiConfig` in `config.ts` / `ipc.ts` |
| Compile pipeline | Re-implement analyze/write | `invoke("compile_wiki_cmd")` only |
| Export button | Ship with Phase 11 | Phase 12 |
| Full E2E wiki journey | New `wiki.spec.ts` this phase | Vitest here; Phase 13 owns journey |
| Hide wiki_page rows when disabled | Filter out kind | Keep list; only hide compile controls (D-04) |

---

## Common Pitfalls

1. **Nesting wiki under「自动洞察」** — Violates D-01; copy AccordionSection sibling pattern instead. `[ASSUMED risk / CONTEXT locked]`
2. **Showing `auto_on_insights` checkbox** — Implies auto-compile (D-02 forbid). `[VERIFIED: CONTEXT]`
3. **Forgetting `wiki_page` exclusion** — Button on notes would call backend that hard-rejects; bad UX (D-06). `[VERIFIED: Phase 10 + CONTEXT]`
4. **Gating on `config.wiki.enabled` without optional chaining** — `config`/`wiki` may be null/undefined → crash. Use `config?.wiki?.enabled === true`. `[VERIFIED: types optional]`
5. **Wiping `auto_on_insights` on toggle** — Must preserve when spreading `wiki`. `[VERIFIED: WikiConfig shape]`
6. **Putting busy inside `useLibrary`** — Hook tests assert busy is NOT exposed; App owns busy. `[VERIFIED: useLibrary.test.ts]`
7. **Shipping export / toast / bulk** — Deferred; keep phase shippable. `[VERIFIED: CONTEXT deferred]`
8. **Skipping Vitest** — WIKI-06 visibility rules are pure FE; unit tests are the right gate this phase. Project E2E rule: full journey Phase 13; prefer Vitest-only unless a one-line settings.spec smoke is free. `[VERIFIED: ROADMAP + CONTEXT discretion]`
9. **Assuming Task 5 plan literally** — Draft plan also mentioned export UI and `auto_on_insights`; CONTEXT overrides those. `[CITED: docs/superpowers/plans/2026-07-16-wiki-compile-layer.md Task 5 + CONTEXT]`

---

## Code Examples

### Library per-source actions (extend this block)

`[VERIFIED: codebase]` `src/views/LibraryView.tsx` ~203–223:

```tsx
{s.status === "indexed" && (
  <>
    <button … data-testid={`summarize-${s.id}`} …>生成摘要</button>
    <button … data-testid={`extract-tasks-${s.id}`} …>提取任务</button>
    {/* ADD: when config?.wiki?.enabled && s.kind !== "wiki_page" → 生成笔记 */}
  </>
)}
```

### App busy wrapper (copy for compile)

`[VERIFIED: codebase]` `src/App.tsx` ~232–239:

```ts
async function handleSummarizeSource(sourceId: string) {
  setErr(null);
  setBusy(true);
  try {
    await libSummarizeSource(sourceId);
  } finally {
    setBusy(false);
  }
}
```

### Settings checkbox (local state only)

`[VERIFIED: codebase]` `src/views/SettingsView.tsx` ~595–607 pattern:

```tsx
<label className="flex items-center gap-2 text-zinc-300">
  <input
    type="checkbox"
    checked={config.auto_summarize_on_index}
    onChange={(e) =>
      setConfig({ ...config, auto_summarize_on_index: e.target.checked })
    }
  />
  索引完成后自动生成摘要
</label>
```

### Tauri wrapper style

`[VERIFIED: codebase]` `src/lib/tauri.ts` ~100–102:

```ts
export function summarizeSource(sourceId: string) {
  return invoke<string>("summarize_source_cmd", { sourceId });
}
```

### Config round-trip already green

`[VERIFIED: codebase]` `src/types/config.test.ts` — `preserves nested wiki flags through flat↔nested round-trip`. No type work required unless Settings typed access needs defaults.

### Backend command (consume only)

`[VERIFIED: codebase]` `src-tauri/src/commands/wiki.rs` — `compile_wiki_cmd(source_id)` registered in `lib.rs`.

---

## Validation Architecture

### Requirement under test

| ID | Behavior |
|----|----------|
| WIKI-06 | Settings toggle with stable testid; Library「生成笔记」when enabled+eligible; controls absent when disabled |

### Recommended automated tests (Vitest)

| Case | Where | Assert |
|------|-------|--------|
| Toggle present | SettingsView render (or thin extract) | `settings-section-wiki`, `wiki-enabled-toggle`, label「启用 Wiki 笔记层」 |
| Toggle mutates local config only | SettingsView | Changing checkbox updates `wiki.enabled` in state; no save invoke until「保存配置」 |
| Button visible when enabled | LibraryView props test | `wiki.enabled: true` + indexed `local_file` → `wiki-compile-{id}` text「生成笔记」 |
| Button hidden when disabled | LibraryView | `wiki.enabled: false` or missing wiki → no `wiki-compile-*` |
| Button hidden for wiki_page | LibraryView | indexed `wiki_page` + enabled → no compile button |
| Hook compile path | useLibrary.test | `compileWiki` calls tauri mock + `refreshSources`; errors via `onError` |

### E2E

- **Not required for phase closeout** if Vitest covers visibility (ROADMAP Phase 13 owns full journey; CONTEXT discretion prefers Vitest here).
- Default E2E config keeps `wiki.enabled=false` → compile controls absent satisfies success criterion 3 without new specs.
- Optional cheap smoke: assert `settings-section-wiki` absent-or-present — only if planner finds existing settings.spec easily extended; do not block on full compile E2E.

### Commands

| Gate | Command |
|------|---------|
| Quick | `npx vitest run src/views/LibraryView.test.tsx src/hooks/useLibrary.test.ts` (paths as created) |
| Full FE | `npm test` / `npx vitest run` |
| Rust (smoke, optional) | none required — no Rust changes expected |

### Nyquist notes for planner

- Every PLAN task that touches UI must include automated Vitest verify.
- Wave 0: create `LibraryView.test.tsx` if missing (currently no view test for Library; `useLibrary.test.ts` and `ChatView.test.tsx` exist as analogs).
- No watch-mode flags in verify commands.

---

## Project Constraints (from .cursor/rules/)

| Rule | Implication for Phase 11 |
|------|--------------------------|
| `tdd-goal-driven.mdc` | Failing Vitest first for visibility/gates; then implement |
| `e2e-required.mdc` | User-facing → E2E normally mandatory; CONTEXT + ROADMAP defer full journey to Phase 13 — document this deferral in PLAN `must_haves` / deferred E2E; Vitest is the phase gate |
| `frontend-taste.mdc` | Match existing AccordionSection / btn-ghost / Chinese copy; no new visual system |
| `karpathy-guidelines.mdc` | Surgical diffs; no export/toast/bulk; no speculative abstractions |
| `jarvis-stack.mdc` | No stack changes; React/Vite/Tailwind as-is |

---

## Recommended Plan Shape (for planner)

| Plan | Wave | Focus |
|------|------|-------|
| 11-01 | 1 | Settings「Wiki 笔记」section + toggle + Vitest |
| 11-02 | 1 or 2 | `tauri.compileWiki` + `useLibrary` + App busy handler + Library button gates + Vitest |

Parallelize 01/02 only if Settings and Library files won't conflict; safer: Wave 1 Settings, Wave 2 Library+IPC wrapper (Library depends on knowing enabled flag exists in config UI but not on Settings file).

**Files expected to modify:**
- `src/views/SettingsView.tsx`
- `src/views/LibraryView.tsx` (+ new `LibraryView.test.tsx`)
- `src/hooks/useLibrary.ts` + `useLibrary.test.ts`
- `src/lib/tauri.ts`
- `src/App.tsx`
- Possibly `src/types/ipc.ts` for `WikiCompileSummary` type

**Files NOT to modify:**
- `src-tauri/src/commands/wiki.rs` (already complete)
- Export UI, wiki.spec.ts, insights auto-compile

---

## Open Questions (resolved by Discretion)

| Question | Recommendation |
|----------|----------------|
| Hook vs App for compile? | Both: hook for invoke+refresh+error; App for busy (match summarize) |
| LibraryView.test vs hook-only? | Prefer **LibraryView.test.tsx** for visibility rules (D-05..D-08); extend useLibrary.test for invoke |
| Minimal E2E? | **Skip** this phase; Phase 13 |
| Accordion placement? | After sync section, before Lark |

---

## RESEARCH COMPLETE

Path: `.planning/phases/11-library-settings-ui/11-RESEARCH.md`

Key findings:
1. Backend `compile_wiki_cmd` is ready — FE-only phase.
2. Mirror summarize/extract: tauri wrapper → useLibrary → App busy → LibraryView button.
3. Settings: new AccordionSection sibling; local setConfig; save via「保存配置」; preserve `auto_on_insights`.
4. Gate: `config?.wiki?.enabled && indexed && kind !== "wiki_page"`.
5. Validate with Vitest; defer full E2E to Phase 13.
