---
phase: 11
slug: library-settings-ui
status: approved
shadcn_initialized: false
preset: none
created: 2026-07-22
reviewed_at: 2026-07-22
---

# Phase 11 — UI Design Contract

> Visual and interaction contract for frontend phases. Generated for gsd-ui-phase, verified by gsd-ui-checker.
>
> **Phase scope:** Feature-flagged Settings toggle + Library「生成笔记」button wired to existing `compile_wiki_cmd`. No new screens, themes, export UI, toasts, or bulk actions.
>
> **Recovery note:** UI researcher subagent historically hangs on this Windows host; contract authored from locked `11-CONTEXT.md` + `11-RESEARCH.md` + existing `SettingsView` / `LibraryView` / `src/index.css` tokens (same pattern as Phase 07 inherit-only UI-SPEC).

---

## Design System

| Property | Value |
|----------|-------|
| Tool | none (no shadcn; `components.json` absent) |
| Preset | not applicable |
| Component library | none — existing React + Tailwind v4 app (`src/`) |
| Icon library | inherit — `@tabler/icons-react` (no new icons this phase) |
| Font | inherit — `"Segoe UI", ui-sans-serif, system-ui, sans-serif` (`src/index.css`) |

**Aesthetic (inherited, not redefined):** Dark tech + liquid glass per `.cursor/rules/frontend-taste.mdc` — zinc/slate base, cool cyan accent, `glass-panel`, `btn-ghost` / `btn-primary`. Phase 11 does not invent a new palette, type ramp, or component set.

**Reuse (mandatory):**
- Settings: existing local `AccordionSection` (`rounded-xl border border-white/10`, title `text-sm font-medium text-zinc-200`)
- Settings checkbox: same label row as「自动洞察」(`flex items-center gap-2 text-sm text-zinc-300`)
- Library actions: same column as「生成摘要」/「提取任务」(`btn-ghost text-xs`, `disabled={busy}`)
- Persist: existing bottom「保存配置」`btn-primary` — no new save control

---

## Spacing Scale

Declared values (must be multiples of 4):

| Token | Value | Usage |
|-------|-------|-------|
| xs | 4px | Inline gaps (`gap-1` / icon padding) — inherit |
| sm | 8px | Accordion inner gaps (`gap-2` / `space-y` compact) — inherit; Library action column `gap-2` |
| md | 16px | Accordion content padding (`p-4`) — inherit |
| lg | 24px | Section padding — inherit |
| xl | 32px | Layout gaps — inherit |
| 2xl | 48px | Major breaks — inherit |
| 3xl | 64px | Page-level — inherit |

Exceptions: **none** — new Wiki accordion and compile button reuse existing AccordionSection / Library action spacing. No new spacing tokens.

---

## Typography

| Role | Size | Weight | Line Height |
|------|------|--------|-------------|
| Body | 14px (`text-sm`) | 400 | ~1.5 |
| Label | 14px (`text-sm`) checkbox / 12px (`text-xs`) Library buttons | 400 | ~1.4 |
| Heading | 14px (`text-sm font-medium`) AccordionSection title | 500 | ~1.2 |
| Display | N/A — unused this phase | — | — |

**Phase 11 typography rules:**
- Accordion title「Wiki 笔记」matches other Settings section titles (`text-sm font-medium text-zinc-200`).
- Checkbox label「启用 Wiki 笔记层」matches「自动洞察」checkbox labels (`text-sm text-zinc-300`).
- Optional helper under checkbox: `text-xs text-zinc-500` (same as Lark section helper paragraphs).
- Library「生成笔记」matches「生成摘要」(`btn-ghost text-xs`).

Declared sizes used: **3** (12 / 14 / inherit heading). Weights: **2** (400 + 500). No new type tokens.

---

## Color

| Role | Value | Usage |
|------|-------|-------|
| Dominant (60%) | `zinc-950` / dark app shell | Background — unchanged |
| Secondary (30%) | `zinc-900/55` glass + `border-white/10` accordion | Settings sections, Library panels — unchanged |
| Accent (10%) | `cyan-500` / `cyan-400` | Reserved for `.btn-primary` (「保存配置」), active nav, progress — **not** for wiki toggle or「生成笔记」 |
| Destructive | existing red/error tones (`text-red-300` source errors) | Compile failures use shared App error bar — no new destructive chrome |

Accent reserved for: `.btn-primary` (保存配置), `.nav-btn-active`, stream cursor, index progress fill — **never** for the wiki checkbox, Accordion chrome, or Library ghost「生成笔记」button.

**Phase 11 color rules:**
- Wiki AccordionSection uses identical border/surface as Lark / Agent sections.
- Checkbox is native input in zinc text row — no accent ring beyond existing focus styles.
-「生成笔记」is `btn-ghost` (zinc text + white/10 border), same as summarize/extract — not primary cyan.

---

## Copywriting Contract

| Element | Copy |
|---------|------|
| Primary CTA (Library) | **生成笔记** (D-14) |
| Settings section title | **Wiki 笔记** (D-15) |
| Settings toggle label | **启用 Wiki 笔记层** (D-13) |
| Empty state heading | N/A — no new empty states |
| Empty state body | N/A — no new empty states |
| Error state | Reuse App shared error bar via existing `reportError` / `setErr` — no new per-button copy; backend error string shown as-is |
| Destructive confirmation | N/A — no destructive actions this phase |
| Persist CTA | Existing **保存配置** (unchanged; not wiki-specific) |

### Locked copy (in scope)

| Key | Exact value | `data-testid` |
|-----|-------------|---------------|
| Settings accordion title | `Wiki 笔记` | `settings-section-wiki` |
| Settings checkbox label | `启用 Wiki 笔记层` | `wiki-enabled-toggle` (on the `<input>`) |
| Library compile button | `生成笔记` | `wiki-compile-{id}` |

### Recommended helper (Discretion — optional, keep minimal)

| Key | Suggested copy |
|-----|----------------|
| Settings helper under toggle | `开启后可在资料库为已索引来源生成笔记页。关闭后仅隐藏生成入口，已有笔记页仍保留。` |

Do not mention `auto_on_insights`, export, or Obsidian in Settings copy.

### Explicitly out of scope (do not invent)

- `auto_on_insights` checkbox or any auto-compile copy (D-02)
- Export /「导出 Wiki」/ Obsidian zip UI (Phase 12)
- Success toast / `WikiCompileSummary` counts (D-11 rejected)
- Bulk「全部生成笔记」
- Wiki chip / icon polish beyond existing「笔记页」kind label
- New empty states for “no wiki pages”
- Bilingual toggle labels

### Interaction contract

| Surface | Phase 11 behavior |
|---------|-------------------|
| Settings →「Wiki 笔记」accordion | Independent sibling section (not nested under「自动洞察」). Prefer placement after「同步与监听」, before「飞书 / lark-cli」. |
| `wiki-enabled-toggle` | Updates local `config.wiki.enabled` only; persists on「保存配置」. Preserve `auto_on_insights` when spreading `wiki`. |
| Library「生成笔记」visibility | Show iff `config?.wiki?.enabled === true` AND `status === "indexed"` AND `kind !== "wiki_page"`. |
| Library when wiki disabled | Hide compile controls only; **keep** `wiki_page` rows labeled「笔记页」(D-04). |
| Busy | Global App `busy` disables Library action buttons during compile (same as summarize). |
| Success | Silent `refreshSources` — new「笔记页」rows are feedback; no toast. |
| Failure | Shared App error path only. |
| Re-compile | Button stays available after success (D-12). |
| Export | Absent (Phase 12). |

---

## Registry Safety

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| shadcn official | none | not applicable — shadcn not initialized |
| third-party | none | not applicable |

---

## Component Inventory (implement against)

| Component | Action | Notes |
|-----------|--------|-------|
| `SettingsView` AccordionSection | **Add** `testId="settings-section-wiki"` title=`Wiki 笔记` | Sibling pattern |
| Checkbox + label | **Add** | Mirror 自动洞察 checkbox row |
| `LibraryView` action column | **Extend** | Insert「生成笔记」with summarize/extract |
| `App` busy wrapper | **Add** `handleCompileWiki` | Mirror `handleSummarizeSource` |
| Toast / modal / new page | **Forbidden** | — |

---

## Out of Scope (UI)

Confirmed from `11-CONTEXT.md` deferred + phase boundary:

- Obsidian zip / export button
- Full E2E wiki journey UI assertions (Phase 13)
- `auto_on_insights` UX
- Bulk compile
- Success toast with summary counts
- Visual redesign of Library or Settings

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: pending
- [ ] Dimension 2 Visuals: pending
- [ ] Dimension 3 Color: pending
- [ ] Dimension 4 Typography: pending
- [ ] Dimension 5 Spacing: pending
- [ ] Dimension 6 Registry Safety: pending

**Approval:** pending

---

*Sources: `11-CONTEXT.md` D-01..D-16; `11-RESEARCH.md`; WIKI-06; `07-UI-SPEC.md` inherit pattern; `src/views/SettingsView.tsx`; `src/views/LibraryView.tsx`; `src/index.css`; `.cursor/rules/frontend-taste.mdc`.*
