# Jarvis v1.9 — Features Dimension (Structural Refactor)

> **Date:** 2026-06-17  
> **Scope:** v1.9.x incremental refactor — structure, not new product surface  
> **Sources:** `.planning/PROJECT.md`, `.planning/codebase/CONCERNS.md`, `.planning/codebase/ARCHITECTURE.md`, `.planning/codebase/TESTING.md`

This document answers: for a structural refactor release, what is **table stakes** (must preserve), what are **differentiators to protect**, and what are **anti-features** (things not to do)?

---

## Executive Summary

Jarvis v1.8.0 is feature-complete. v1.9.x exists to reduce maintenance cost while keeping the product shippable after every merge. Success is measured by **architecture review passes**, **E2E green**, and **secrets out of plaintext JSON** — not by net-new user features.

**Core value to preserve:** Users can ask questions and run agents against their own indexed knowledge — locally, with citations — and trust answers come from their data.

---

## Table Stakes (Must Preserve)

Behaviors and contracts that must not regress during refactor. Breaking any of these blocks a v1.9 release.

### Cross-cutting

| Must preserve | Why |
|---------------|-----|
| All existing user journeys (Chat, Library, Tasks, Memory, Settings, navigation) | Refactor-only framing; no behavior-breaking releases |
| Hybrid RAG (vector + FTS5 + RRF) with citations | Core product promise |
| Local-first indexing pipeline (watch folders, md/txt/csv/xlsx) | Primary knowledge ingestion |
| Feishu/Lark sync via `lark-cli` subprocess | Validated integration; `CommandRunner` injectable for tests |
| Cursor agent-transcript indexing | v1.1 differentiator |
| Chat sessions (persisted) with streaming | v1.6+ baseline |
| FastEmbed + OpenAI-compatible cloud providers | Default and cloud paths both live |
| Agent mode (tools, skills, hooks, plugins) + pipeline/router orchestration | v1.4–1.7 surface |
| Per-agent chat/embedder provider overrides | v1.7–1.8 behavior |
| Memory CRUD + forget/update tools + UI | v1.8.0 shipped surface |
| Insights (source summaries + task extraction) | v1.2.0 capability |
| E2E test mode (`JARVIS_E2E=1`, deterministic mocks) | CI gate and refactor safety net |
| Existing `config.json` and SQLite DB compatibility (migrate, no data loss) | Brownfield users |
| Single DB owner (`crates/store` only opens SQLite) | Architectural invariant |
| Trait-based providers (`Embedder`, `ChatModel`, `CommandRunner`) | Testability and swappable backends |
| Tauri IPC command signatures (unless explicitly versioned/migrated) | Frontend depends on stable invoke names |
| Index progress events (`IndexProgressEvent` + frontend listeners) | UX during rebuild/sync |

### By category

#### Frontend modularization

| Must preserve | Notes |
|---------------|-------|
| All five views and navigation behavior | Chat · Library · Tasks · Memory · Settings |
| Existing `data-testid` selectors used by E2E | Refactor must not break WebDriver specs |
| IPC via `invoke` / `Channel` / `listen` patterns | Thin UI; business logic stays in Rust |
| `src/lib/tauri.ts` (or equivalent) as IPC wrapper home | Do not scatter raw `invoke` across new components |
| Loading / empty / error states per view | Already shipped UX contract |
| Liquid-glass dark UI aesthetic + accessibility fallbacks | Design system, not refactor target |
| `setReactInputValue` / `clickViaDom` compatibility | E2E helpers assume current DOM/React patterns |

#### Tauri command split

| Must preserve | Notes |
|---------------|-------|
| All ~40 existing Tauri commands (names, args, return shapes) | Split is file organization only unless explicitly migrating |
| `AppState` lifecycle (provider build, watcher, scheduler startup) | Startup order matters |
| `index_ops.rs` orchestration semantics | Rebuild, retry, batch index, progress emission |
| `sync_scheduler.rs` periodic sync behavior | Even if errors are currently swallowed — don't change timing/triggers silently |
| `e2e.rs` bootstrap path | `apply_e2e_config`, fixtures, mock Lark auth |
| `insights_ops.rs` post-index hooks | Auto-insights on index |
| Delegation to crates (thin shell, fat crates) | Commands stay orchestration-only |

#### Config / secrets

| Must preserve | Notes |
|---------------|-------|
| Round-trip load/save of all non-secret settings | Nested structs must serde-compat with flat legacy JSON |
| Provider factory behavior (`build_embedder`, `build_chat_model`) | Mock in E2E, FastEmbed/Ollama/OpenAI in prod |
| Watch folders, agents, skills, hooks, plugins config surface | Users' existing configs must load |
| Settings UI for all current fields | Keychain move must not remove configurability |
| E2E config overrides in `e2e.rs` | Deterministic CI |

#### Agent protocol

| Must preserve | Notes |
|---------------|-------|
| Tool surface: `search_knowledge`, `list_sources`, `list_tasks`, `list_memories`, `add_memory`, `complete_task`, forget/update memory | Agent capabilities unchanged |
| Skills from `skills/*.md`, hooks from `hooks/*.json`, plugins from `plugins/*/plugin.json` | File-based extension model |
| Pipeline and router orchestration modes | Multi-agent paths |
| `MAX_TOOL_ROUNDS` effective behavior (currently 3) | Can make configurable later; don't silently change default |
| Plugin permission gate semantics (even if default is permissive today) | Changing defaults is a product decision, not refactor collateral |

#### Memory model

| Must preserve | Notes |
|---------------|-------|
| Memory appears in Memory UI and participates in RAG/agent retrieval | User-visible memory workflow |
| `learn_from_exchange` auto-learning path | v1.3+ behavior |
| Forget/update via tools and UI | v1.8.0 surface |
| Memories indexed as searchable chunks | Same retrieval pipeline as other sources |
| Existing memory records readable after migration | URI/title scheme change needs migration path |

#### Testing gates

| Must preserve | Notes |
|---------------|-------|
| `cargo test --workspace` green | All crates |
| `npm test` (Vitest) green | Current `src/lib/*.test.ts` |
| `npm run test:e2e:local` green after every phase merge | Primary refactor regression gate |
| CI: Ubuntu Rust + Windows E2E split | Do not break either job |
| Deterministic mocks (no live LLM/Feishu in CI) | Policy from `e2e-required.mdc` |
| TDD workflow: tests before behavior change when behavior changes | Refactor slices that touch behavior need regression tests |

---

## Differentiators to Protect

Capabilities that distinguish Jarvis from generic chat/RAG apps. Refactor must not dilute or accidentally remove these.

| Differentiator | Protect because | Refactor risk |
|----------------|-----------------|---------------|
| **Local-first + citations from user's index** | Core value prop; trust model | Splitting RAG/agent paths could break citation wiring |
| **Multi-source normalization** (files, Lark, Cursor transcripts, memory → `Document`) | Unified index pipeline | Memory refactor could fork ingest path |
| **Hybrid search (RRF)** | Better recall than vector-only | Store/schema changes during memory work |
| **Feishu/Lark via `lark-cli`** | Unique integration; subprocess isolation | Tauri split could break `CommandRunner` injection |
| **Cursor transcript indexing** | v2 milestone delivered; power-user appeal | Config path UI + sync scheduler touch points |
| **Agent extensions** (skills, hooks, plugins, orchestration) | Beyond simple Q&A | Agent protocol change could drop extension points |
| **Per-agent provider overrides** | Power-user tuning | Config nesting could break resolver |
| **Memory as first-class knowledge** (learn, forget, update) | Personalization loop | Dedicated table migration could orphan old memories |
| **Insights** (summarize + task extraction) | Proactive KB utility | `insights_ops` wiring during shell split |
| **Swappable providers without stack change** | Mock/E2E + prod parity | Provider lifecycle moves in `lib.rs` |
| **Deterministic E2E harness** | Enables safe brownfield refactor | E2E branching scattered; easy to diverge mocks |
| **Desktop-native** (Tauri, offline-capable index) | Positioning vs web-only tools | Not applicable unless stack change attempted |

**Protection strategy:** Treat each differentiator as an E2E or integration test anchor. Every refactor phase should map touched differentiators to at least one existing spec (`qa`, `lark`, `memory`, `agent`, `full-ui`).

---

## Anti-Features (Do NOT Do During Refactor)

Explicit out-of-scope actions that increase risk without serving v1.9 goals.

### Global anti-features

| Do NOT | Reason |
|--------|--------|
| v2.0.0 or breaking semver bump | Deferred until refactor complete |
| Big-bang rewrite of any layer | Incremental slices only |
| Replace SQLite, Tauri, or React | Stack is fixed |
| New large user-facing features | Maintenance focus; small fixes in touched files only |
| Full plugin sandbox / DAG orchestration | Design spec non-goals |
| Change retrieval/index quality "while we're in there" | Scope creep; separate milestone |
| Merge refactor + performance overhaul in one phase | Hard to bisect regressions |
| Remove or rename Tauri commands without migration shim | Breaks frontend and E2E |
| Open SQLite outside `crates/store` | Violates single-DB-owner rule |

### By category

#### Frontend modularization

| Do NOT | Do instead |
|--------|------------|
| Introduce a new router framework or state library mid-refactor | Extract views + hooks; keep React patterns |
| Move business logic into frontend | Keep IPC thin; logic stays in Rust crates |
| Rewrite UI visual design / design system | Structure-only; use existing Tailwind/glass patterns |
| Add Vitest component test suite as gate before extraction | E2E is the gate; component tests optional later |
| Change `data-testid` names without updating E2E | Preserve or update specs in same PR |
| Split into micro-frontends or lazy routes that change load order | Same bundle, same startup behavior |

#### Tauri command split

| Do NOT | Do instead |
|--------|------------|
| Combine or rename commands for "cleaner API" | File split only; stable IPC surface |
| Move domain logic from crates into command modules | Commands delegate to crates |
| Inline `index_ops` into `lib.rs` "temporarily" | Keep orchestration modules; split commands by domain |
| Change `AppState` fields without tracing all usages | Mechanical move with compile + E2E check |
| Refactor `sync_scheduler` threading model in same PR as command split | Separate phase |
| Drop `Result<T, String>` error pattern for typed errors without UI plan | String errors are debt; fixing is optional v1.9 slice |

#### Config / secrets

| Do NOT | Do instead |
|--------|------------|
| Store secrets in new plaintext files | OS keychain (`keyring`) for `cloud_api_key` |
| Require manual re-entry of all settings on upgrade | Migrate legacy flat JSON → nested structs |
| Break E2E by requiring real keychain in CI | E2E bypass or in-memory secret store |
| Encrypt entire `config.json` | Secrets only in keychain; non-secrets stay JSON |
| Add new required config fields without defaults | Backward compatible serde defaults |
| Log or emit config values in errors/events | Redact secrets |

#### Agent protocol

| Do NOT | Do instead |
|--------|------------|
| Switch to native function-calling API in one breaking change | Incremental: structured output + fallback to XML+JSON |
| Remove XML+JSON parser before new path is proven | Dual-path during transition |
| Change tool names or JSON schema seen by skills/plugins | Stable tool contract |
| Increase `MAX_TOOL_ROUNDS` default without explicit decision | Configurable yes; silent behavior change no |
| Rewrite orchestration (pipeline/router) during protocol work | Isolate agent protocol to `crates/agent` |
| Default-deny `shell_exec` silently in refactor PR | Security improvement yes — but separate explicit phase with E2E update |

#### Memory model

| Do NOT | Do instead |
|--------|------------|
| Big-bang drop of `SourceKind::Memory` without migration | Dual-read or one-time migration script |
| Break existing title-based forget/update without URI migration | Introduce `memory://{uuid}` + migrate old rows |
| Split memory out of RAG retrieval path | Memories must remain searchable like today |
| Add dedicated `memories` table AND change chunk schema in one PR | Phased: schema → migration → deprecate fuzzy match |
| Mix memory list with Library UI changes | Memory view contract unchanged |

#### Testing gates

| Do NOT | Do instead |
|--------|------------|
| Merge refactor PR with red E2E "to fix later" | E2E green is merge blocker |
| Disable specs to make CI pass | Fix or update deterministically |
| Rely on manual smoke only | Automation is the refactor safety net |
| Add live LLM/Feishu to CI for refactor validation | Mocks and fixtures only |
| Skip E2E when "only backend moved code" | IPC paths still execute |
| Expand E2E coverage as mandatory gate for every slice | Nice-to-have; not blocker unless touching that surface |
| Remove `JARVIS_E2E=1` branching | E2E mode is infrastructure |

---

## Category Deep-Dives

### 1. Frontend modularization

**Goal:** Extract route views and hooks from ~2,800-line `src/App.tsx` without changing user-visible behavior.

**Table stakes:** Navigation, IPC wrappers, E2E selectors, streaming chat UX, index progress listeners, all Settings editors.

**Differentiators to protect:** Agent orchestration UI, hooks/plugins toggles, per-source Library actions, Memory learn/forget flows, Lark sync UI.

**Anti-features:** New state management framework, visual redesign, frontend business logic, breaking E2E helpers.

**Suggested slice pattern:** Extract one view (e.g. `SettingsView`) → verify E2E `settings.spec.ts` + `full-ui.spec.ts` → repeat.

### 2. Tauri command split

**Goal:** Split `src-tauri/src/lib.rs` into `commands/{chat,index,lark,memory,agent,config}.rs`; keep `lib.rs` as registration + `AppState`.

**Table stakes:** Command names/signatures, startup wiring, E2E bootstrap, event emission, provider lifecycle.

**Differentiators to protect:** Lark fixture path, agent command surface, insights hooks, embedder dim/reinit path.

**Anti-features:** API redesign, logic migration into shell, scheduler rewrite in same PR.

**Suggested slice pattern:** Move one command group → `cargo test` + targeted E2E spec → repeat.

### 3. Config / secrets

**Goal:** Nested `AppConfig` structs + OS keychain for API secrets; measurable done = no plaintext `cloud_api_key` in JSON.

**Table stakes:** Legacy config load, all Settings fields, E2E overrides, provider factories.

**Differentiators to protect:** Per-agent overrides, plugin permissions config, watch folder / sync settings.

**Anti-features:** Full-file encryption, breaking migration, keychain in CI without mock.

**Suggested slice pattern:** Nested structs with `serde(flatten)` compat → keychain write/read → migration on first launch → Settings UI reads keychain.

### 4. Agent protocol

**Goal:** Replace fragile `<tool_call>{json}</tool_call>` parsing with structured output — without breaking skills/hooks/plugins.

**Table stakes:** All tool names and behaviors, 3-round loop default, orchestration modes, extension file formats.

**Differentiators to protect:** Text-based tool loop (works with any OpenAI-compatible provider), multi-agent pipeline/router.

**Anti-features:** Provider lock-in to native tools API, removing XML parser before parity, changing tool JSON schema.

**Suggested slice pattern:** Add structured path behind config flag → provider support matrix → dual-parse → deprecate XML when E2E agent spec covers tool UI.

### 5. Memory model

**Goal:** Dedicated storage or strict `memory://` URIs; eliminate fuzzy title collision in `resolve_memory_id`.

**Table stakes:** Memory UI CRUD, RAG retrieval, agent tools (`list_memories`, `add_memory`, forget/update), auto-learn.

**Differentiators to protect:** Memory as indexed knowledge (not a side cache), chat learning loop.

**Anti-features:** Big-bang schema change without migration, breaking existing memories, decoupling from indexer pipeline.

**Suggested slice pattern:** Add URI scheme for new memories → migration for existing rows → switch resolver → remove fuzzy match.

### 6. Testing gates

**Goal:** Every phase merge leaves `cargo test`, `npm test`, and `npm run test:e2e:local` green on Windows.

**Table stakes:** Existing 8 E2E specs, deterministic mocks, CI workflow jobs.

**Differentiators to protect:** E2E as refactor enabler (not afterthought).

**Anti-features:** Red CI, disabled specs, live external services in tests, manual-only verification.

**Known gaps (do not block refactor, but do not widen):** Tasks CRUD E2E, insights/summarize E2E, hooks/plugins E2E, scheduled sync error UI. Filling gaps is optional v1.9 work when touching those files.

---

## Phase Acceptance Checklist

Use at end of each v1.9 slice:

```
[ ] No user-facing behavior change (or documented intentional fix with regression test)
[ ] Tauri IPC surface unchanged (or migration documented)
[ ] config.json + DB backward compatible
[ ] cargo test --workspace — green
[ ] npm test — green
[ ] npm run test:e2e:local — green
[ ] Architecture review: module boundaries match crate layout
[ ] Differentiators touched? → mapped to E2E spec run
[ ] Secrets: no new plaintext credentials on disk
```

---

## References

| Document | Relevance |
|----------|-----------|
| `.planning/PROJECT.md` | v1.9 scope, success criteria, out of scope |
| `.planning/codebase/CONCERNS.md` | Debt items driving each category |
| `.planning/codebase/ARCHITECTURE.md` | Layer boundaries to preserve |
| `.planning/codebase/TESTING.md` | E2E spec map, CI split |
| `AGENTS.md` | Milestones, file boundaries, never/always rules |
| `.cursor/rules/e2e-required.mdc` | E2E mandatory policy |

---

*Generated for GSD v1.9 planning — 2026-06-17*
