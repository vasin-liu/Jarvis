# Project Research Summary

**Project:** Jarvis
**Domain:** Local-first RAG knowledge hub — optional Wiki Compile Layer
**Researched:** 2026-07-17
**Confidence:** HIGH

## Executive Summary

v1.10 adds an **optional, rebuildable Markdown wiki** (entity/concept/source pages + Obsidian zip) **beside** existing hybrid RAG — not a replacement. Compile lives in `crates/insights`, pages land under `{app_data}/wiki/` and re-enter the index as `SourceKind::WikiPage` via the existing Document→indexer path. Default `wiki.enabled = false` keeps upgrades behavior-neutral.

Stack additions are minimal: stable **`zip` 2.4.2** (+ optional `walkdir`), hand-rolled frontmatter/slugify (avoid deprecated `serde_yaml`). Critical risks are idempotency (`content_hash` + `wiki://` URIs), citation crowding, CJK/unsafe slugs, parse-fail partial writes, wiki↔compile feedback loops, and zip path safety — each mapped to phases 07–13.

## Key Findings

### Recommended Stack

Preserve Tauri/Rust/SQLite/React. Add only export/walk deps; keep LLM via existing `ChatModel` + Mock.

**Core technologies:**
- **`zip` 2.4.2** — Obsidian vault export (not 9.x pre)
- **Hand YAML + slugify** — frontmatter / path segments (CJK → hash fallback)
- **`walkdir` 2.5** (optional) — zip tree walk
- **Existing insights + indexer + store** — no new DB

### Expected Features

**Must have (table stakes):**
- Opt-in `WikiConfig` (default off)
- LLM JSON → summary/entities/concepts → Markdown + `[[wikilinks]]` + `index.md`
- Index as `WikiPage` with hash skip; Library compile + Obsidian zip; E2E

**Should have (competitive):**
- Wiki **beside** RAG (citations stay on originals)
- Reuse insights/indexer/`SourceKind` (Memory precedent)
- `generated: true` overwrite policy only

**Defer (v2+ / out of scope):**
- Graph UI, LanceDB, bidirectional Obsidian sync, clipper, Deep Research, cross-corpus entity merge

### Architecture Approach

Side path off insights → write wiki files → `index_document(WikiPage)`. Thin Tauri `commands/wiki.rs`; Library/Settings gated by flag. No retriever redesign.

**Major components:**
1. **insights/wiki.rs** — analyze, render, persist, index wiring
2. **WikiConfig + SourceKind::WikiPage** — flag + kind
3. **export_wiki_zip** — deterministic zip + `.obsidian` stub
4. **Library UI + wiki.spec.ts** — user journey

### Critical Pitfalls

1. **No content_hash / unstable URI** — duplicate embeds — Phase **10**
2. **Wiki crowds out citations** — policy + E2E — Phases **10/13**
3. **Default-on / auto-compile** — serde defaults + UI gate — Phase **07/11**
4. **CJK / unsafe slugs** — hash fallback + tests — Phase **08**
5. **Partial write on LLM parse fail / compile loops** — fail closed; skip WikiPage as compile input — Phases **09/10**

## Implications for Roadmap

Phase numbering continues after v1.9 (**07+**).

### Phase 07: Wiki kind + config
**Rationale:** Foundation; default-off safety first  
**Delivers:** `SourceKind::WikiPage`, `WikiConfig`, sourceDisplay label  
**Avoids:** Default-on pitfall

### Phase 08: Deterministic Markdown renderer
**Rationale:** Pure logic before LLM/I/O  
**Delivers:** `render_wiki_pages`, slugify, frontmatter, wikilinks  
**Avoids:** CJK/path pitfalls

### Phase 09: LLM → WikiAnalysis
**Rationale:** Needs renderer contract  
**Delivers:** `analyze_source_for_wiki`, Mock-friendly JSON, fail-closed parse  
**Avoids:** Partial-write on bad JSON

### Phase 10: Persist + index
**Rationale:** Disk + DB idempotency  
**Delivers:** write `wiki/`, `wiki://` URIs, `index_document`, skip WikiPage inputs  
**Avoids:** Dual-write, compile loops

### Phase 11: IPC + Library/Settings UI
**Rationale:** User-facing gated by enabled  
**Delivers:** compile/list commands, 生成笔记, Settings toggle, testids  
**Avoids:** UI when disabled

### Phase 12: Obsidian zip export
**Rationale:** Needs wiki tree on disk  
**Delivers:** `export_wiki_zip`, zip 2.4.2, path-safe entries  
**Avoids:** Zip-slip / absolute paths

### Phase 13: E2E + citation regression
**Rationale:** Mandatory for user-facing  
**Delivers:** `wiki.spec.ts`, default-off full-ui, QA citation still on originals  
**Avoids:** “Looks done” without journey

### Phase Ordering Rationale

- Config/kind before any write path
- Pure render before LLM and I/O
- Persist before UI/export
- E2E last as release gate

### Research Flags

- **Phase 10:** Citation ranking policy if wiki dominates RRF — may need light retriever filter (research if E2E fails)
- **Phase 12:** Pin exact `zip` features for Windows CI size
- **Phases 07–09, 11, 13:** Standard patterns — skip deep re-research at plan-phase

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | crates.io `cargo info zip` → 2.4.2 stable |
| Features | HIGH | Plan + llm_wiki / Obsidian patterns |
| Architecture | HIGH | Matches Memory/`SourceKind` precedent |
| Pitfalls | HIGH | Mapped to phases 07–13 |

**Overall confidence:** HIGH

### Gaps to Address

- Exact zip feature flags for minimal binary — decide in Phase 12 plan
- Whether auto_on_insights ships in v1.10 UI or config-only — confirm in requirements scoping
- Citation demotion of WikiPage — only if Phase 13 E2E shows crowding

## Sources

### Primary (HIGH confidence)
- `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md`
- crates.io / `cargo info zip` (2026-07-17)
- Existing Jarvis crates: insights, store, indexer, config

### Secondary (MEDIUM confidence)
- nashsu/llm_wiki / Karpathy LLM Wiki pattern notes
- Obsidian vault + `.obsidian` stub conventions

---
*Research completed: 2026-07-17*
*Ready for roadmap: yes*
