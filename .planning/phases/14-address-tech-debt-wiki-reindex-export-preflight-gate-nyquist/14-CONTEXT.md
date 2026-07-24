# Phase 14: Tech debt closeout - Context

**Gathered:** 2026-07-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Clear v1.10 milestone-audit tech debt so closeout can reach audit `passed` without accepting the four Phase-14 items as debt: (1) WikiPage reindex stub that marks Failed, (2) export preflight ungated on `wiki.enabled`, (3) Nyquist `false` on phases 12/13 VALIDATION.md, (4) ROADMAP WIKI-08/09 coverage sync (already drafted). No new wiki product features. Leave WIKI-F01 (`auto_on_insights`) and dual `index.md` writers as post-v1.10 backlog.

</domain>

<decisions>
## Implementation Decisions

### WikiPage「重新同步」
- **D-01:** `SourceKind::WikiPage` reindex path is a **soft skip** — do **not** call `mark_failed`; return success-skip semantics consistent with “nothing to do”.
- **D-02:** Soft skip is **silent** — no toast, no error bar.
- **D-03:** In `LibraryView`, **hide** the「重新同步」/「重试」button for `wiki_page` rows (`retry-source-{id}`); backend soft-skip remains as IPC safety net.
- **D-04:** Tests: **Rust unit** (soft skip does not set Failed) + **Vitest** (button hidden for wiki_page). **No new E2E** for this behavior.
- **D-05:** **Bulk** rebuild/retry that encounters `WikiPage` uses the **same soft skip** — no Failed status, do not abort the batch.

### Export preflight when wiki disabled
- **D-06:** When `wiki.enabled` is false, `wiki_export_preflight_cmd` returns a **hard error** with **`InsightsError::WikiDisabled`** (parity with `export_wiki_zip`).
- **D-07:** Prefer **crate reuse** — gate via enabled flag and map the same `WikiDisabled` error string (`wiki is disabled`) as export.
- **D-08:** FE keeps existing `catch` → `reportError(error)`; **no** special Chinese mapping for this string.
- **D-09:** Tests: **Rust unit only** for disabled → WikiDisabled; **no new Vitest** required.

### Nyquist phases 12 / 13
- **D-10:** Set `nyquist_compliant: true` on `12-VALIDATION.md` and `13-VALIDATION.md` based on existing VERIFICATION + green tests; **do not** require a full `/gsd-validate-phase` re-run.
- **D-11:** **Only** flip `nyquist_compliant`; leave `status` and `wave_0_complete` unchanged (Phase 11 pattern).
- **D-12:** Add a **short body note** in each VALIDATION.md citing VERIFICATION + green tests as evidence for the nyquist backfill.

### Audit / milestone closeout bar
- **D-13:** After Phase 14 is verified, **must** re-run `/gsd-audit-milestone` expecting **`status: passed`**, then `/gsd-complete-milestone v1.10`.
- **D-14:** Audit may still list **WIKI-F01** and **dual `index.md` writers** as Deferred / Out of Scope backlog; clearing the four Phase-14 items is enough for `passed`.
- **D-15:** Phase 14 verify gate = relevant **cargo test** + **Vitest** green + VALIDATION/ROADMAP docs updated; **no new E2E**.

### Claude's Discretion
- Exact placement of soft-skip helper vs inline match arm in `index_ops.rs`.
- Whether preflight gate lives only in Tauri cmd (reading `cfg.wiki.enabled`) or a thin crate helper used by both preflight and export — as long as D-06/D-07 hold.
- Exact wording of the VALIDATION.md evidence note (D-12).
- How ROADMAP Phase 14 plan checkboxes are filled after planning (not locked here).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Audit & roadmap
- `.planning/v1.10-MILESTONE-AUDIT.md` — tech_debt items this phase closes
- `.planning/ROADMAP.md` — Phase 14 goal + success criteria
- `.planning/REQUIREMENTS.md` — WIKI-01..09 complete; Future WIKI-F01 deferred
- `.planning/STATE.md` — current milestone position
- `.planning/PROJECT.md` — Core Value / wiki beside RAG

### Prior phase decisions
- `.planning/phases/10-persist-index/10-CONTEXT.md` — wiki persist/index, dual writer note
- `.planning/phases/12-obsidian-zip-export/12-CONTEXT.md` — export gates, preflight empty path
- `.planning/phases/13-e2e-citation-trust/13-CONTEXT.md` — E2E already green; no new E2E in Phase 14

### Implementation touchpoints
- `src-tauri/src/index_ops.rs` — WikiPage reindex arm (~653)
- `src-tauri/src/commands/wiki.rs` — `wiki_export_preflight_cmd` / `export_wiki_zip_cmd`
- `crates/insights/src/error.rs` — `InsightsError::WikiDisabled`
- `crates/insights/src/wiki_export.rs` — export enabled gate pattern
- `src/views/LibraryView.tsx` — `retry-source-{id}` button; wiki_page compile hide pattern
- `src/views/LibraryView.test.tsx` — Vitest patterns for wiki_page visibility
- `.planning/phases/12-obsidian-zip-export/12-VALIDATION.md` — nyquist frontmatter
- `.planning/phases/13-e2e-citation-trust/13-VALIDATION.md` — nyquist frontmatter
- `.planning/phases/12-obsidian-zip-export/12-VERIFICATION.md` — evidence for D-10/D-12
- `.planning/phases/13-e2e-citation-trust/13-VERIFICATION.md` — evidence for D-10/D-12

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `InsightsError::WikiDisabled` + `export_wiki_zip(..., enabled)` gate pattern
- LibraryView already hides「生成笔记」for `wiki_page` — mirror for retry button
- Existing Vitest matrix in `LibraryView.test.tsx` for wiki_page / enabled gates
- `mark_failed` helper in `index_ops.rs` — stop calling it for WikiPage

### Established Patterns
- Hide (not disable) wiki controls when inappropriate
- Thin Tauri commands; domain errors from `insights` crate
- Soft skip / Ok(false) used elsewhere in index_ops for “nothing indexed”

### Integration Points
- Single-source retry and bulk rebuild share WikiPage match arm
- Preflight IPC called from `useLibrary.exportWiki` before Save dialog
- Milestone close: Phase 14 VERIFICATION → `/gsd-audit-milestone` → `/gsd-complete-milestone`

</code_context>

<specifics>
## Specific Ideas

- Soft skip + hide button + silent (no toast)
- Preflight hard-fail with same `wiki is disabled` string as export
- Nyquist: frontmatter-only flip + short evidence note; no validate-phase ceremony
- Closeout: mandatory re-audit to `passed`; F01 / dual writers stay backlog

</specifics>

<deferred>
## Deferred Ideas

- **WIKI-F01** `auto_on_insights` UX — post-v1.10 backlog
- **Dual `index.md` writers** consolidation (Phase 09 test helper vs compile `rebuild_index_md_from_disk`) — intentional; leave deferred
- True disk re-index of wiki Markdown / re-compile from parent source — out of Phase 14 scope
- Chinese mapping for `wiki is disabled` in FE — rejected for this phase (D-08)
- New E2E for retry-button absence — rejected (D-04, D-15)

</deferred>

---

*Phase: 14-tech-debt-closeout*
*Context gathered: 2026-07-24*
