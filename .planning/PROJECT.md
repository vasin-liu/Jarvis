# Jarvis

## What This Is

Jarvis is a **local-first personal AI knowledge hub** — a Tauri 2 desktop app for power users who want RAG over their own documents, Feishu/Lark content, and Cursor agent transcripts, with agent-assisted chat, memory, task extraction, an optional Markdown wiki compile layer (Obsidian zip export), Library related-docs discovery, and a read-only stdio MCP server for external agents. It runs entirely on the user's machine with swappable LLM/embedder providers.

## Current State

**Shipped:**
- **v1.9** Structural Refactor (2026-07-17) — FE modularization, Tauri command split, keychain secrets, `memory://` URIs, JSON agent protocol, Settings + arch review. Archive: `.planning/milestones/v1.9-ROADMAP.md`.
- **v1.10** Wiki Compile Layer (2026-07-25) — optional wiki notes beside RAG, Obsidian export, E2E + citation trust, tech-debt closeout. Archive: `.planning/milestones/v1.10-ROADMAP.md`.
- **v1.11** Related-docs + MCP (2026-07-29) — Library related-docs panel + `jarvis-mcp` read-only `search` / `list_sources`. Archive: `.planning/milestones/v1.11-ROADMAP.md`.

**In progress:** **v1.12** Release Hardening — DeferredEmbedder readiness + Windows **1.12.0** local package.

**App product version** (package): still **1.8.0** until this milestone’s release gate bumps to **1.12.0**.

## Core Value

**Users can ask questions and run agents against their own indexed knowledge — locally, with citations — and trust that answers come from their data, not the model's training.**

## Current Milestone: v1.12 Release Hardening

**Goal:** Make FastEmbed deferred init observable and failure-visible, verify Windows release cold-start, then ship local **1.12.0** install package + changelog.

**Target features:**
- `DeferredEmbedder` readiness API (`ready_state` + testable timeout)
- `get_embedder_readiness` IPC + Settings Pending/Ready/Failed UI
- Release smoke → version bump **1.12.0** → `CHANGELOG.md` → `tauri build` (Windows local only)

**Out of this milestone:** GitHub Release, multi-platform packages, Wiki/MCP/REL new features, RAG default changes, auto-fallback embedder provider

**Spec / plan:** `docs/superpowers/specs/2026-07-29-release-hardening-design.md` · `docs/superpowers/plans/2026-07-29-release-hardening.md`

## Next Milestone Goals (after v1.12)

Carryover candidates from backlog:

- WIKI-F01 `auto_on_insights` / bulk compile UX
- REL affinity threshold / WikiPage neighbor demotion (REL-F01)
- MCP read tools expansion (MCP-F01) or Settings toggle (MCP-F02)

## Requirements

### Validated

- ✓ Local file indexing (watch folders, md/txt/csv/xlsx) — v1.4+
- ✓ Hybrid search (vector + FTS5 + RRF) and RAG Q&A with citations — v1.3+
- ✓ Feishu/Lark sync via `lark-cli` subprocess — v1.4+
- ✓ Cursor agent-transcript indexing — v1.1.0
- ✓ Chat sessions (persisted) with streaming — v1.6+
- ✓ FastEmbed local embedder + OpenAI-compatible cloud providers — v1.7–1.8
- ✓ Source summaries + task extraction (insights) — v1.2.0
- ✓ Memory learning from chat (`SourceKind::Memory`) — v1.3.0
- ✓ Agent mode with tools, skills, hooks, plugins — v1.4–1.5
- ✓ Multi-agent pipeline/router orchestration — v1.6–1.7
- ✓ Per-agent chat/embedder provider overrides — v1.7–1.8
- ✓ Memory forget/update tools + UI — v1.8.0
- ✓ E2E test mode with deterministic mocks (`JARVIS_E2E=1`) — v1.0+
- ✓ Frontend view/hook extraction (Chat/Library/Tasks/Memory/Settings) — v1.9
- ✓ Tauri shell command modularization — v1.9
- ✓ Nested AppConfig + OS keychain for API secrets — v1.9
- ✓ `memory://{uuid}` strict identity + migration — v1.9
- ✓ JSON-first agent tool protocol + parse warnings UI — v1.9
- ✓ Architecture review + sync error surfacing — v1.9
- ✓ Wiki compile layer (WIKI-01..WIKI-09) + Obsidian zip + E2E citation trust — v1.10
- ✓ WikiPage reindex soft-skip + export preflight gate + Nyquist closeout — v1.10
- ✓ Related-docs panel + overlap API (REL-01..04) — v1.11
- ✓ Read-only MCP `jarvis-mcp` search / list_sources (MCP-01..04) — v1.11
- ✓ Citation/E2E trust gate for related-docs + MCP (TRUST-01..02) — v1.11

### Active

- [ ] DeferredEmbedder readiness observable + Settings UI (v1.12)
- [ ] Windows release smoke + package **1.12.0** + changelog (v1.12)
- [ ] WIKI-F01 `auto_on_insights` / bulk compile UX (deferred from v1.10)
- [ ] REL-F01 Soft-exclude/demote WikiPage neighbors (optional)
- [ ] MCP-F01 `get_chunk` / `read` tools (optional)

### Out of Scope

- v2.0.0 major version bump — still deferred until product owners want breaking framing
- Full plugin sandbox / DAG orchestration — per design spec non-goals
- Big-bang rewrite — incremental slices only
- Replacing SQLite, Tauri, or React stack
- Wiki graph UI / Louvain / LanceDB / Chrome clipper / Deep Research (v1.10 exclusions)
- Bidirectional Obsidian sync (export-only in v1.10)
- MCP write / mutate tools (v1.11 stayed read-only)
- Hard WikiPage RAG citation filter (deferred; E2E fixture trust)
- GitHub Release / Linux/macOS packages (v1.12 is local Windows only)
- Auto-fallback embedder provider on FastEmbed failure (v1.12: user-visible fail only)

<details>
<summary>v1.11 planning context (archived narrative)</summary>

**Goal:** Related-docs panel + read-only MCP without changing write paths or citation trust.

**Phases 15–19:** overlap API → Library panel → MCP scaffold → live tools → E2E/trust gate.

**Adjusted at ship:** REL-02 weak-affinity empty deferred by CONTEXT D-06 (top-N only).

**Audit:** `.planning/milestones/v1.11-MILESTONE-AUDIT.md` — passed.

</details>

<details>
<summary>v1.10 planning context (archived narrative)</summary>

**Goal:** Optional rebuildable Markdown wiki beside RAG with Obsidian zip export.

**Phases 07–14:** WikiConfig + WikiPage → render → analyze → persist/index → UI → export → E2E → tech-debt closeout.

**Deferred at ship:** WIKI-F01, dual `index.md` writers, hard WikiPage RAG filter.

**Plan draft:** `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md`

</details>

<details>
<summary>v1.9 planning context (archived narrative)</summary>

**Brownfield codebase map:** `.planning/codebase/` (mapped 2026-06-17).

**Former top structural debt (addressed in v1.9):**
- Monolithic `App.tsx` / fat `lib.rs`
- Plaintext `cloud_api_key` in config.json
- Fragile agent XML+JSON tool parse
- Fuzzy memory title matching

**Phasing strategy used:** Balanced vertical MVP slices; E2E-gated.

</details>

## Constraints

- **Tech stack**: Rust stable (MSRV 1.85), Tauri 2, React 19, SQLite — no stack changes
- **Testing**: TDD required; user-facing changes need E2E updates; CI E2E on Windows
- **Incremental**: Each phase must leave the app shippable; `npm run test:e2e:local` green
- **Store ownership**: Only `crates/store` opens SQLite — preserve single-DB-owner rule
- **Compatibility**: Existing user `config.json` and SQLite DBs must migrate without data loss

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Focus on structural refactor (not new features) | v1.8.0 feature set complete; maintenance cost bottleneck | ✓ Shipped v1.9 |
| Incremental approach with E2E gate | Brownfield with full E2E harness | ✓ Shipped |
| Balanced phasing across layers | Avoid freezing one layer | ✓ Shipped |
| v1.9.x version framing | Signal refactor without v2.0 break | ✓ Shipped |
| Success = arch review + E2E + keychain | Measurable done criteria | ✓ Met |
| Vertical MVP phase structure | End-to-end refactor slices | ✓ Shipped |
| Wiki default-off (`WikiConfig`) | No surprise behavior on upgrade | ✓ Good (v1.10) |
| Fail-closed wiki parse (no partial tree) | Protect vault integrity | ✓ Good (v1.10) |
| Soft-skip WikiPage reindex (not true MD reindex) | Avoid Failed stubs / chunk wipe | ✓ Good (v1.10) |
| Defer WIKI-F01 / dual index writers | Ship without accepting audit gaps on required path | ✓ Accepted (D-14) |
| v1.11 = related-docs panel + read-only MCP | Plan draft Out of Scope table; both halves this milestone | ✓ Shipped v1.11 |
| `related_sources` reuses hybrid `retrieve` (no score field / no affinity threshold) | Same retrieval brain as RAG; D-06/D-09; SC#2 deferred | ✓ Good (v1.11) |
| Shared `kb_readonly` for agent + MCP | Prevent semantic drift between surfaces | ✓ Good (v1.11) |
| Pin `rmcp` 2.2.0 (not 3.x beta) | MSRV / stability; no bump solely for MCP | ✓ Good (v1.11) |
| WAL on `Store::open` | GUI + MCP concurrent readers | ✓ Good (v1.11) |
| v1.12 = release hardening (1.12.0 product) | Decouple GSD feature milestones from ship version; reliability gate before bump | — Pending |
| Keep DeferredEmbedder + deferred initial_scan | Harden existing cold-start path; no embedder rewrite | — Pending |
| No GitHub Release in v1.12 | Local Windows package + changelog is enough for this ship bar | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-07-29 — Milestone v1.12 Release Hardening started*
