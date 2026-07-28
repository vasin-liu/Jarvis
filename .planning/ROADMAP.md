# Roadmap: Jarvis

**Updated:** 2026-07-25

## Milestones

- ✅ **v1.9 Structural Refactor** — Phases 01–06 (shipped 2026-07-17) — [archive](./milestones/v1.9-ROADMAP.md)
- ✅ **v1.10 Wiki Compile Layer** — Phases 07–14 (shipped 2026-07-25) — [archive](./milestones/v1.10-ROADMAP.md)
- 🚧 **v1.11 Related-docs + MCP** — Phases 15–19 (in progress)

## Phases

<details>
<summary>✅ v1.9 Structural Refactor (Phases 01–06) — SHIPPED 2026-07-17</summary>

- [x] Phase 01–06 — 22 plans — completed 2026-07-17
- Details: [v1.9-ROADMAP.md](./milestones/v1.9-ROADMAP.md) · phases: [v1.9-phases/](./milestones/v1.9-phases/)

</details>

<details>
<summary>✅ v1.10 Wiki Compile Layer (Phases 07–14) — SHIPPED 2026-07-25</summary>

- [x] Phase 07: Wiki kind + config (3/3) — completed 2026-07-18
- [x] Phase 08: Deterministic Markdown renderer (2/2) — completed 2026-07-19
- [x] Phase 09: LLM wiki analysis (2/2) — completed 2026-07-19
- [x] Phase 10: Persist + index (2/2) — completed 2026-07-21
- [x] Phase 11: Library / Settings UI (2/2) — completed 2026-07-23
- [x] Phase 12: Obsidian zip export (2/2) — completed 2026-07-23
- [x] Phase 13: E2E + citation trust (2/2) — completed 2026-07-24
- [x] Phase 14: Tech debt closeout (3/3) — completed 2026-07-25

Details: [v1.10-ROADMAP.md](./milestones/v1.10-ROADMAP.md) · phases: [v1.10-phases/](./milestones/v1.10-phases/) · audit: [v1.10-MILESTONE-AUDIT.md](./milestones/v1.10-MILESTONE-AUDIT.md)

</details>

## v1.11 Related-docs + MCP

**Goal:** Let users discover overlapping sources in Library and let external agents query the local KB read-only — without changing write paths or citation trust.

**Constraints:** `store` sole SQLite owner; pin `rmcp` 2.2.0 (not 3.x beta); no MCP write/mutate tools; TDD + E2E for user-facing.

### Phases

- [x] **Phase 15: Overlap scoring API** - Hybrid retrieve → source rollup (`related_sources`) (completed 2026-07-25)
- [x] **Phase 16: Related-docs Library panel** - Selection, panel UI, navigate (completed 2026-07-28)
- [x] **Phase 17: MCP transport + read-only scaffold** - `jarvis-mcp` stdio binary + structural allowlist (completed 2026-07-28)
- [x] **Phase 18: MCP tools search + list_sources** - Shared `kb_readonly` helpers (completed 2026-07-28)
- [ ] **Phase 19: E2E + citation regression gate** - Panel journey + qa/full-ui trust

### Phase Details

### Phase 15: Overlap scoring API

**Goal**: Callers can get top overlapping indexed sources for a seed source via hybrid retrieval, without noise or self-hits
**Depends on**: Nothing (v1.11 start; builds on shipped retriever/store)
**Requirements**: REL-02
**Success Criteria** (what must be TRUE):

  1. Given a seed source id, the API returns other sources only (seed excluded), rolled up by `source_id` from chunk hits
  2. Results are capped (~3–8); when affinity is weak the result set is empty (prefer empty over junk neighbors)
  3. Overlap uses the existing hybrid retrieve path (vector + FTS + RRF) — not a second vector store or cosine-% UI contract
  4. Unit/integration tests with MockEmbedder pass for related vs unrelated fixtures (`cargo test -p retriever`)

**Plans**: 2/2 plans complete

Plans:
**Wave 1**

- [x] 15-01-PLAN.md — RED: RelatedSource stub + failing REL-02 integration tests

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 15-02-PLAN.md — GREEN: related_sources hybrid rollup implementation + full retriever gate

### Phase 16: Related-docs Library panel

**Goal**: Users selecting a Library source see honest related neighbors and can open them
**Depends on**: Phase 15
**Requirements**: REL-01, REL-03, REL-04
**Success Criteria** (what must be TRUE):

  1. Selecting an indexed Library source shows a related-docs panel with loading, empty, and error states
  2. Each related row shows title, kind label, and a short overlap snippet/reason (not raw similarity scores alone)
  3. Clicking a related result navigates/selects that source in Library
  4. Panel uses stable `data-testid`s suitable for E2E (no graph UI scope creep)

**Plans**: 4/4 plans complete
**UI hint**: yes

Plans:
**Wave 1**

- [x] 16-01-PLAN.md — Serialize RelatedSource + thin list_related_sources IPC + TS wrappers
- [x] 16-02-PLAN.md — Dual E2E fixture seed (related-neighbor.md + seed_e2e_fixture)

**Wave 2** *(blocked on 16-01)*

- [x] 16-03-PLAN.md — Library selection + related panel UI + Vitest (REL-01/03/04)

**Wave 3** *(blocked on 16-02 + 16-03)*

- [x] 16-04-PLAN.md — Focused related-docs E2E journey + e2e-required spec map

### Phase 17: MCP transport + read-only scaffold

**Goal**: External hosts can spawn a local stdio MCP binary whose tool surface is structurally read-only
**Depends on**: Phase 15 (shared Store/data-dir assumptions; can parallelize with Phase 16 after 15)
**Requirements**: MCP-03, MCP-04
**Success Criteria** (what must be TRUE):

  1. Workspace ships a console stdio binary (e.g. `jarvis-mcp`) separate from the Windows GUI subsystem app
  2. `tools/list` exposes exactly `{search, list_sources}` — no write/delete/ingest/memory-mutate tool names
  3. Binary resolves KB via documented data-dir (`JARVIS_DATA_DIR` / `--db`); Cursor/Claude Desktop config docs exist
  4. Only `crates/store` opens SQLite; pin is `rmcp` 2.2.0 (not 3.x beta / no MSRV bump solely for MCP)

**Plans**: 4 plans

Plans:
**Wave 1**

- [x] 17-01-PLAN.md — Scaffold `crates/mcp` + path resolution (`--db` / `JARVIS_DATA_DIR` / AppData) + missing-DB fail-closed
- [x] 17-02-PLAN.md — Enable SQLite WAL in `Store::open` for GUI+MCP coexistence

**Wave 2** *(blocked on 17-01)*

- [x] 17-03-PLAN.md — `JarvisMcp` stub tools exactly `{search, list_sources}` + allowlist tests

**Wave 3** *(blocked on 17-01 + 17-02 + 17-03)*

- [x] 17-04-PLAN.md — stdio main wiring + `docs/mcp.md` + README pointer

### Phase 18: MCP tools search + list_sources

**Goal**: External MCP clients get the same hybrid search and source inventory semantics as in-app agent tools
**Depends on**: Phase 17
**Requirements**: MCP-01, MCP-02
**Success Criteria** (what must be TRUE):

  1. MCP `search` uses the same hybrid retrieval path as in-app agent knowledge search (shared helper, not a fork)
  2. MCP `list_sources` inventories indexed sources with bounded payload size
  3. Agent `search_knowledge` / `list_sources` and MCP handlers share `kb_readonly` (or equivalent) so semantics cannot drift
  4. `cargo test -p mcp` (and agent tests covering shared helpers) pass offline with mocks

**Plans**: 4 plans

Plans:
**Wave 1**

- [x] 18-01-PLAN.md — TDD: `retriever::kb_readonly` search_kb + list_indexed_sources

**Wave 2** *(blocked on 18-01)*

- [x] 18-02-PLAN.md — Agent thin wrappers over kb_readonly (anti-drift)

**Wave 3** *(blocked on 18-01 + 18-02)*

- [x] 18-03-PLAN.md — MCP real handlers + build_embedder fail-closed + tools_kb tests

**Wave 4** *(blocked on 18-03)*

- [x] 18-04-PLAN.md — Update docs/mcp.md for live tools + caps

### Phase 19: E2E + citation regression gate

**Goal**: Related-docs and MCP ship without regressing RAG citation trust or user-facing coverage
**Depends on**: Phase 16, Phase 18
**Requirements**: TRUST-01, TRUST-02
**Success Criteria** (what must be TRUE):

  1. Existing qa / citation E2E (and full-ui citation trust) stay green — default RAG/citation behavior unchanged
  2. Automated coverage includes related-docs panel visibility + navigate under `JARVIS_E2E=1` mocks (no live LLM)
  3. MCP tool happy paths are verified offline (cargo harness); `npm run test:e2e:local` green for the related-docs journey
  4. e2e-required spec map updated for related-docs; README/docs MCP config snippet remains accurate

**Plans**: 3 plans
**UI hint**: yes (E2E verification of existing panel — no new chrome)

Plans:
**Wave 1**

- [ ] 19-01-PLAN.md — Thin full-ui related-docs step + e2e-required map confirm
- [ ] 19-02-PLAN.md — cargo test -p mcp green + docs/mcp.md + README drift check

**Wave 2** *(blocked on Wave 1)*

- [ ] 19-03-PLAN.md — Citation + related-docs E2E gate + 19-VERIFICATION evidence

## Progress

| Phase | Milestone | Plans | Status | Completed |
|-------|-----------|-------|--------|-----------|
| 07. Wiki kind + config | v1.10 | 3/3 | Complete | 2026-07-18 |
| 08. Markdown renderer | v1.10 | 2/2 | Complete | 2026-07-19 |
| 09. LLM wiki analysis | v1.10 | 2/2 | Complete | 2026-07-19 |
| 10. Persist + index | v1.10 | 2/2 | Complete | 2026-07-21 |
| 11. Library / Settings UI | v1.10 | 2/2 | Complete | 2026-07-23 |
| 12. Obsidian zip export | v1.10 | 2/2 | Complete | 2026-07-23 |
| 13. E2E + citation trust | v1.10 | 2/2 | Complete | 2026-07-24 |
| 14. Tech debt closeout | v1.10 | 3/3 | Complete | 2026-07-25 |
| 15. Overlap scoring API | v1.11 | 2/2 | Complete    | 2026-07-25 |
| 16. Related-docs Library panel | v1.11 | 4/4 | Complete    | 2026-07-28 |
| 17. MCP transport + read-only scaffold | v1.11 | 4/4 | Complete    | 2026-07-28 |
| 18. MCP tools search + list_sources | v1.11 | 4/4 | Complete    | 2026-07-28 |
| 19. E2E + citation regression gate | v1.11 | 0/3 | Planned | - |

## Backlog (post-v1.11 / carryover)

- Bulk compile + `auto_on_insights` UX (WIKI-F01) — deferred OK through v1.10 audit (D-14)
- Cross-corpus entity merge
- Dual `index.md` writers consolidation (Phase 09 test helper vs compile rebuild) — intentional; defer (D-14)
- Ship DeferredEmbedder / deferred scan startup fix in a release build
- REL-F01: Soft-exclude/demote WikiPage neighbors in related-docs
- REL-F02: Open related source scoped into chat / ask
- MCP-F01: `get_chunk` / `read` document tools
- MCP-F02: Settings toggle to enable/disable MCP server
- MCP-F03: HTTP/SSE local transport

---

*Next:* `/gsd-execute-phase 18`
