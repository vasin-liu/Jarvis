# Phase 11: Library / Settings UI - Context

**Gathered:** 2026-07-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver feature-flagged UI for WIKI-06: Settings exposes a wiki `enabled` toggle with stable `data-testid`; when enabled, Library shows「生成笔记」for eligible indexed sources and invokes existing `compile_wiki_cmd`; when disabled, compile/export-related controls are absent. No new compile pipeline — wire FE to Phase 10 IPC. Obsidian zip export UI/IPC is Phase 12; full E2E journey is Phase 13 (Vitest gating for this phase is in scope).

**Requirements:** WIKI-06  
**Success criteria (ROADMAP):** Settings toggle + testid; Library「生成笔记」when enabled; controls hidden when disabled (default E2E config).

**Out of scope this phase:** Obsidian zip / export button (Phase 12); `wiki.spec.ts` full journey (Phase 13); wiring `auto_on_insights`; bulk「全部生成笔记」; wiki chip/icon polish beyond existing「笔记页」label.

</domain>

<decisions>
## Implementation Decisions

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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone / phase
- `.planning/ROADMAP.md` — Phase 11 goal, success criteria, WIKI-06
- `.planning/REQUIREMENTS.md` — WIKI-06
- `.planning/PROJECT.md` — v1.10 wiki goals; default-off; local-first
- `.planning/STATE.md` — current phase position

### Research / plan draft
- `.planning/research/SUMMARY.md` — Library/Settings feature-flag surface
- `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` — Task 5 (IPC + Library UI + Settings toggle). Apply discussion overrides: no export UI here; no `auto_on_insights` checkbox; save via「保存配置」; silent success.

### Prior phases
- `.planning/phases/10-persist-index/10-CONTEXT.md` — `compile_wiki_cmd` / summary / hard gates (enabled + WikiPage reject)
- `.planning/phases/07-wiki-kind-config/07-CONTEXT.md` — `WikiConfig`,「笔记页」label; Settings toggle deferred to 11; `auto_on_insights` dormant
- `.planning/phases/09-llm-wiki-analysis/09-CONTEXT.md` — fail-closed analyze (errors may surface in UI)
- `.planning/phases/08-deterministic-markdown-renderer/08-CONTEXT.md` — vault shape (no UI work)

### Code integration points
- `src/views/SettingsView.tsx` — AccordionSection + checkbox patterns;「保存配置」
- `src/views/LibraryView.tsx` — per-source actions (`summarize-{id}`, `extract-tasks-{id}`)
- `src/hooks/useLibrary.ts` — invoke + `busy` + `reportError` + `refreshSources`
- `src/types/config.ts` / `src/types/ipc.ts` — nested `wiki` already present
- `src-tauri/src/commands/wiki.rs` — `compile_wiki_cmd(source_id)` already registered
- `crates/config` — `WikiConfig.enabled` persistence

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `compile_wiki_cmd` — ready; FE only needs `invoke("compile_wiki_cmd", { sourceId })`
- `WikiConfig` / FE `wiki?: { enabled, auto_on_insights }` — already in types + Vitest round-trip
- `sourceKindLabel("wiki_page")` →「笔记页」
- Settings AccordionSection + checkbox labels under「自动洞察」as visual precedent (but wiki gets its own section)
- Library indexed-only button group for summarize/extract

### Established Patterns
- Thin views: handlers passed as props from App / hooks
- Global `busy` disables Library buttons during IPC
- Config: local `setConfig` then bottom「保存配置」→ `handleSaveConfig`
- Chinese product copy; kebab-case `data-testid`

### Integration Points
- Gate UI on `config.wiki.enabled` from loaded AppConfig (E2E default remains false → controls absent)
- After compile success, `refreshSources` so new WikiPage rows appear
- Phase 12 will add export control (likely same「Wiki 笔记」Settings section or Library toolbar — not this phase)

</code_context>

<specifics>
## Specific Ideas

- Match「生成摘要」UX exactly for compile (busy / error / silent refresh / re-run).
- Keep「笔记页」list visibility when wiki disabled so users do not lose sight of already-compiled notes.
- Prefer draft testids from Task 5 plan; accordion id follows `settings-section-lark` pattern.

</specifics>

<deferred>
## Deferred Ideas

- Obsidian zip export button / IPC UI — Phase 12
- Full `wiki.spec.ts` enable → compile → list → export — Phase 13
- Wire `auto_on_insights` to compile on index — Future / post-MVP (Phase 07 D-09)
- Wiki chip / icon beside「笔记页」— optional polish, not required for WIKI-06
- Bulk「全部生成笔记」— out of phase scope
- Success toast with `WikiCompileSummary` counts — explicitly rejected for this phase

</deferred>

---

*Phase: 11-Library / Settings UI*
*Context gathered: 2026-07-22*
