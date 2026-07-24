# Requirements: Jarvis v1.10 Wiki Compile Layer

**Defined:** 2026-07-17  
**Core Value:** Users can ask questions and run agents against their own indexed knowledge — locally, with citations — and trust that answers come from their data, not the model's training.

## v1 Requirements

Requirements for milestone v1.10. Each maps to roadmap phases 07+.

### Config & Source Kind

- [x] **WIKI-01**: User can enable/disable wiki via `WikiConfig` (`enabled`, `auto_on_insights`); both default **false** so existing installs see no behavior change
- [x] **WIKI-02**: System persists wiki pages as `SourceKind::WikiPage` (`wiki_page`) with Library label (e.g. Wiki / 笔记页)

### Compile

- [x] **WIKI-03**: User can compile an indexed source into Markdown wiki pages (source summary + entities + concepts) with YAML frontmatter, `[[wikilinks]]`, and `index.md`
- [x] **WIKI-04**: System writes pages under `{app_data}/wiki/` and indexes them through the existing ingest→index pipeline using stable `wiki://{slug}` URIs and `content_hash` skip (idempotent re-compile)
- [x] **WIKI-05**: On LLM JSON parse failure, system does **not** write a partial wiki tree (fail closed)

### Library & Settings UX

- [x] **WIKI-06**: When wiki is enabled, user can trigger “生成笔记” from Library and toggle wiki in Settings; controls are hidden when disabled
- [x] **WIKI-07**: User can export the wiki tree as an Obsidian-compatible zip (includes minimal `.obsidian/` stub)

### Trust & Verification

- [x] **WIKI-08**: With wiki off or on, RAG Q&A still cites original sources when relevant — wiki pages are additive, not a replacement for citations
- [x] **WIKI-09**: E2E journey covers enable → compile → list wiki page → export under `JARVIS_E2E=1` mocks; default-off does not break `full-ui`

## Future Requirements

Deferred past v1.10.

### Wiki enhancements

- **WIKI-F01**: `auto_on_insights` wired with progress UX (config field may exist; full auto-compile UX deferred)
- **WIKI-F02**: Bulk compile all indexed sources + progress events
- **WIKI-F03**: Cross-corpus entity merge (one entity page across many sources)
- **WIKI-F04**: Related-docs panel / read-only MCP surface (post-v1.10 Active in PROJECT)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Knowledge graph UI / Louvain | High cost; Obsidian export covers viewing |
| LanceDB / second vector store | Violates single-DB-owner; duplicates sqlite-vec |
| Bidirectional Obsidian sync | Conflict/merge complexity; export-only |
| Chrome clipper / Deep Research | Separate products |
| Wiki replaces RAG Q&A | Breaks Core Value / citation trust |
| Full schema.md / lint / review queues | Faithful llm_wiki product; too large for v1.10 |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| WIKI-01 | 07 | Complete |
| WIKI-02 | 07 | Complete |
| WIKI-03 | 08 + 09 | Complete |
| WIKI-04 | 10 | Complete |
| WIKI-05 | 09 | Complete |
| WIKI-06 | 11 | Complete |
| WIKI-07 | 12 | Complete |
| WIKI-08 | 13 | Complete |
| WIKI-09 | 13 | Complete |

**Coverage:**

- v1 requirements: 9 total
- Mapped to phases: 9
- Unmapped: 0 ✓

---
*Requirements defined: 2026-07-17*
*Last updated: 2026-07-17 after roadmap draft*
