# Jarvis

## What This Is

Jarvis is a **local-first personal AI knowledge hub** — a Tauri 2 desktop app for power users who want RAG over their own documents, Feishu/Lark content, and Cursor agent transcripts, with agent-assisted chat, memory, and task extraction. It runs entirely on the user's machine with swappable LLM/embedder providers.

**Current state:** v1.8.0 brownfield — feature-complete through v9 milestones (RAG, Lark sync, Cursor indexing, memory, tasks, insights, multi-agent orchestration, E2E harness). This GSD project targets **v1.9.x structural refactor** to reduce maintenance cost without breaking user-facing behavior.

## Core Value

**Users can ask questions and run agents against their own indexed knowledge — locally, with citations — and trust that answers come from their data, not the model's training.**

Refactor work must preserve this: every phase keeps E2E green and does not regress retrieval, indexing, or chat quality.

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

### Active

- [ ] Incremental frontend refactor — extract route views and hooks from monolithic `src/App.tsx`
- [ ] Incremental Tauri shell refactor — split `src-tauri/src/lib.rs` into `commands/*` modules
- [ ] Config hardening — nested `AppConfig` structs + OS keychain for API secrets
- [ ] Agent protocol improvement — structured tool calls (replace fragile XML+JSON parsing)
- [ ] Memory model improvement — dedicated storage or strict `memory://` URIs (no title collision)
- [ ] Architecture review passes for each phase — module boundaries match crate layout
- [ ] All E2E specs remain green after every phase merge

### Out of Scope

- v2.0.0 major version bump — deferred until refactor complete and reviewed
- New large user-facing features — only small features that touch files already being refactored
- Full plugin sandbox / DAG orchestration — per design spec non-goals
- Big-bang rewrite — incremental slices only; no behavior-breaking releases
- Replacing SQLite, Tauri, or React stack — refactor structure, not foundations

## Context

**Brownfield codebase map:** `.planning/codebase/` (7 documents, mapped 2026-06-17).

**Top structural debt (from CONCERNS.md):**
- `src/App.tsx` ~2,800 lines — all views, state, IPC in one component
- `src-tauri/src/lib.rs` ~1,300 lines — ~40 Tauri commands in one file
- `config.json` stores `cloud_api_key` in plaintext
- Agent tool loop parses `<tool_call>{json}</tool_call>` from raw LLM text
- Memory uses `SourceKind::Memory` with fuzzy title matching

**Milestone framing:** v1.9.x refactor-only releases. Small features OK when they touch refactored files.

**Phasing strategy:** Balanced — interleave small slices across frontend, shell, config, agent, and memory layers rather than finishing one layer entirely first.

**Success criteria:** Architecture review passes + E2E green + secrets in OS keychain (not plaintext JSON).

## Constraints

- **Tech stack**: Rust stable (MSRV 1.85), Tauri 2, React 19, SQLite — no stack changes
- **Testing**: TDD required; user-facing changes need E2E updates; CI E2E on Windows
- **Incremental**: Each phase must leave the app shippable; `npm run test:e2e:local` green
- **Store ownership**: Only `crates/store` opens SQLite — preserve single-DB-owner rule
- **Compatibility**: Existing user `config.json` and SQLite DBs must migrate without data loss

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Focus on structural refactor (not new features) | v1.8.0 feature set is complete; maintenance cost is the bottleneck | — Pending |
| Incremental approach with E2E gate | Brownfield with full E2E harness; big-bang too risky | — Pending |
| Balanced phasing across layers | Avoid long periods where one layer is frozen | — Pending |
| v1.9.x version framing | Signal refactor releases without v2.0 breaking-change expectations | — Pending |
| Small features allowed in touched files | Avoid blocking legitimate fixes during refactor | — Pending |
| Success = arch review + E2E + keychain | Measurable done criteria beyond "feels cleaner" | — Pending |
| Vertical MVP phase structure | Each phase delivers an end-to-end refactor slice | — Pending |
| Skip greenfield research on "what is Jarvis" | Codebase map already documents domain | — Pending |

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
*Last updated: 2026-06-17 after initialization*
