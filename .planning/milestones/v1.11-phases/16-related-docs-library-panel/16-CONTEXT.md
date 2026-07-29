# Phase 16: Related-docs Library panel - Context

**Gathered:** 2026-07-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Users selecting a Library source see a **related-docs panel** backed by Phase 15 `related_sources`: loading / empty / error states, rows with title + kind label + snippet (no raw scores), and click-to-navigate that selects the neighbor in Library. Includes Tauri IPC + focused E2E under `JARVIS_E2E=1`. No graph UI, no MCP, no citation-trust work (Phases 17–19).

</domain>

<decisions>
## Implementation Decisions

### Panel placement
- **D-01:** Layout composition (side column vs under-row strip vs other) is **Claude's discretion** — prefer the cleanest fit with today's single-column Library glass panel; minimize layout churn.
- **D-02:** When **no source is selected**, the related panel is **fully hidden** (no placeholder chrome).
- **D-03:** Long related lists scroll **inside the panel** independently of the main source list.
- **D-04:** Panel appearance uses **light motion** (fade / short translate), honoring `prefers-reduced-motion` / existing Library motion patterns.

### Selection model
- **D-05:** Selecting a source = click on the **row blank area** (not action buttons). Per-row buttons (summarize / tasks / retry / etc.) keep their own handlers and do not toggle selection.
- **D-06:** Non-`indexed` sources (pending / failed / etc.) **remain selectable**; the related panel shows an honest non-indexed empty/error state instead of silent ignore.
- **D-07:** Selected row uses **highlight border / background** consistent with existing cyan accent.
- **D-08:** Clicking the **same selected row again** clears selection and hides the related panel (toggle).

### Row content & copy (Chinese UI)
- **D-09:** Each related row shows **title + kind label + snippet** — no similarity / RRF scores (aligns REL-04 + Phase 15 D-09).
- **D-10:** Empty (indexed seed, zero neighbors): **「暂无相关文档」**.
- **D-11:** Selected but not indexed: **「仅已索引来源可查看相关文档」**.
- **D-12:** Loading: **「加载相关文档…」**; error: **「相关文档加载失败」** (optional short error detail OK).

### Navigate-on-click
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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone / requirements
- `.planning/ROADMAP.md` — Phase 16 goal, success criteria, REL-01 / REL-03 / REL-04
- `.planning/REQUIREMENTS.md` — REL-01, REL-03, REL-04; REL-F01 deferred soft Wiki demote
- `.planning/PROJECT.md` — v1.11 Active Requirements (related-docs panel)

### Prior phase (backend contract)
- `.planning/phases/15-overlap-scoring-api/15-CONTEXT.md` — D-01..D-11 seed/DTO/top_n/no scores
- `.planning/phases/15-overlap-scoring-api/15-02-SUMMARY.md` — `related_sources` GREEN; serde deferred to Phase 16 IPC
- `crates/retriever/src/related.rs` — `related_sources`, `RelatedSource` DTO

### Frontend / E2E anchors
- `src/views/LibraryView.tsx` — Library list UI (no selection today)
- `src/hooks/useLibrary.ts` — Library data/actions hook
- `src/lib/sourceDisplay.ts` — `sourceKindLabel` / status helpers
- `src/lib/tauri.ts` — IPC invoke wrappers
- `.cursor/rules/e2e-required.mdc` — user-facing features need E2E
- `.cursor/rules/frontend-taste.mdc` — glass panel / motion / loading-empty-error
- `e2e/specs/` — existing Library / navigation specs for patterns

### Research (optional)
- `.planning/research/FEATURES.md` — related-docs feature shape
- `.planning/research/ARCHITECTURE.md` — thin UI over retrieve

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `retriever::related_sources` — backend; needs thin Tauri command + serde on `RelatedSource`
- `sourceKindLabel` — kind display for related rows
- Library glass-panel + motion (`useReducedMotion`) — match panel appear animation
- Existing `data-testid` patterns (`library-stats`, `source-list`, per-source action ids)

### Established Patterns
- Thin React → `invoke` → Rust crates; no business logic in `App.tsx` beyond wiring
- Loading / empty / error states required on interactive surfaces
- E2E via WebdriverIO + `JARVIS_E2E=1` mocks; Vitest for pure UI/helpers

### Integration Points
- Add selection state to Library (view and/or `useLibrary`)
- New IPC: list related for `source_id` (+ optional `top_n`)
- Scroll-into-view on source list row by id when navigating from related panel
- E2E: select indexed fixture source → assert panel testids → click related → assert new selection

</code_context>

<specifics>
## Specific Ideas

- Chinese copy locked as product strings above (D-10..D-12) — keep UI language consistent with existing Library (「资料库」 etc.)
- Related-chain browsing (D-13) is intentional: each click re-seeds the panel
- No placeholder when nothing selected — keep default Library clean

</specifics>

<deferred>
## Deferred Ideas

- Soft-exclude / demote WikiPage neighbors — **REL-F01** (roadmap future)
- MCP tools — **Phases 17–18**
- Citation / trust regression gate + full journey hardening — **Phase 19** (this phase still does focused related-docs E2E per D-16)
- Graph UI / co-citation visualization — out of milestone
- Affinity score threshold / exposing numeric overlap scores — Phase 15 D-06/D-09; do not add here

None — discussion stayed within phase scope for new capabilities.

</deferred>

---

*Phase: 16-Related-docs Library panel*
*Context gathered: 2026-07-26*
