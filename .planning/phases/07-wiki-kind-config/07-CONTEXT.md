# Phase 07: Wiki kind + config - Context

**Gathered:** 2026-07-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver the foundation for the Wiki Compile Layer: `SourceKind::WikiPage` (`wiki_page`), nested `WikiConfig` on `AppConfig` (defaults both `false`), and Library label「笔记页」via `sourceKindLabel`. No compile pipeline, no Settings toggle UI, no zip export in this phase.

**Requirements:** WIKI-01, WIKI-02  
**Success criteria (ROADMAP):** pre-v1.10 config loads with wiki off; kind round-trip; Vitest label「笔记页」.

</domain>

<decisions>
## Implementation Decisions

### Library label
- **D-01:** `sourceKindLabel("wiki_page")` returns exactly **「笔记页」** (Chinese, consistent with「记忆」「本地文件」).
- **D-02:** No bilingual UI label; no Wiki chip/suffix in Phase 07.
- **D-03:** Vitest asserts exact equality to `"笔记页"`.

### config.json shape
- **D-04:** Nested object — `"wiki": { "enabled": bool, "auto_on_insights": bool }` on `AppConfig` as `pub wiki: WikiConfig` with `#[serde(default)]`. **Do not** `#[serde(flatten)]` WikiConfig (would collide with ambiguous top-level field names).
- **D-05:** Missing `wiki` key → `WikiConfig::default()` (`enabled: false`, `auto_on_insights: false`).
- **D-06:** Unknown fields inside `wiki` are ignored (serde default forward-compat).
- **D-07:** Config tests cover (a) JSON without `wiki` and (b) explicit `wiki` object round-trip.

### auto_on_insights field timing
- **D-08:** Include `auto_on_insights: bool` on `WikiConfig` in Phase 07 (default `false`).
- **D-09:** Do **not** wire `auto_on_insights` to insights/compile in this milestone (Future / later phase).
- **D-10:** Phase 07 does not expose Settings UI for wiki (Settings toggle is Phase 11; auto switch not required in 07).
- **D-11:** Tests: default `auto_on_insights == false`; explicit JSON `true` round-trips.

### Unknown kind / old UI
- **D-12:** Pre-update frontend showing raw `wiki_page` string is acceptable (existing `default: return kind`).
- **D-13:** Keep `SourceKind::parse` → `None` for unknown strings; callers degrade as today. No `Unknown` variant.
- **D-14:** Adding `WikiPage` must fix all exhaustive matches so `cargo test -p store -p config` (and any crate that match on `SourceKind`) compile and pass.
- **D-15:** Keep FE `default` branch as `return kind`; only add the `wiki_page` case.

### Claude's Discretion
- Exact placement of unit tests (inline `#[cfg(test)]` vs `crates/*/tests/`) — follow local crate conventions.
- Whether `WikiConfig` lives in `types.rs` vs a small submodule — prefer existing `types.rs` pattern unless file is already oversized.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone / phase
- `.planning/ROADMAP.md` — Phase 07 goal, success criteria, WIKI-01/02
- `.planning/REQUIREMENTS.md` — WIKI-01, WIKI-02
- `.planning/PROJECT.md` — v1.10 milestone goals; wiki default-off constraint
- `.planning/research/SUMMARY.md` — stack/pitfalls for wiki (Phase 07 = kind + config)

### Implementation plan
- `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` — Task 1 interfaces (`WikiConfig`, `WikiPage` ↔ `wiki_page`)

### Code integration points
- `crates/store/src/types.rs` — `SourceKind` enum, `as_str` / `parse`
- `crates/config/src/types.rs` — `AppConfig` nested structs + serde defaults
- `src/lib/sourceDisplay.ts` + `src/lib/sourceDisplay.test.ts` — kind labels

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `SourceKind::Memory` / `"memory"` — precedent for new kind + FE label「记忆」
- `AppConfig` nested configs with `#[serde(default)]` — pattern for `wiki: WikiConfig` **without** flatten
- `sourceKindLabel` switch — add one case; leave `default` alone

### Established Patterns
- Kind strings are snake_case (`local_file`, `wiki_page`)
- UI labels are Chinese product copy
- Exhaustive Rust matches on `SourceKind` must be updated when adding variants

### Integration Points
- Store persistence of `kind` column strings
- Config load/save for `config.json`
- Library list rendering via `sourceKindLabel(kind)`

</code_context>

<specifics>
## Specific Ideas

- Prefer nested `"wiki": {…}` over flattened `wiki_enabled` for clarity and future fields.
- Ship dormant `auto_on_insights` now so later phases do not reshape config again.

</specifics>

<deferred>
## Deferred Ideas

- Settings toggle for `wiki.enabled` — Phase 11
- Wiring `auto_on_insights` into insights_ops — Future / post-MVP
- Wiki chip / icon beside label — Phase 11+ UI polish if desired
- Compile / renderer / zip / E2E — Phases 08–13

</deferred>

---

*Phase: 07-Wiki kind + config*
*Context gathered: 2026-07-17*
