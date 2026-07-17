# Jarvis

## What This Is

Jarvis is a **local-first personal AI knowledge hub** — a Tauri 2 desktop app for power users who want RAG over their own documents, Feishu/Lark content, and Cursor agent transcripts, with agent-assisted chat, memory, and task extraction. It runs entirely on the user's machine with swappable LLM/embedder providers.

## Current State

**Shipped:** **v1.9** Structural Refactor (2026-07-17) — FE modularization, Tauri command split, keychain secrets, `memory://` URIs, JSON agent protocol, Settings + arch review. Archive: `.planning/milestones/v1.9-ROADMAP.md`.

**App product version** (package): still tracks 1.8.x feature line until a dedicated release bump; planning milestone v1.9 is the refactor gate.

## Current Milestone: v1.10 Wiki Compile Layer

**Goal:** Add an optional, rebuildable Markdown wiki layer beside existing RAG — with Obsidian zip export — without replacing hybrid retrieval or citations.

**Target features:**
- `WikiConfig` (default off) + `SourceKind::WikiPage`
- Insights compile: LLM analysis → entity/concept/source Markdown under `wiki/`
- Index wiki pages through existing ingest pipeline (`content_hash` idempotent)
- Library/Settings: compile notes + export Obsidian zip
- E2E journey: enable → compile → list wiki page → export

**Plan draft:** `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md`

## Core Value

**Users can ask questions and run agents against their own indexed knowledge — locally, with citations — and trust that answers come from their data, not the model's training.**

## Next Milestone Goals

Superseded by **Current Milestone: v1.10 Wiki Compile Layer** (above).

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

### Active

- [ ] Wiki compile layer + Obsidian export (v1.10)
- [ ] Related-docs / MCP read-only surface (post-v1.10)
- [ ] Ship DeferredEmbedder / deferred scan startup fix in a release build

### Out of Scope

- v2.0.0 major version bump — still deferred until product owners want breaking framing
- Full plugin sandbox / DAG orchestration — per design spec non-goals
- Big-bang rewrite — incremental slices only
- Replacing SQLite, Tauri, or React stack
- Wiki graph UI / Louvain / LanceDB / Chrome clipper / Deep Research (v1.10 exclusions)
- Bidirectional Obsidian sync (export-only in v1.10)

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
| Focus on structural refactor (not new features) | v1.8.0 feature set complete; maintenance cost bottleneck | Shipped v1.9 |
| Incremental approach with E2E gate | Brownfield with full E2E harness | Shipped |
| Balanced phasing across layers | Avoid freezing one layer | Shipped |
| v1.9.x version framing | Signal refactor without v2.0 break | Shipped |
| Success = arch review + E2E + keychain | Measurable done criteria | Met |
| Vertical MVP phase structure | End-to-end refactor slices | Shipped |

---
*Last updated: 2026-07-17 after `/gsd-new-milestone` v1.10 Wiki Compile Layer*
