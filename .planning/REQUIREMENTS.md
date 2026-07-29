# Requirements: Jarvis v1.12 Release Hardening

**Defined:** 2026-07-29  
**Core Value:** Users can ask questions and run agents against their own indexed knowledge — locally, with citations — and trust that answers come from their data, not the model's training.

## v1 Requirements

Requirements for milestone v1.12. Each maps to roadmap phases 20+.

### Boot / Embedder readiness

- [x] **BOOT-01**: Developer can observe deferred FastEmbed init as Pending, Ready, or Failed (including failure message) via a non-blocking readiness API on `DeferredEmbedder`
- [x] **BOOT-02**: App exposes embedder readiness to the UI over IPC (`get_embedder_readiness` or equivalent) without opening SQLite outside `store`
- [x] **BOOT-03**: User opening Settings can see Pending / Ready / Failed for local embedder init, with Failed showing a reason and actionable hint (no silent hang)
- [x] **BOOT-04**: When FastEmbed init fails, subsequent embed/index attempts return a clear error (not an indefinite wait with no feedback)

### Release ship

- [ ] **SHIP-01**: Release Windows cold-start smoke checklist exists and is executed before version bump (window interactive; readiness visible)
- [ ] **SHIP-02**: After SHIP-01 passes, `package.json` and `tauri.conf.json` are bumped to **1.12.0**
- [ ] **SHIP-03**: Project ships a `CHANGELOG.md` summarizing 1.9–1.11 features plus 1.12.0 hardening
- [ ] **SHIP-04**: `tauri build` produces a Windows install/bundle artifact that launches in a final smoke check

### Trust (no regression)

- [ ] **TRUST-01**: Default RAG / citation / `RetrieverConfig` behavior is unchanged; existing Mock-path unit/Vitest (and relevant E2E) stay green
- [ ] **TRUST-02**: CI/E2E does not require downloading a real FastEmbed model; release smoke is the FastEmbed cold-start authority

## Future Requirements

Deferred beyond v1.12.

- Unify Settings `reload_providers` with deferred cold-start path
- GitHub Release automation for Windows assets
- Linux / macOS packages
- WIKI-F01, REL-F01/F02, MCP-F01..F03 (from backlog)

## Out of Scope

| Feature | Reason |
|---------|--------|
| GitHub Release / multi-platform | v1.12 ship bar is local Windows only |
| Wiki / MCP / related-docs new features | Release hardening only |
| RAG / RetrieverConfig default changes | Trust freeze |
| Auto-fallback to Mock/Ollama on FastEmbed fail | User must choose provider explicitly |
| v2.0.0 major framing | Still deferred |
| Embedder architecture rewrite | Harden existing DeferredEmbedder |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| BOOT-01 | Phase 20 | Complete |
| BOOT-04 | Phase 20 | Complete |
| BOOT-02 | Phase 21 | Complete |
| BOOT-03 | Phase 22 | Complete |
| SHIP-01 | Phase 23 | Pending |
| SHIP-02 | Phase 23 | Pending |
| SHIP-03 | Phase 23 | Pending |
| SHIP-04 | Phase 23 | Pending |
| TRUST-01 | Phase 23 | Pending |
| TRUST-02 | Phase 23 | Pending |

**Coverage:**

- v1 requirements: 10 total
- Mapped to phases: 10
- Unmapped: 0

---
*Requirements defined: 2026-07-29*
*Last updated: 2026-07-29 — roadmap phases 20–23 mapped*
