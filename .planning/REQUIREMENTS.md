# Requirements: Jarvis v1.11 Related-docs + MCP

**Defined:** 2026-07-25  
**Core Value:** Users can ask questions and run agents against their own indexed knowledge — locally, with citations — and trust that answers come from their data, not the model's training.

## v1 Requirements

Requirements for milestone v1.11. Each maps to roadmap phases 15+.

### Related-docs

- [ ] **REL-01**: User selecting an indexed Library source sees a related-docs panel with loading, empty, and error states
- [x] **REL-02**: Related results are other **sources** (chunk hits rolled up by `source_id`, seed excluded), capped (~3–8), empty when affinity is weak
- [ ] **REL-03**: User can click a related result to navigate/select that source in Library
- [ ] **REL-04**: Each related result shows title, kind label, and a short overlap snippet/reason (not raw similarity scores alone)

### Read-only MCP

- [ ] **MCP-01**: External MCP clients can call `search` using the same hybrid retrieval path as in-app agent knowledge search
- [ ] **MCP-02**: External MCP clients can call `list_sources` to inventory indexed sources
- [ ] **MCP-03**: Project ships a stdio MCP binary (e.g. `jarvis-mcp`) plus Cursor/Claude Desktop config documentation
- [ ] **MCP-04**: MCP tool surface is structurally read-only (allowlist: search + list_sources only; no write/delete/ingest tools)

### Trust & Verification

- [ ] **TRUST-01**: Related-docs and MCP do not change default RAG/citation behavior; existing qa / citation E2E stays green
- [ ] **TRUST-02**: Automated coverage includes related-docs panel visibility + navigate and MCP tool happy paths under `JARVIS_E2E=1` mocks (no live LLM)

## Future Requirements

Deferred beyond v1.11.

### Related-docs

- **REL-F01**: Soft-exclude or demote `WikiPage` neighbors in related-docs
- **REL-F02**: Open related source scoped into chat / ask

### MCP

- **MCP-F01**: `get_chunk` / `read` document tools
- **MCP-F02**: Settings toggle to enable/disable MCP server
- **MCP-F03**: HTTP/SSE local transport (if ever needed; prefer stdio)

### Carryover (unchanged)

- **WIKI-F01**: `auto_on_insights` / bulk compile UX
- Dual `index.md` writers consolidation
- Hard WikiPage RAG citation filter
- DeferredEmbedder / deferred scan in a release build
- Related-docs was previously WIKI-F04 — now split into REL-* / MCP-* above

## Out of Scope

| Feature | Reason |
|---------|--------|
| MCP write / ingest / memory mutate / shell | v1.11 read-only contract |
| Graph UI / Louvain / LanceDB | Core Value is RAG + citations, not graph product |
| Second vector DB for overlap | Reuse hybrid retriever |
| Bidirectional Obsidian sync | Still export-only |
| Bump MSRV solely for `rmcp` 3.x beta | Pin `rmcp` 2.2.0 |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| REL-01 | Phase 16 | Pending |
| REL-02 | Phase 15 | Complete |
| REL-03 | Phase 16 | Pending |
| REL-04 | Phase 16 | Pending |
| MCP-01 | Phase 18 | Pending |
| MCP-02 | Phase 18 | Pending |
| MCP-03 | Phase 17 | Pending |
| MCP-04 | Phase 17 | Pending |
| TRUST-01 | Phase 19 | Pending |
| TRUST-02 | Phase 19 | Pending |

**Coverage:**

- v1 requirements: 10 total
- Mapped to phases: 10
- Unmapped: 0

| Phase | Requirements | Count |
|-------|--------------|-------|
| 15 | REL-02 | 1 |
| 16 | REL-01, REL-03, REL-04 | 3 |
| 17 | MCP-03, MCP-04 | 2 |
| 18 | MCP-01, MCP-02 | 2 |
| 19 | TRUST-01, TRUST-02 | 2 |

---
*Requirements defined: 2026-07-25*
*Last updated: 2026-07-25 — roadmap v1.11 phases 15–19 mapped*
