# Jarvis Architecture

> Generated: 2026-06-17  
> Version: 1.8.0 (workspace `Cargo.toml`)  
> Stack: Tauri 2 + React 19 + Rust workspace crates

Jarvis is a **local-first personal AI knowledge hub** — a Tauri 2 desktop app that indexes documents (local files, Feishu/Lark content, Cursor agent transcripts, user memories), stores chunked embeddings in SQLite, and answers questions via hybrid RAG or tool-augmented agent chat.

---

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Desktop (Tauri 2 WebView)                           │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  src/  React UI                                                       │  │
│  │  App.tsx — Chat · Library · Tasks · Memory · Settings                 │  │
│  │  invoke() / Channel / listen()  ←→  Tauri IPC                         │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  src-tauri/  Tauri shell (thin orchestration layer)                         │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────┐  ┌──────────────┐  │
│  │ lib.rs      │  │ index_ops.rs │  │ sync_scheduler │  │ insights_ops │  │
│  │ AppState    │  │ rebuild/sync │  │ periodic sync  │  │ auto-insights│  │
│  │ IPC cmds    │  │ progress evt │  │ watch+lark+cur │  │ on index     │  │
│  └─────────────┘  └──────────────┘  └────────────────┘  └──────────────┘  │
│  ┌─────────────┐                                                            │
│  │ e2e.rs      │  JARVIS_E2E=1 mocks, fixtures, deterministic CI             │
│  └─────────────┘                                                            │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
          ┌───────────────────────────┼───────────────────────────┐
          ▼                           ▼                           ▼
┌──────────────────┐      ┌──────────────────┐      ┌──────────────────────┐
│  Index pipeline  │      │  Q&A / RAG       │      │  Agent loop          │
│  ingest→chunker  │      │  retriever→rag   │      │  agent (tools/skills)│
│  →embedder       │      │  →llm            │      │  →orchestrate/router │
│  →indexer→store  │      │                  │      │                      │
└──────────────────┘      └──────────────────┘      └──────────────────────┘
          │                           │                           │
          └───────────────────────────┼───────────────────────────┘
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  crates/store  — sole SQLite owner (FTS5 + sqlite-vec)                        │
│  sources · chunks · vectors · chat_sessions · tasks · meta                  │
└─────────────────────────────────────────────────────────────────────────────┘
                                      ▲
          ┌───────────────────────────┼───────────────────────────┐
          ▼                           ▼                           ▼
┌──────────────────┐      ┌──────────────────┐      ┌──────────────────────┐
│  watcher         │      │  lark            │      │  cursor              │
│  folder scan     │      │  lark-cli subproc│      │  agent-transcripts   │
│  notify events   │      │  doc/sheet/mail  │      │  JSONL parse         │
└──────────────────┘      └──────────────────┘      └──────────────────────┘
```

**External dependencies (runtime):** Ollama, OpenAI-compatible APIs, FastEmbed ONNX models, `lark-cli` subprocess for Feishu.

---

## Component Responsibilities

| Component | Path | Responsibility |
|-----------|------|----------------|
| React UI | `src/App.tsx`, `src/lib/*` | Views (chat, library, tasks, memory, settings); IPC via `invoke` / `Channel`; event listeners for index progress |
| Tauri shell | `src-tauri/src/lib.rs` | `AppState`, provider wiring, all `#[tauri::command]` handlers, startup (watcher, scheduler, initial scan) |
| Index orchestration | `src-tauri/src/index_ops.rs` | Rebuild, retry, local batch index, Lark/Cursor sync helpers, `IndexProgressEvent` emission |
| Sync scheduler | `src-tauri/src/sync_scheduler.rs` | Background thread: periodic watch-folder rescan, Lark re-fetch, Cursor transcript sync |
| Insights glue | `src-tauri/src/insights_ops.rs` | Post-index summarize + task extraction hooks |
| E2E harness | `src-tauri/src/e2e.rs` | `JARVIS_E2E=1` config overrides, fixture seeding, mock Lark auth |
| Config | `crates/config/` | `AppConfig` persistence, `build_embedder` / `build_chat_model` factory |
| Ingest | `crates/ingest/` | Load files → `Document` (text, hash, uri, title) |
| Chunker | `crates/chunker/` | Split text into overlapping chunks with line locations |
| Embedder | `crates/embedder/` | `Embedder` trait; Mock, Ollama, FastEmbed, OpenAI implementations |
| Indexer | `crates/indexer/` | `index_document` / `index_path`; chunk + embed + store; embed cache |
| Store | `crates/store/` | SQLite schema, FTS5 keyword search, sqlite-vec vector search, CRUD for all entities |
| Retriever | `crates/retriever/` | Hybrid search: vector + FTS → RRF fusion |
| LLM | `crates/llm/` | `ChatModel` trait; Mock, Ollama, OpenAI; streaming support |
| RAG | `crates/rag/` | `ask` / `ask_stream`: retrieve → prompt → complete |
| Agent | `crates/agent/` | Tool loop, skills, hooks, plugins, pipeline/router orchestration |
| Memory | `crates/memory/` | Long-term memory CRUD + `learn_from_exchange` |
| Insights | `crates/insights/` | LLM summarize source, extract tasks |
| Lark | `crates/lark/` | `CommandRunner` abstraction; `lark-cli` fetch + auth health |
| Cursor | `crates/cursor/` | Discover/parse Cursor `agent-transcripts` JSONL |
| Watcher | `crates/watcher/` | `notify`-based folder watch, `scan_folder` |

---

## Pattern Overview

| Pattern | Where | Purpose |
|---------|-------|---------|
| **Trait-based providers** | `Embedder`, `ChatModel`, `CommandRunner` | Swappable backends; mocks in tests and E2E |
| **Thin shell, fat crates** | `src-tauri` delegates to `crates/*` | Business logic testable without Tauri |
| **Single DB owner** | `crates/store` only opens SQLite | No connection races; one schema authority |
| **Normalize-to-Document** | ingest, lark, cursor, memory | All sources share `indexer::index_document` pipeline |
| **RRF hybrid retrieval** | `retriever` | Combines semantic (vector) + lexical (FTS5) without score calibration |
| **Text-based agent tools** | `agent::run` | LLM emits JSON tool calls parsed from reply text (not native function-calling API) |
| **Event-driven progress** | Tauri `Emitter` + frontend `listen` | Index/rebuild UX without polling |
| **Deterministic E2E** | `e2e.rs` + `JARVIS_E2E=1` | CI runs full UI journeys with Mock providers |

---

## Layers

```
┌─────────────────────────────────────────┐
│  Presentation  src/ (React + Tailwind)  │
├─────────────────────────────────────────┤
│  IPC / Shell   src-tauri/ (commands)    │
├─────────────────────────────────────────┤
│  Application   index_ops, sync_scheduler│
│                insights_ops, e2e        │
├─────────────────────────────────────────┤
│  Domain        crates: rag, agent,      │
│                indexer, retriever, …    │
├─────────────────────────────────────────┤
│  Infrastructure embedder, llm, lark,    │
│                watcher, config          │
├─────────────────────────────────────────┤
│  Persistence   crates/store (SQLite)    │
└─────────────────────────────────────────┘
```

**Dependency rule:** Domain crates depend on `store` and traits; they do **not** depend on Tauri or React. `src-tauri` is the composition root.

---

## Data Flow

### Index Pipeline

```
Source (file / lark-cli / cursor JSONL / memory text)
    │
    ▼
ingest::load_path  OR  lark::fetch_*  OR  cursor::load_transcript  OR  memory
    │  produces ingest::Document { uri, title, text, content_hash }
    ▼
indexer::index_document(store, embedder, chunker, doc, SourceKind)
    │
    ├─ store.upsert_source (status: Pending)
    ├─ chunker::chunk_text → Vec<ChunkDraft>
    ├─ embedder::embed (with embed cache in store meta)
    ├─ store.insert_chunks (text + vectors + FTS index)
    └─ store.upsert_source (status: Indexed | Failed)
    │
    ▼ (optional, if auto_* flags in AppConfig)
insights_ops::maybe_run_insights_for_source
    ├─ insights::summarize_source → store.update_source_summary
    └─ insights::extract_tasks_from_source → store.insert_task
```

**Triggers:** manual file pick (`index_file`), watch-folder events (`watcher`), initial startup scan, rebuild (`rebuild_index`), Lark sync commands, Cursor sync, scheduled sync, memory add/update.

### Q&A (RAG)

```
User question (Chat view, session-bound)
    │
    ▼
src-tauri: ask_in_session / ask_in_session_stream
    │
    ▼
rag::ask / ask_stream(store, embedder, chat, retriever, question)
    │
    ├─ retriever::retrieve
    │     ├─ embedder.embed(query) → store.search_vector
    │     ├─ store.search_fts (built FTS query)
    │     └─ reciprocal_rank_fusion → top-k ChunkHit
    ├─ build_messages(question, hits, citations)
    ├─ chat.complete / complete_stream
    └─ AskResponse { answer, citations }
    │
    ▼
store.append_chat_message (user + assistant + citations_json)
    │
    ▼ (optional, auto_learn_from_chat)
memory::learn_from_exchange
```

Streaming uses Tauri `Channel<TokenEvent>` from `ask_in_session_stream`; UI accumulates tokens via `Channel` callback.

### Agent Mode

```
User question + active AgentProfile (from AppConfig.agents)
    │
    ▼
src-tauri: ask_agent_in_session / ask_agent_in_session_stream
    │
    ├─ AgentOrchestrationMode::Single  → agent::run_agent
    ├─ Pipeline                        → agent::run_orchestrated
    └─ Router                          → agent::run_routed
    │
    ▼
agent::run_agent(ctx, chat, embedder, profile, skills, …)
    │
    ├─ System prompt = profile.system_prompt + tools prompt + enabled skills
    ├─ Loop (max 3 rounds):
    │     chat.complete → parse_tool_call?
    │       ├─ yes → execute_tool (search_knowledge, list_sources, …)
    │       │         hooks::run_hooks (BeforeToolCall / AfterToolCall)
    │       └─ no  → final answer
    └─ AgentResponse { answer, citations, tool_calls, orchestration_steps }
```

Per-agent overrides: `ChatResolver` / `EmbedResolver` in `crates/agent` allow profile-level `chat_provider` / `embedder_provider`.

---

## Key Abstractions

### `Embedder` (`crates/embedder/src/lib.rs`)

```rust
#[async_trait]
pub trait Embedder: Send + Sync {
    fn id(&self) -> &str;
    fn dim(&self) -> usize;
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}
```

Built via `config::build_embedder` / `build_embedder_for_profile`. Store vector dimension must match embedder dim; mismatch triggers `needs_rebuild` in index health.

### `ChatModel` (`crates/llm/src/lib.rs`)

```rust
#[async_trait]
pub trait ChatModel: Send + Sync {
    fn id(&self) -> &str;
    async fn complete(&self, messages: &[Message]) -> Result<String>;
    async fn complete_stream(&self, messages: &[Message], on_token: &mut …) -> Result<String>;
}
```

Used by RAG, insights, memory learning, and agent loop.

### `Store` (`crates/store/src/store.rs`)

- **Only** module that opens `rusqlite::Connection`.
- Tables: `sources`, `chunks`, vector virtual table, FTS5, `chat_sessions`, `chat_messages`, `tasks`, `meta`.
- Search: `search_vector`, `search_fts`; hybrid fusion happens in `retriever`, not store.
- Thread-safe via `Mutex<Connection>`.

### `CommandRunner` (`crates/lark/src/runner.rs`)

Injectable subprocess runner for `lark-cli`; `FakeRunner` in tests, `ProcessRunner` in production.

### `AppState` (`src-tauri/src/lib.rs`)

Central runtime context: `Arc<Store>`, `Mutex<Arc<dyn Embedder>>`, `Mutex<Arc<dyn ChatModel>>`, configs, watcher/scheduler handles, paths to skills/hooks/plugins dirs.

---

## Entry Points

| Entry | Path | Role |
|-------|------|------|
| Desktop binary | `src-tauri/src/main.rs` | Calls `tauri_app_lib::run()` |
| Tauri bootstrap | `src-tauri/src/lib.rs::run()` | `setup`: `init_state`, E2E seed, initial scan, watcher, scheduler; registers ~50 commands |
| Web UI bootstrap | `src/main.tsx` | Renders `<App />` |
| UI root | `src/App.tsx` | All views and IPC; ~2800 lines monolith |
| Workspace manifest | `Cargo.toml` | 15 workspace members + shared deps |
| Frontend build | `vite.config.ts`, `package.json` | Vite + Tauri integration |
| E2E runner | `e2e/wdio.conf.ts` | WebdriverIO + `tauri-driver` |

**Startup sequence (`lib.rs::setup`):**

1. Resolve app data dir → `config.json`, `jarvis.db`, `skills/`, `hooks/`, `plugins/`
2. Load/apply E2E config if `JARVIS_E2E=1`
3. `Store::open` with embedding dimension from config
4. Build embedder + chat model
5. Seed E2E fixture OR `initial_scan_with_progress` on watch folders
6. `restart_watcher`, `restart_scheduler`
7. `app.manage(AppState)`

---

## Architectural Constraints

1. **Store exclusivity** — Only `crates/store` opens SQLite. No other crate may hold a `Connection`.
2. **Thin Tauri commands** — Commands validate inputs, read `AppState`, delegate to crate functions; no embedded business rules.
3. **Provider injection** — External I/O (embed, chat, lark-cli) goes through traits for testability.
4. **Source normalization** — Every indexable item becomes `ingest::Document` + `SourceKind` before `index_document`.
5. **Dimension consistency** — `embedder.dim()` must equal `store.dim()`; config changes may require `reinit_and_rebuild_index`.
6. **E2E determinism** — CI sets `JARVIS_E2E=1`; no live LLM or Feishu in automated tests.
7. **Local-first** — Primary data in SQLite under app data dir; cloud APIs optional via config.
8. **MSRV 1.85** — Workspace `rust-version` in root `Cargo.toml`; edition 2021.

---

## Anti-Patterns

| Anti-pattern | Why it fails | Do instead |
|--------------|--------------|------------|
| Open SQLite outside `store` | Schema drift, lock contention, untestable | Add methods to `Store` |
| Put RAG/agent logic in `App.tsx` | Untestable, duplicates backend | `invoke` to Tauri command → crate |
| Call live LLM/Feishu in unit tests | Flaky CI, cost, non-determinism | `MockEmbedder`, `MockChatModel`, `FakeRunner`, `JARVIS_E2E=1` |
| Skip E2E for user-facing features | Regressions ship silently | Add spec under `e2e/specs/` per `.cursor/rules/e2e-required.mdc` |
| Hardcode provider choice in crates | Breaks per-agent overrides | Use `config::build_*` or `EmbedResolver` / `ChatResolver` |
| Index without content_hash check | Wasted embed work | `indexer` already skips unchanged indexed sources |
| Native tool-calling API assumption | Agent uses text JSON protocol | Follow `agent::parse_tool_call` format |
| Multiple tokio runtimes per operation | Complexity; current code spawns runtimes in threads | Match existing pattern in watcher/scheduler threads |
| Store secrets in repo | Security incident | `cloud_api_key` in local `config.json` only |
| Fat Tauri commands (100+ lines) | Hard to test | Extract to `index_ops`, `insights_ops`, or domain crate |

---

## Related Documents

- Product design: `docs/superpowers/specs/2026-06-12-personal-ai-knowledge-hub-design.md`
- Agent onboarding: `AGENTS.md`
- Directory reference: `.planning/codebase/STRUCTURE.md`
