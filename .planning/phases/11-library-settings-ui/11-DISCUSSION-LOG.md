# Phase 11: Library / Settings UI - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-22
**Phase:** 11-Library / Settings UI
**Areas discussed:** Settings placement & fields, Library compile eligibility, Compile feedback, Copy & testids

---

## Settings placement & fields

| Option | Description | Selected |
|--------|-------------|----------|
| Independent accordion「Wiki 笔记」 | Sibling to Lark/Agent | ✓ |
| Under「自动洞察」 | Minimal change | |
| You decide | | |

### Follow-ups

| Option | Description | Selected |
|--------|-------------|----------|
| Only expose `enabled` | Hide `auto_on_insights` | ✓ |
| Show auto disabled + “即将推出” | | |
| Show both saveable but unwired | | |
| Save via「保存配置」 | Match most Settings fields | ✓ |
| Immediate save on toggle | Like pipeline/router | |
| Keep wiki_page in Library when disabled | Hide compile only | ✓ |
| Filter out wiki_page when disabled | | |

**Notes:** User chose clarity over nesting; avoid misleading auto-compile UI.

---

## Library「生成笔记」eligibility

| Option | Description | Selected |
|--------|-------------|----------|
| Indexed only | Same as summarize/extract | ✓ |
| Indexed or pending | | |
| Hide for wiki_page | Align with backend reject | ✓ |
| Show wiki_page + error | | |
| All non–wiki_page kinds | | ✓ |
| local_file only | | |
| local_file + Lark only | | |
| Same button group as summarize/extract | | ✓ |
| Separate visual group | | |

**Notes:** User reply `11` on kinds question interpreted as option 1 (all non–wiki_page).

---

## Compile feedback

| Option | Description | Selected |
|--------|-------------|----------|
| Global busy | Match summarize | ✓ |
| Per-source only disable | | |
| Busy +「生成中…」label | | |
| Existing error bar | | ✓ |
| Inline button error | | |
| Silent success + refreshSources | | ✓ |
| Success toast with counts | | |
| Always allow re-compile | | ✓ |
| Hide after first compile | | |

**Notes:** `WikiCompileSummary` unused in UX this phase.

---

## Copy & testids

| Option | Description | Selected |
|--------|-------------|----------|
| 「启用 Wiki 笔记层」 | Plan draft | ✓ |
| 「启用知识笔记（Wiki）」 | | |
| 「启用笔记编译」 | | |
| 「生成笔记」 | ROADMAP | ✓ |
| 「编译笔记」 / 「生成 Wiki」 | | |
| testids: section-wiki / wiki-enabled-toggle / wiki-compile-{id} | | ✓ |
| Alternate settings-wiki-enabled + shared wiki-compile | | |
| Accordion title「Wiki 笔记」 | | ✓ |
| 「知识笔记」 / 「Wiki」 | | |

**Notes:** All copy/testid choices locked to plan-aligned Chinese + kebab testids.

---

## Claude's Discretion

- Accordion helper text under checkbox
- Hook vs App wiring for compile invoke
- Vitest file placement; Phase 11 vs 13 E2E split judgment for planner

## Deferred Ideas

- Export UI (Phase 12), full wiki E2E (Phase 13), `auto_on_insights` wiring, wiki chip, bulk compile, success toast
