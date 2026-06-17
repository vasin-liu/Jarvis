# Phase 1: Scaffold + Shell Foundation - Research

**Researched:** 2026-06-17  
**Domain:** Tauri 2 shell refactor + React frontend scaffold (zero behavior change)  
**Confidence:** HIGH (brownfield codebase inspected; planning docs locked)

<user_constraints>
## User Constraints (from CONTEXT.md)

**CRITICAL:** Locked decisions from `/gsd-discuss-phase` — planner and executor MUST honor these.

### Phase boundary constraints (verbatim)

- Zero behavior change
- config+library first command modules; optional sync.rs
- Stub views only - App.tsx retains logic
- index_ops.rs untouched
- E2E gate: smoke + navigation only
- data-testid unchanged

### Locked Decisions

**Tauri Command Split**
- **D-01:** Extract `commands/config.rs` + `commands/library.rs` as the first two modules (low coupling, read-heavy).
- **D-02:** Optional 3rd module: `commands/sync.rs` (`get_sync_status`, `run_scheduled_sync_cmd`) — smallest module wins; skip if it adds risk.
- **D-03:** `commands/mod.rs` re-exports command fns; `lib.rs` keeps `generate_handler![...]` listing names from `commands::`.
- **D-04:** Private helpers move with their command module when extracted; helpers for not-yet-extracted modules stay in `lib.rs`.

**AppState (`state.rs`)**
- **D-05:** Move `AppState` struct definition + impl methods (e.g., `reload_providers`, watcher/scheduler lifecycle) to `state.rs`.
- **D-06:** `init_state()` / `setup()` orchestration stays in `lib.rs` for Phase 1.
- **D-07:** `lark_opts`, `seed_skills_dir` / `seed_hooks_dir` / `seed_plugins_dir`, and agent glue (`build_agent_context`, `execute_agent_question`, `agent_response_to_ask`) **stay in `lib.rs`** until their target phases (Lark Phase 3, agent Phase 5).

**Frontend Scaffold**
- **D-08:** Create stub views: one file per view under `src/views/` (`ChatView.tsx`, `LibraryView.tsx`, etc.) re-exporting fragments still defined in `App.tsx`.
- **D-09:** Extract `View` type to `src/types/view.ts`; routing logic stays in `App.tsx`.
- **D-10:** No shared component extraction (Sidebar, banners) in Phase 1 — defer to later phases.
- **D-11:** All existing `data-testid` values unchanged; selectors stay on same DOM elements in App.tsx.

**IPC Typing Layer**
- **D-12:** Create `src/types/ipc.ts` by moving TS interfaces from App.tsx (`ChatSession`, `Source`, `TaskItem`, etc.).
- **D-13:** Create `src/lib/tauri.ts` with wrappers **only** for Phase 1 extracted commands (config, library, sync).
- **D-14:** App.tsx continues direct `invoke()` calls in Phase 1 — migrate to `tauri.ts` starting Phase 2 Chat extraction.
- **D-15:** Error handling: passthrough `Result<T, String>` — no new frontend error taxonomy in Phase 1.

**index_ops.rs**
- **D-16:** Leave `index_ops.rs` completely untouched in Phase 1; refactor when `commands/index.rs` is extracted (roadmap Phase 3 / research S2f).

**Quality Gates**
- **D-17:** E2E merge gate: `smoke.spec.ts` + `navigation.spec.ts` only (roadmap minimum).
- **D-18:** `cargo test --workspace` + `npm test` required on Phase 1 merge (QA-03).
- **D-19:** Regression bar = zero behavior change — no IPC renames, no testid moves, no new features; line-count reduction optional.

**Stub Structure & PR Slicing**
- **D-20:** One stub file per view; App.tsx imports each directly (no barrel `views/index.ts` unless planner finds benefit).
- **D-21:** Single Phase 1 PR preferred (shell + FE scaffold together); if split, shell lands first and each PR passes full gate.

### Claude's Discretion

- Exact file naming inside `commands/` (e.g., `library.rs` vs `sources.rs`) as long as grouping matches research map.
- Whether optional 3rd `sync.rs` is included if diff stays minimal.
- Stub implementation detail (named re-export vs thin wrapper component) as long as App.tsx owns all logic and E2E passes.

### Deferred Ideas (OUT OF SCOPE)

- Component extraction (Sidebar, IndexProgressBanner) → Phase 2+ / Phase 6
- Full `lib/tauri.ts` migration → Phase 2 Chat extraction
- `index_ops.rs` + `commands/index.rs` → Phase 3
- Agent/lark/memory command modules → Phases 3–5
- Full E2E suite as Phase 1 gate → later phases (Phase 1 uses smoke+navigation minimum)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Requirement (REQUIREMENTS.md) | Phase 1 interpretation | Verification signal |
|----|------------------------------|--------------------------|---------------------|
| **FE-01** | `App.tsx` reduced to layout shell, navigation, and view router only | **Partial scaffold only** — create `src/views/*` stubs + type extraction; all JSX/state/IPC remain in `App.tsx` until Phase 2+ | `src/views/` exists (5 files); `App.tsx` imports stubs; nav testids unchanged |
| **SHELL-01** | `AppState` moved to `state.rs`; `lib.rs` is registration + startup only | **Partial** — move `AppState` + impl to `state.rs`; extract 2–3 command modules; `lib.rs` still holds ~44 commands + agent/Lark glue + `init_state` | `state.rs` + `commands/mod.rs` + `commands/{config,library,sync?}.rs` compile; `generate_handler!` unchanged IPC names |
| **QA-01** | `npm run test:e2e:local` green after every phase merge | Run **smoke + navigation** specs only (D-17) | `e2e/specs/smoke.spec.ts`, `e2e/specs/navigation.spec.ts` pass |
| **QA-03** | `cargo test --workspace` and `npm test` green after every phase merge | Full Rust workspace + Vitest | `cargo test --workspace`; `npm test` |
</phase_requirements>

<architectural_responsibility_map>
## Architectural Responsibility Map

Jarvis is a **two-tier desktop app** (Tauri webview + Rust shell). Phase 1 touches both tiers structurally without changing semantics.

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| View routing / nav UI | **Browser (React `src/`)** | — | Presentation; `App.tsx` owns `view` state and `data-testid="nav-*"` |
| IPC command handlers (config/library/sync) | **API/Backend (`src-tauri/commands/`)** | Application (`state.rs`, `sync_scheduler.rs`) | Tauri commands validate + delegate; no new domain logic |
| Runtime composition (`AppState`, watcher, scheduler) | **API/Backend (`state.rs` + `lib.rs` setup)** | Infrastructure crates (`config`, `watcher`) | Composition root stays in `src-tauri` |
| Type mirrors for IPC payloads | **Browser (`src/types/ipc.ts`)** | — | TS interfaces mirror Rust serde shapes; no runtime logic |
| E2E contract (`data-testid`, invoke names) | **Cross-cutting invariant** | CI (`e2e/specs/`) | Refactor must preserve public IPC + selector API |

**Dependency rule (unchanged):** Domain crates ↛ Tauri/React. Phase 1 moves files within `src-tauri` and `src/` only — **no `crates/*` changes**, **no `index_ops.rs` changes**.
</architectural_responsibility_map>

<research_summary>
## Summary

Phase 1 establishes extraction points for the v1.9.x refactor by splitting the Tauri shell monolith (`src-tauri/src/lib.rs`, ~1,317 lines, **52** `#[tauri::command]` handlers) and scaffolding the React monolith (`src/App.tsx`, ~2,798 lines) without moving business logic or changing user-visible behavior.

**Backend:** Move `AppState` (lines 54–68) and its `impl` block (lines 77–183) to new `state.rs`. Extract **8 IPC commands** into `commands/config.rs` (3), `commands/library.rs` (3), and optionally `commands/sync.rs` (2). Keep `init_state` (1183–1240), `run()` setup (1243–1257), agent/Lark/seed helpers, and `generate_handler![...]` (1260–1313) in `lib.rs`. `index_ops.rs`, `sync_scheduler.rs`, `insights_ops.rs`, and `e2e.rs` are **not modified** except for import path updates (`crate::AppState` → `crate::state::AppState` or re-export).

**Frontend:** Extract `View` to `src/types/view.ts` and ~15 interfaces to `src/types/ipc.ts`. Add five stub view files under `src/views/` that delegate to JSX still rendered from `App.tsx` (thin wrappers or re-exports). Add partial `src/lib/tauri.ts` wrappers for the 8 extracted commands only; **App.tsx keeps direct `invoke()`** per D-14.

**Primary recommendation:** Follow the existing `index_ops.rs` / `sync_scheduler.rs` split pattern — one module per concern, `pub(crate)` re-exports, commands delegate to crates/state helpers, zero IPC signature changes. Gate with `cargo test --workspace`, `npm test`, and E2E smoke + navigation only.
</research_summary>

<standard_stack>
## Standard Stack

No new packages expected for Phase 1. Use existing pinned stack:

### Core

| Component | Version (repo) | Purpose | Why Standard |
|-----------|----------------|---------|--------------|
| **Tauri** | `2.x` (`src-tauri/Cargo.toml`: `tauri = "2"`) | Desktop shell, IPC, events | Project baseline; command module pattern is idiomatic Rust `mod` + `generate_handler!` |
| **React** | `^19.1.0` | UI | Existing monolith; Phase 1 adds folders only |
| **TypeScript** | `~5.8.3` | Frontend types | `src/types/*` extraction |
| **Rust workspace** | MSRV **1.85**, edition **2021** | Domain + shell | `cargo test --workspace` gate |
| **Vitest** | `^3.2.4` | `src/lib/*.test.ts` | QA-03 frontend gate |
| **WebdriverIO + tauri-driver** | WDIO 9.x | E2E | QA-01; Windows CI |

### Supporting (unchanged)

| Tool | Purpose | Phase 1 use |
|------|---------|-------------|
| `@tauri-apps/api` | `invoke`, `listen`, `Channel` | No API changes |
| `cross-env` | E2E env injection | `test:e2e:local` script |
| Existing crates (`store`, `config`, `sync_scheduler`, …) | Domain delegation | Commands call same paths as today |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| File move + re-export | Macro-generated command registry | Over-engineering; Tauri expects explicit `generate_handler!` list |
| Barrel `views/index.ts` | Direct per-view imports (D-20) | Barrel adds indirection with no Phase 1 benefit |
| Skip `sync.rs` | 2 modules only | Valid per D-02 if `sync_scheduler` import cycles appear |

**Installation:** None required for Phase 1.
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Phase 1 data flow (unchanged semantics)

```
React App.tsx (view router, all logic)
    │ invoke("get_config" | "list_sources" | …)  [unchanged names]
    ▼
lib.rs :: generate_handler![…]  [same 52 names, some fns moved to commands/*]
    │
    ├── commands/config.rs   → state.rs (AppState) → config crate / index_ops::index_status_view
    ├── commands/library.rs  → state.rs → store crate
    ├── commands/sync.rs?    → state.rs → sync_scheduler::
    └── lib.rs (remaining 44 commands + init_state + setup)
    │
    ▼
Domain crates (store, config, …) — untouched
```

### Recommended project structure (Phase 1 ONLY)

```
src/
├── App.tsx                    # Still owns all view JSX, state, invoke, listen
├── types/
│   ├── view.ts                # export type View = "chat" | …
│   └── ipc.ts                 # ChatSession, Source, AppConfig, … (from App.tsx L34–218)
├── lib/
│   ├── tauri.ts               # NEW: wrappers for 8 Phase-1 commands only
│   ├── citations.ts           # existing
│   └── sourceDisplay.ts       # existing
└── views/
    ├── ChatView.tsx           # stub → content still composed in App.tsx
    ├── LibraryView.tsx
    ├── TasksView.tsx
    ├── MemoryView.tsx
    └── SettingsView.tsx

src-tauri/src/
├── lib.rs                     # mod declarations, run(), setup, init_state, generate_handler!, agent/lark glue
├── state.rs                   # NEW: AppState struct + impl (L54–183 today)
├── commands/
│   ├── mod.rs                 # pub use config::*; pub use library::*; [pub use sync::*;]
│   ├── config.rs              # get_config, set_config, get_index_status
│   ├── library.rs             # source_count, list_sources, remove_source
│   └── sync.rs                # OPTIONAL: get_sync_status, run_scheduled_sync_cmd
├── index_ops.rs               # UNTOUCHED (D-16)
├── sync_scheduler.rs          # UNTOUCHED (import path for AppState only)
├── insights_ops.rs            # UNTOUCHED
└── e2e.rs                     # UNTOUCHED (import path for AppState only)
```

### Pattern 1: Tauri 2 command module split (`mod.rs` + `generate_handler!`)

**What:** Move `#[tauri::command]` fns to submodules; re-export for handler registration.  
**When:** Phase 1 config/library/(sync).  
**Example (target shape):**

```rust
// src-tauri/src/commands/mod.rs
mod config;
mod library;
mod sync;

pub use config::{get_config, set_config, get_index_status};
pub use library::{source_count, list_sources, remove_source};
pub use sync::{get_sync_status, run_scheduled_sync_cmd};
```

```rust
// src-tauri/src/lib.rs
mod state;
mod commands;

use commands::{
    get_config, set_config, get_index_status,
    source_count, list_sources, remove_source,
    get_sync_status, run_scheduled_sync_cmd,
    // … remaining commands still defined in lib.rs for Phase 1
};

// in run():
.invoke_handler(tauri::generate_handler![
    source_count,
    list_sources,
    remove_source,
    get_config,
    set_config,
    get_index_status,
    // … exact same 52 names, same order not required
    get_sync_status,
    run_scheduled_sync_cmd,
    // …
])
```

**Critical:** Command **names** are the IPC contract (`snake_case` fn names). Do not rename fns or change signatures.

### Pattern 2: `AppState` in `state.rs` with `pub(crate)` visibility

**What:** `sync_scheduler.rs` and `e2e.rs` already use `crate::AppState` and access `state.store`, `state.config()`.  
**When:** Moving struct out of `lib.rs`.  
**Example:**

```rust
// src-tauri/src/state.rs
pub(crate) struct AppState { /* fields unchanged */ }

impl AppState {
    pub(crate) fn config(&self) -> AppConfig { … }
    pub(crate) fn reload_providers(&self, cfg: &AppConfig) -> Result<(), String> { … }
    // restart_watcher, restart_scheduler, initial_scan_with_progress, save_config, …
}
```

```rust
// src-tauri/src/lib.rs
mod state;
pub(crate) use state::AppState; // preserves crate::AppState for e2e/sync_scheduler
```

### Pattern 3: Frontend stub views (zero behavior change)

**What:** Stub components imported by `App.tsx`; JSX and `data-testid` remain in `App.tsx` until Phase 2+.  
**Options (planner discretion per D-08):**
- Named export of a fragment renderer passed props from `App.tsx`
- Thin component that returns `children` only
- Re-export of inline section wrapped later

**Anti-pattern:** Moving `data-testid="nav-chat"` or view sections into stubs in Phase 1 — breaks D-11 unless DOM structure is identical (risky).

### Command inventory — Phase 1 extraction (from `lib.rs`)

#### `commands/config.rs` (3 commands)

| Command | Lines | Signature | Delegates to |
|---------|-------|-----------|--------------|
| `get_config` | 208–211 | `State<AppState>` → `Result<AppConfig, String>` | `state.config()` |
| `get_index_status` | 213–221 | `State<AppState>` → `Result<IndexStatusView, String>` | `index_ops::index_status_view(store, cfg, embedder)` |
| `set_config` | 223–229 | `(AppConfig, AppHandle, State<AppState>)` → `Result<(), String>` | `state.save_config`, `restart_watcher`, `restart_scheduler` |

**Module-private helpers:** None today — logic is in `AppState` methods (move with `state.rs`).

#### `commands/library.rs` (3 commands)

| Command | Lines | Signature | Delegates to |
|---------|-------|-----------|--------------|
| `source_count` | 185–192 | `State<AppState>` → `Result<i64, String>` | `store.list_sources().len()` |
| `list_sources` | 194–197 | `State<AppState>` → `Result<Vec<Source>, String>` | `store.list_sources()` |
| `remove_source` | 199–206 | `(String id, State<AppState>)` → `Result<(), String>` | `store.delete_chunks_for_source`, `store.delete_source` |

#### `commands/sync.rs` (optional — 2 commands)

| Command | Lines | Signature | Delegates to |
|---------|-------|-----------|--------------|
| `get_sync_status` | 1007–1010 | `State<AppState>` → `Result<SyncStatusView, String>` | `sync_scheduler::sync_status_view(&state)` |
| `run_scheduled_sync_cmd` | 1012–1018 | `(AppHandle, State<AppState>)` → `Result<RebuildReport, String>` | `sync_scheduler::run_scheduled_sync` |

**Note:** `sync_status_view` reads `state.store.get_meta("last_scheduled_sync_at")` — requires `AppState` field access from `sync_scheduler.rs` (already does today).

#### Remaining in `lib.rs` (44 commands + helpers)

All other `#[tauri::command]` handlers stay for later phases. Key **stay-in-lib.rs** non-command items per D-06/D-07:

| Symbol | Lines | Reason |
|--------|-------|--------|
| `init_state` | 1183–1240 | D-06: setup orchestration |
| `run()` / `setup` closure | 1243–1257 | D-06 |
| `lark_opts` | 70–75 | D-07: Lark phase |
| `seed_skills_dir`, `seed_hooks_dir`, `seed_plugins_dir` | 595–650 | D-07 |
| `build_agent_context` | 652–668 | D-07: agent phase |
| `execute_agent_question` | 833+ | D-07 |
| `agent_response_to_ask` | (near 833) | D-07 |
| `emit_index_progress`, `emit_index_complete` | 231–237 | Used by `index_ops`; keep `pub(crate)` in `lib.rs` or small `events.rs` — **not** Phase 1 scope to move |

### AppState field/method map (`state.rs` vs `lib.rs`)

#### Move to `state.rs` (D-05)

**Struct fields (L54–68):**

| Field | Type | Accessed by |
|-------|------|-------------|
| `store` | `Arc<Store>` | All commands, sync_scheduler, e2e |
| `embedder` | `Mutex<Arc<dyn Embedder>>` | index/RAG/agent paths |
| `chat` | `Mutex<Arc<dyn ChatModel>>` | RAG/agent paths |
| `chunker` | `ChunkerConfig` | watcher thread, indexing |
| `retriever` | `RetrieverConfig` | agent context |
| `config_path` | `PathBuf` | save_config |
| `db_path` | `PathBuf` | init only (stored) |
| `skills_dir`, `hooks_dir`, `plugins_dir` | `PathBuf` | agent commands (stay invoked from lib.rs) |
| `config` | `Mutex<AppConfig>` | config(), save_config |
| `watch` | `Mutex<Option<WatchHandle>>` | restart_watcher |
| `scheduler` | `Mutex<Option<SchedulerHandle>>` | restart_scheduler |

**Impl methods (L77–183):**

| Method | Move to `state.rs` |
|--------|-------------------|
| `config()` | Yes |
| `embedder()` | Yes |
| `chat()` | Yes |
| `reload_providers()` | Yes |
| `save_config()` | Yes |
| `restart_watcher()` | Yes |
| `restart_scheduler()` | Yes |
| `initial_scan_with_progress()` | Yes |

#### Stay in `lib.rs` (D-06, D-07)

| Symbol | Lines | Phase |
|--------|-------|-------|
| `init_state()` | 1183–1240 | Phase 1 (orchestration) |
| `lark_opts()` | 70–75 | Lark (Phase 3) |
| `seed_*_dir()` | 595–650 | Agent (Phase 5) |
| `build_agent_context()` | 652–668 | Agent (Phase 5) |
| `execute_agent_question()` | 833+ | Agent (Phase 5) |
| `ConfigChatResolver` / `ConfigEmbedResolver` | ~780–831 | Agent (Phase 5) |
| All non-extracted `#[tauri::command]` fns | scattered | Phases 2–6 |

### Frontend type extraction map

#### `src/types/view.ts`

| Export | Source (App.tsx) |
|--------|------------------|
| `View` | L32: `"chat" \| "library" \| "tasks" \| "memory" \| "settings"` |

#### `src/types/ipc.ts` (move from App.tsx L34–218)

| Interface | App.tsx line |
|-----------|--------------|
| `ChatSession` | 34 |
| `ChatMessage` | 41 |
| `Source` | 50 |
| `TaskItem` | 61 |
| `InsightsReport` | 72 |
| `IndexProgressView` | 78 |
| `LarkAuthStatus` | 87 |
| `AppConfig` | 101 |
| `OrchestrationStepInfo` | 138 |
| `ToolCallInfo` | 145 |
| `AskResponse` | 151 |
| `AgentProfile` | 158 |
| `PluginItem` | 167 |
| `SkillItem` | 175 |
| `HookItem` | 182 |
| `SyncStatusView` | 190 |
| `CursorTranscriptSummary` | 196 |
| `IndexStatusView` | 203 |
| `RebuildReport` | 214 |

**Keep in App.tsx Phase 1:** `SourceKindIcon`, `StatusBadge`, `IndexProgressBanner` (D-10).

#### View stub files ↔ App.tsx sections

| Stub file | App.tsx `view ===` block | Key testids (must stay in App.tsx DOM) |
|-----------|--------------------------|----------------------------------------|
| `ChatView.tsx` | L1209–1460 | `chat-session-ready`, `chat-input`, `ask-submit`, `agent-mode-toggle` |
| `LibraryView.tsx` | L1462–1700 | `library-stats`, `source-list`, `lark-sync-url-input` |
| `TasksView.tsx` | L1702–1774 | `task-list`, `task-list-items` |
| `MemoryView.tsx` | L1776–1900 | `new-memory-input`, `memory-list`, `add-memory-submit` |
| `SettingsView.tsx` | L1902+ | `settings-panel`, `index-status`, `rebuild-index` |

Nav testids (sidebar L1171–1190): `nav-chat`, `nav-library`, `nav-tasks`, `nav-memory`, `nav-settings`.

#### `src/lib/tauri.ts` (Phase 1 wrappers only — D-13)

| Wrapper | IPC command | Used by App.tsx today |
|---------|-------------|----------------------|
| `getConfig()` | `get_config` | L410 `refreshConfig` |
| `setConfig(config)` | `set_config` | L873 |
| `getIndexStatus()` | `get_index_status` | L415 |
| `sourceCount()` | `source_count` | (via library stats — optional) |
| `listSources()` | `list_sources` | L396 |
| `removeSource(id)` | `remove_source` | L860 |
| `getSyncStatus()` | `get_sync_status` | L366 |
| `runScheduledSync()` | `run_scheduled_sync_cmd` | L1054 |

**D-14:** Do not migrate App.tsx call sites to these wrappers in Phase 1 — wrappers exist for Phase 2+ and documentation of contracts.

### Anti-patterns to avoid

- **Renaming commands** during file moves (PITFALLS §5)
- **Moving `data-testid` attributes** into stubs with different mount timing (PITFALLS §2)
- **Touching `index_ops.rs`** “while we're here” (D-16)
- **Command-to-command calls** — extracted modules delegate to `AppState`/crates, not each other (PITFALLS §4)
</architecture_patterns>

<runtime_state_inventory>
## Runtime State Inventory

Refactor must preserve all runtime contracts below. **Do not rename, relocate, or alter behavior.**

### 1. IPC command names (52 total — frozen)

All names in `generate_handler!` (`lib.rs` L1260–1313). Phase 1 moves **8** fn bodies; **0** renames.

| Group | Commands |
|-------|----------|
| **Phase 1 extract** | `get_config`, `set_config`, `get_index_status`, `source_count`, `list_sources`, `remove_source`, `get_sync_status`, `run_scheduled_sync_cmd` |
| **Remain in lib.rs** | `rebuild_index`, `reinit_and_rebuild_index`, `retry_source`, `list_cursor_transcripts`, `sync_cursor_transcripts_cmd`, `add_watch_folder`, `remove_watch_folder`, `list_chat_sessions`, `create_chat_session`, `delete_chat_session`, `list_chat_messages`, `is_e2e_mode_cmd`, `start_ask_e2e`, `ask_in_session`, `ask_in_session_stream`, `check_lark_connection`, `detect_lark_cli`, `sync_lark_doc`, `sync_lark_url`, `sync_lark_sheet`, `sync_lark_mail`, `sync_lark_im`, `index_file`, `ask_question`, `summarize_source_cmd`, `extract_tasks_cmd`, `run_insights_all_cmd`, `list_tasks`, `update_task_status`, `delete_task`, `list_memories_cmd`, `get_memory_content_cmd`, `forget_memory_cmd`, `update_memory_cmd`, `add_memory_cmd`, `list_agent_profiles`, `list_skills`, `list_hooks`, `list_plugins`, `upsert_agent_profile`, `remove_agent_profile`, `set_active_agent`, `ask_agent_in_session`, `ask_agent_in_session_stream` |

**Frontend invoke audit (App.tsx):** 30+ distinct invoke sites; highest-traffic for Phase 1 gates: `get_config`, `get_index_status`, `list_sources`, `list_chat_sessions`, `create_chat_session`, `is_e2e_mode_cmd`.

### 2. Config and runtime data paths

| Path | Set in | Production | E2E (`JARVIS_E2E=1`) |
|------|--------|------------|----------------------|
| App data root | `init_state` L1184–1195 | `app.path().app_data_dir()` | `app_cache_dir()/jarvis-e2e` (wiped each run) |
| `config.json` | L1197 | `{app_data}/config.json` | same under e2e dir |
| `kb.sqlite` | L1198 | `{app_data}/kb.sqlite` | same |
| `skills/` | L1199 | `{app_data}/skills` | seeded |
| `hooks/` | L1200 | `{app_data}/hooks` | seeded |
| `plugins/` | L1201 | `{app_data}/plugins` | seeded |

**Startup side effects (must preserve order in `setup`):** `init_state` → E2E `seed_e2e_fixture` OR `initial_scan_with_progress` → `restart_watcher` → `restart_scheduler` → `app.manage(state)`.

### 3. E2E environment variables

| Variable | Set by | Read by | Required for Phase 1 gate |
|----------|--------|---------|---------------------------|
| `JARVIS_E2E=1` | `npm run test:e2e:local` (`package.json` L13) | `e2e.rs::is_e2e_mode()` | **Yes** |
| `JARVIS_E2E_FIXTURE=./e2e/fixtures/sample.md` | `test:e2e:local` | `e2e.rs::seed_e2e_fixture` | **Yes** (local script) |

**E2E behavioral branches to preserve:** Mock embedder/chat via `apply_e2e_config`; no scheduler spawn when `is_e2e_mode()` (`AppState::restart_scheduler` L152–154); fixture index at startup; `lark_auth_status` mock; `lark_fixture_from_input` URL markers (`E2E_LARK_URL`, `E2E_LARK_MARKER`).

### 4. Tauri event names (Emitter ↔ listen)

| Event | Emitter | Listener (App.tsx) | Payload type |
|-------|---------|-------------------|--------------|
| `index-progress` | `emit_index_progress` (L231–233) | L460–462 | `IndexProgressView` |
| `index-complete` | `emit_index_complete` (L235–237) | L463–467 | `RebuildReport` |

Channel streaming (`TokenEvent` on ask/agent stream) — unchanged; not Phase 1 extraction.

### 5. E2E `data-testid` contract (navigation + smoke gate)

**Verified spec files exist:** `e2e/specs/smoke.spec.ts`, `e2e/specs/navigation.spec.ts`.

| testid | Spec | App.tsx location |
|--------|------|------------------|
| `app-root` | wdio `before` hook | L1155 |
| `e2e-active` | qa/agent helpers | L1156 |
| `nav-chat` | smoke, navigation | L1183 |
| `nav-settings` | smoke | L1183 |
| `nav-library`, `nav-tasks`, `nav-memory` | navigation | L1183 |
| `settings-panel` | smoke, navigation | L1905 |
| `index-status` | smoke, navigation | L1916 |
| `ask-submit` | navigation (chat) | L1450 |
| `library-stats` | navigation | L1472 |
| `task-list` | navigation | L1705 |
| `new-memory-input`, `memory-list` | navigation | L1795, L1815 |

**Invariant:** Phase 1 stubs must not become the mount point for these selectors unless DOM hierarchy is byte-identical (forbidden by D-11).
</runtime_state_inventory>

<common_pitfalls>
## Common Pitfalls

Applicable entries from `.planning/research/PITFALLS.md` for Phase 1:

### Pitfall 1: Big-bang extraction (PITFALLS §1)

**What goes wrong:** Moving all commands or all views at once — unreviewable diff, E2E bisect impossible.  
**How to avoid:** Phase 1 scope = 8 commands + stubs only; `index_ops.rs` frozen.  
**Warning signs:** PR touches chat/agent/lark commands or moves JSX out of `App.tsx`.

### Pitfall 2: `data-testid` / E2E breakage (PITFALLS §2)

**What goes wrong:** Relocating testids into stub components changes mount order; WebDriver waits fail.  
**How to avoid:** D-11 — testids stay on same elements in `App.tsx`.  
**Warning signs:** `smoke.spec.ts` or `navigation.spec.ts` fail on `nav-*` or `settings-panel`.

### Pitfall 3: IPC signature drift (PITFALLS §5)

**What goes wrong:** Renaming fns or changing `Result` types breaks frontend `invoke` (no TS compile check).  
**How to avoid:** `generate_handler!` list uses same 52 identifiers; move bodies only.  
**Warning signs:** Grep mismatch between `invoke("` in `src/` and `#[tauri::command]` fn names.

### Pitfall 4: Circular deps after `lib.rs` split (PITFALLS §4)

**What goes wrong:** `commands/sync.rs` ↔ `sync_scheduler` ↔ `commands` cycles.  
**How to avoid:** Commands depend on `AppState` + domain crates; `sync_scheduler` already depends on `AppState` — do not import commands from scheduler.  
**Warning signs:** `mod` cycle errors; new `Arc<Mutex>` fields to break cycles.

### Pitfall 5: E2E mock path divergence (PITFALLS §9)

**What goes wrong:** Refactored `init_state` or `set_config` skips `apply_e2e_config` / scheduler guard.  
**How to avoid:** Keep `init_state` in `lib.rs`; run existing `e2e.rs` unit test after shell move.  
**Warning signs:** E2E passes but production scheduler double-starts; or E2E hangs without fixture.

### Pitfall 6: Skipping E2E for “refactor only” (PITFALLS §10)

**What goes wrong:** `cargo test` green but navigation broken.  
**How to avoid:** D-17 — mandatory `npm run test:e2e:local` (smoke + navigation).  
**Warning signs:** PR lacks E2E checklist; only Rust tests run locally.

### Pitfall 7: Scope creep — features in refactor PR (PITFALLS §18)

**What goes wrong:** “Small” Settings fix bundled with command split invalidates regression signal.  
**How to avoid:** D-19 — zero behavior change; line count optional.  
**Warning signs:** New UI strings, changed defaults, or config schema edits in Phase 1 PR.
</common_pitfalls>

<validation_architecture>
## Validation Architecture (Nyquist)

Maps Phase 1 requirements to **automated commands** and **existing spec files**. Phase 1 does not add new user-facing behavior — verification is regression-only.

### Requirement → test command matrix

| Requirement | Behavior to verify | Automated command | Spec / artifact | File exists |
|-------------|-------------------|-------------------|-----------------|-------------|
| **FE-01** (scaffold) | `src/views/` + `src/types/` present; app builds; nav renders | `npm run build` | — | N/A (structural) |
| **FE-01** (no regression) | All nav views load | `npm run test:e2e:local` (includes smoke + all specs*) | `e2e/specs/navigation.spec.ts` | ✅ |
| **SHELL-01** | Rust shell compiles; workspace tests pass | `cargo test --workspace` | `src-tauri/src/state.rs`, `commands/*.rs` | ❌ (to create) |
| **SHELL-01** | IPC registry intact (52 commands) | `cargo test --workspace` + manual grep of `generate_handler!` | `lib.rs` L1260–1313 | ✅ |
| **QA-01** | E2E local green | `npm run test:e2e:local` | `e2e/specs/smoke.spec.ts` | ✅ |
| **QA-01** | E2E local green | `npm run test:e2e:local` | `e2e/specs/navigation.spec.ts` | ✅ |
| **QA-03** | Rust full suite | `cargo test --workspace` | CI `.github/workflows/ci.yml` | ✅ |
| **QA-03** | Vitest helpers | `npm test` | `src/lib/*.test.ts` | ✅ |

\*Note: `test:e2e:local` runs **all** `e2e/specs/**/*.spec.ts` per wdio config; Phase 1 **merge gate** is smoke + navigation minimum (D-17). Running full local E2E before merge is recommended but gate is explicitly those two specs.

### Nyquist sampling plan (Phase 1)

| When | Command | Max latency (approx) |
|------|---------|----------------------|
| After each task / commit | `cargo test --workspace` | ~minutes |
| After frontend touch | `npm test` | ~seconds |
| Before phase merge | `npm run test:e2e:local` | ~3–5 min (build + wdio) |
| Pre-merge targeted | `npm run tauri build` then wdio smoke+navigation only* | ~5 min |

\*Targeted run (optional): `npx wdio run e2e/wdio.conf.ts --spec e2e/specs/smoke.spec.ts --spec e2e/specs/navigation.spec.ts` with same env as `test:e2e:local`.

### E2E spec verification (Phase 1 gate)

| File | Path | Tests |
|------|------|-------|
| Smoke | `e2e/specs/smoke.spec.ts` | `nav-chat` visible; `nav-settings` → `settings-panel` + `index-status` |
| Navigation | `e2e/specs/navigation.spec.ts` | All 5 views via `openNav()` + view-specific testids |

### Wave 0 / new test expectations

Phase 1 likely **does not require new tests** if behavior unchanged (D-19). Optional hardening:

| Area | Suggestion | Priority |
|------|------------|----------|
| `commands/config.rs` | No new unit tests required — delegation-only | Low |
| `state.rs` | Existing workspace tests cover store/config; shell move is structural | Low |
| Import regression | `cargo test -p tauri-app` if crate tests added later | Deferred |

**Existing infrastructure covers Phase 1 requirements** — primary gate is green regression suites.
</validation_architecture>

<security_domain>
## Security Domain (ASVS)

Phase 1 is a **structural refactor with zero behavior change** — no new secrets handling, auth surfaces, or trust boundaries.

| ASVS area | Phase 1 applicability | Notes |
|-----------|-------------------------|-------|
| V1 Architecture | **N/A** | No new components; same Tauri capability model |
| V2 Authentication | **N/A** | No auth changes |
| V3 Session management | **N/A** | |
| V4 Access control | **N/A** | Plugin/hook shell_exec defaults unchanged — not in Phase 1 scope |
| V5 Validation | **Low** | Preserve `Result<T, String>` passthrough (D-15); no new input validators |
| V6 Stored cryptography | **N/A** | Keychain deferred to Phase 3 (CFG-02/03) |
| V7 Error handling | **N/A** | No change to error taxonomy |
| V8 Data protection | **Watch** | Do not log `AppConfig` / paths in new debug code; no functional change |
| V9 Communication | **N/A** | IPC names unchanged |
| V10 Malicious code | **N/A** | Agent/hook execution untouched |
| V11 Business logic | **N/A** | |
| V12 Files | **N/A** | Same app data paths |
| V13 API | **N/A** | |
| V14 Config | **Watch** | `set_config` move must still use `save_config` — no plaintext key handling change |

**Threat model for Phase 1:** Accidental exposure of `cloud_api_key` via new logging in moved command code — **prevent by code review**, not new controls.

**Planner action:** Include minimal `<threat_model>` in PLAN.md: “structural move only; preserve config write path; no new network or shell surfaces.”
</security_domain>

<open_questions>
## Open Questions

1. **Include `sync.rs` or stop at 2 modules?**
   - What we know: 2 commands, 11 lines total (L1007–1018); depends on `sync_scheduler` + `AppState`.
   - Recommendation: Include if `pub(crate) use state::AppState` avoids cycles; skip if compile cycle appears (D-02).

2. **Stub view pattern — wrapper vs re-export?**
   - What we know: E2E requires DOM in `App.tsx`.
   - Recommendation: Stubs export typed props interfaces for Phase 2; render `null` or pass-through `children` only.

3. **FE-01 completion vs Phase 1 scaffold**
   - What we know: REQUIREMENTS.md FE-01 describes end state; CONTEXT limits Phase 1 to stubs.
   - Recommendation: Mark FE-01 **partial** in VERIFICATION.md at phase end; full FE-01 completes Phase 6 per roadmap.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)

- `.planning/phases/01-scaffold-shell-foundation/01-CONTEXT.md` — locked decisions D-01–D-21
- `.planning/ROADMAP.md` — Phase 1 success criteria
- `.planning/REQUIREMENTS.md` — FE-01, SHELL-01, QA-01, QA-03
- `.planning/research/ARCHITECTURE.md` — S0 scaffold, command grouping map
- `.planning/research/PITFALLS.md` — refactor pitfalls
- `.planning/codebase/ARCHITECTURE.md`, `STRUCTURE.md`, `TESTING.md`, `CONCERNS.md`
- `src-tauri/src/lib.rs` — command inventory (52 handlers), AppState, generate_handler
- `src/App.tsx` — View type, interfaces, view sections, testids
- `e2e/specs/smoke.spec.ts`, `e2e/specs/navigation.spec.ts` — gate specs (verified on disk)
- `package.json` — test scripts
- `src-tauri/src/e2e.rs` — E2E env contract

### Secondary (MEDIUM confidence)

- `AGENTS.md` — stack policy, E2E mandatory rule
- `.cursor/rules/e2e-required.mdc` — E2E policy alignment with D-17
</sources>

<metadata>
## Metadata

**Research scope:**
- Tauri 2 command module split pattern
- AppState / command line-level map from production `lib.rs`
- Frontend stub + types extraction from production `App.tsx`
- Runtime contract inventory (IPC, paths, E2E, events, testids)
- Validation mapping for Nyquist / VALIDATION.md downstream

**Confidence breakdown:**
- Standard stack: **HIGH** — pinned in repo manifests
- Architecture / line maps: **HIGH** — direct codebase read 2026-06-17
- Pitfalls: **HIGH** — aligned with PITFALLS.md + CONTEXT
- Validation: **HIGH** — spec files verified present

**Research date:** 2026-06-17  
**Valid until:** 2026-07-17 (30 days — stable brownfield refactor)
</metadata>

---

*Phase: 01-scaffold-shell-foundation*  
*Research completed: 2026-06-17*  
*Ready for planning: yes*
