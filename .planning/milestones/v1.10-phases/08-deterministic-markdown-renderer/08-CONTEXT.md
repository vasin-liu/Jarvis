# Phase 08: Deterministic Markdown renderer - Context

**Gathered:** 2026-07-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver a **pure** `render_wiki_pages(analysis, source_uri, source_title) -> WikiCompileResult` in `crates/insights`: stable Markdown pages with YAML frontmatter, `[[path|display]]` wikilinks, and an `index_markdown` string. **No LLM, no disk I/O, no Tauri/UI** in this phase.

**Requirements:** WIKI-03 (render half only)  
**Success criteria (ROADMAP):** empty entities/concepts still yield a source-summary with `sources:`; non-empty entities emit `entities/` pages + wikilinks from the summary; CJK/unsafe names produce filesystem-safe slugs (ASCII or `e-{hash6}`) in unit tests.

LLM analyze → `WikiAnalysis` is Phase 09; write `{app_data}/wiki/` + index as `WikiPage` is Phase 10.

</domain>

<decisions>
## Implementation Decisions

### Wikilink style
- **D-01:** Forward links use Obsidian alias form **`[[path|display]]`**, e.g. `[[entities/acme|Acme]]`, `[[concepts/foo|Foo]]`.
- **D-02:** Linking is **bidirectional**: source-summary body links to each entity/concept page; each entity/concept body links back to the source summary.
- **D-03:** Back-link form is **`[[sources/{slug}|{source_title}]]`** (same path|display convention).

### Page tree / slugs
- **D-04:** Output tree layout: **`sources/`**, **`entities/`**, **`concepts/`**, plus root-level **`index.md`** content (as `index_markdown` string in Phase 08).
- **D-05:** Slugify rule: keep `[a-z0-9-]` from unicode lowercase of the name; if empty after filtering, use **`e-{sha256_6(name)}`** (6 hex chars). Title/display name stays in frontmatter and link display text — never require pinyin crates.
- **D-06:** Source-summary slug uses **the same slugify rules** as entity/concept names (applied to `source_title`).
- **D-07:** Same-directory slug collisions: deterministic suffix **`-2`, `-3`, …** by input order within that directory. Cross-directory same name (entity vs concept) is **not** a collision.

### Frontmatter (Phase 08 drafts)
- **D-08:** Every page draft body includes YAML frontmatter with **`title`**, **`type`**, **`sources`**, **`generated: true`**. **Omit `content_hash`** in Phase 08 (filled at write time in Phase 10).
- **D-09:** `type` literals are snake_case: **`source_summary`**, **`entity`**, **`concept`** (aligned with `WikiPageType` serde).
- **D-10:** `sources` is a **one-line JSON-style array**, e.g. `sources: ["file:///a.md"]`.

### index.md shape (`index_markdown`)
- **D-11:** Structure: `# Wiki` heading, then sections **`## Sources`**, **`## Entities`**, **`## Concepts`** with bullet `[[path|display]]` entries.
- **D-12:** **Omit empty sections** (no entities → no `## Entities` block).
- **D-13:** No YAML frontmatter on the index string (catalog only, not a `WikiPageDraft`).

### Claude's Discretion
- Exact body prose layout under frontmatter (headings for blurb/summary text) as long as D-01…D-03 links and D-08…D-10 frontmatter hold.
- Whether slugify lives in `wiki.rs` vs a tiny helper module inside `insights`.
- Test placement: `#[cfg(test)]` in `wiki.rs` vs `crates/insights/tests/` — follow crate conventions.
- SHA-256 truncation helper reuse of workspace `sha2`/`hex`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone / phase
- `.planning/ROADMAP.md` — Phase 08 goal, success criteria, WIKI-03 (render)
- `.planning/REQUIREMENTS.md` — WIKI-03
- `.planning/PROJECT.md` — v1.10 wiki goals; local-first constraints
- `.planning/research/SUMMARY.md` — Phase 08 deliverables; hand YAML + slugify
- `.planning/research/STACK.md` — no `serde_yaml`; hash fallback for CJK slugs
- `.planning/research/FEATURES.md` — frontmatter / wikilink / index table stakes
- `.planning/research/PITFALLS.md` — CJK slugs, pure render before I/O

### Implementation plan
- `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` — Task 2: `WikiAnalysis`, `WikiPageDraft`, `render_wiki_pages` interfaces (apply discussion overrides for link style / tree / frontmatter)

### Prior phase
- `.planning/phases/07-wiki-kind-config/07-CONTEXT.md` — WikiPage kind + WikiConfig defaults (no re-litigate)
- `.planning/phases/07-wiki-kind-config/07-SUMMARY.md` files — shipped kind/config/label

### Code integration points
- `crates/insights/src/lib.rs` — module export surface for new `wiki` module
- `crates/insights/src/summarize.rs` / `tasks.rs` — existing insights patterns (style only; no I/O in Phase 08)
- Workspace `sha2` / `hex` — slug hash fallback

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/insights` — natural home for `wiki.rs` / `render_wiki_pages` (no module exists yet)
- Workspace `sha2` + `hex` — CJK/empty slug → `e-{hash6}` without new deps
- Plan draft types: `WikiAnalysis`, `WikiEntity`, `WikiConcept`, `WikiPageDraft`, `WikiCompileResult`

### Established Patterns
- Pure logic + unit tests first (TDD); Mock LLM deferred to Phase 09
- Hand-rolled frontmatter strings (research: avoid deprecated `serde_yaml`)
- Feature remains behind `WikiConfig.enabled` at later UI/persist phases — renderer itself is ungated pure function

### Integration Points
- Phase 09 will produce `WikiAnalysis` matching these types
- Phase 10 will write `{slug}.md` under wiki root, set `content_hash`, index `wiki://{slug}` as `SourceKind::WikiPage`
- Phase 12 zip will package the same tree shape

</code_context>

<specifics>
## Specific Ideas

- Prefer path|display wikilinks for Obsidian readability **and** stable paths when titles are CJK / hashed.
- Bidirectional links chosen for navigation completeness even though it slightly complicates the renderer.
- Index is a simple `# Wiki` catalog string in Phase 08; global merge onto disk `index.md` is Phase 10's problem.

</specifics>

<deferred>
## Deferred Ideas

- Filling / overwriting `content_hash` and disk merge of `index.md` — Phase 10
- LLM JSON → `WikiAnalysis` — Phase 09
- Settings / Library compile UX — Phase 11
- Obsidian zip — Phase 12
- Title characters that break wikilink alias syntax (`|`, `]`) escaping policy — not deeply specified; implementer should sanitize display text conservatively if needed (note for planner, not a new phase)

None — discussion stayed within phase scope (no folded todos)

</deferred>

---

*Phase: 08-Deterministic Markdown renderer*
*Context gathered: 2026-07-18*
