<!-- GSD:project-start source:PROJECT.md -->

## Project

**Jarvis**

Jarvis is a **local-first personal AI knowledge hub** — a Tauri 2 desktop app for power users who want RAG over their own documents, Feishu/Lark content, and Cursor agent transcripts, with agent-assisted chat, memory, and task extraction. It runs entirely on the user's machine with swappable LLM/embedder providers.

**Current state:** v1.8.0 brownfield — feature-complete through v9 milestones (RAG, Lark sync, Cursor indexing, memory, tasks, insights, multi-agent orchestration, E2E harness). This GSD project targets **v1.9.x structural refactor** to reduce maintenance cost without breaking user-facing behavior.

**Core Value:** **Users can ask questions and run agents against their own indexed knowledge — locally, with citations — and trust that answers come from their data, not the model's training.**

Refactor work must preserve this: every phase keeps E2E green and does not regress retrieval, indexing, or chat quality.

### Constraints

- **Tech stack**: Rust stable (MSRV 1.85), Tauri 2, React 19, SQLite — no stack changes
- **Testing**: TDD required; user-facing changes need E2E updates; CI E2E on Windows
- **Incremental**: Each phase must leave the app shippable; `npm run test:e2e:local` green
- **Store ownership**: Only `crates/store` opens SQLite — preserve single-DB-owner rule
- **Compatibility**: Existing user `config.json` and SQLite DBs must migrate without data loss

<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->

## Technology Stack

## Languages

| Layer | Language | Version / target | Primary locations |
|-------|----------|------------------|-------------------|
| Backend / core logic | **Rust** | Edition **2021**, MSRV **1.85** (`Cargo.toml` workspace) | `src-tauri/`, `crates/*` |
| Frontend UI | **TypeScript** | `~5.8.3` (`package.json`) | `src/` |
| Frontend markup | **TSX (React)** | React **^19.1.0** | `src/App.tsx`, `src/main.tsx` |
| E2E tests | **TypeScript** | WebdriverIO **^9.20.0** | `e2e/specs/*.spec.ts`, `e2e/wdio.conf.ts` |
| Config / manifests | **JSON** | Tauri 2 schema | `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json` |
| Build scripts | **Rust build.rs** | Minimal | `src-tauri/build.rs` |

- `src-tauri` — Tauri shell, IPC commands, scheduler, E2E hooks
- `crates/store` — SQLite + FTS5 + sqlite-vec (sole DB owner)
- `crates/chunker` — text chunking
- `crates/ingest` — file loading (md, txt, csv, xlsx via calamine)
- `crates/embedder` — embedding providers (Mock, Ollama, FastEmbed, OpenAI-compatible)
- `crates/indexer` — document → chunks → vectors pipeline
- `crates/retriever` — hybrid search (vector + FTS5) with RRF fusion
- `crates/llm` — chat providers (Mock, Ollama, OpenAI-compatible)
- `crates/rag` — Q&A with citations
- `crates/config` — `AppConfig` types, provider factory
- `crates/watcher` — folder watch via `notify`
- `crates/lark` — Feishu/Lark via `lark-cli` subprocess
- `crates/cursor` — Cursor agent-transcript JSONL discovery/parse
- `crates/insights` — LLM source summaries + task extraction
- `crates/memory` — chat learning → indexed memories
- `crates/agent` — agent profiles, skills, hooks, plugins, tool loop

## Runtime

| Component | Runtime | Notes |
|-----------|---------|-------|
| Desktop shell | **Tauri 2** (`tauri = "2"`, `tauri-build`, `tauri-cli`) | Native webview; Windows subsystem hides console in release (`src-tauri/src/main.rs`) |
| Rust toolchain | **`stable`** channel | `rust-toolchain.toml` — components: `rustfmt`, `clippy` |
| Async | **Tokio** `1.x` (features: `rt`, `macros`, `sync`, `time`, `fs`) | Used in embedder, llm, watcher, memory, agent tests |
| Dev frontend server | **Vite 7** on port **1420** | `vite.config.ts`; HMR on 1421 when `TAURI_DEV_HOST` set |
| E2E driver | **tauri-driver** + **msedgedriver** | Spawned by `e2e/wdio.conf.ts`; WebDriver port 4444 |
| Node (CI / dev) | **Node 22** | `.github/workflows/ci.yml` |

## Frameworks

### Desktop

- **Tauri 2** — app lifecycle, IPC, events, bundling (`src-tauri/tauri.conf.json`)
- **Tauri plugins:**
- **Capabilities:** `src-tauri/capabilities/default.json` — `core:default`, `opener:default`, `dialog:default`

### Frontend

- **React 19** — single-page app (`src/App.tsx` is the main UI surface)
- **Vite 7** + `@vitejs/plugin-react` — bundling and dev server
- **Tailwind CSS v4** — via `@tailwindcss/vite` plugin; entry `src/index.css` (`@import "tailwindcss"`)
- **Motion** (`motion` ^12) — UI animation (`package.json`)
- **Icons:** `@tabler/icons-react` ^3.44

### Testing frameworks

| Layer | Framework | Config |
|-------|-----------|--------|
| Rust unit/integration | `cargo test` | `#[cfg(test)]` in crates |
| Frontend unit | **Vitest 3** | `vite.config.ts` → `test.include: ["src/**/*.test.ts"]` |
| E2E | **WebdriverIO 9** + Mocha + `@wdio/tauri-service` | `e2e/wdio.conf.ts` |

## Key Dependencies

### Rust workspace (`Cargo.toml` `[workspace.dependencies]`)

| Crate | Version | Role |
|-------|---------|------|
| `rusqlite` | 0.32 (`bundled`, FTS5) | SQLite access in `crates/store` |
| `sqlite-vec` | 0.1 | Vector similarity (`vec0` virtual table) |
| `serde` / `serde_json` | 1 | Config, IPC payloads, Lark CLI JSON |
| `thiserror` | 1 | Domain errors across crates |
| `async-trait` | 0.1 | `Embedder`, `ChatModel` traits |
| `tokio` | 1 | Async runtime |
| `reqwest` | 0.12 (`json`, `rustls-tls`, `stream`) | Ollama + OpenAI-compatible HTTP |
| `futures-util` | 0.3 | LLM streaming |
| `fastembed` | 5 | Local ONNX embeddings (default provider) |
| `calamine` | 0.26 | Spreadsheet ingest (xls/xlsx/ods) |
| `notify` | 7 | Filesystem watcher |
| `sha2` / `hex` | — | Content hashing |

### Tauri app (`src-tauri/Cargo.toml`)

### npm (`package.json`)

### Architectural patterns

- **Trait-based providers:** `Embedder` (`crates/embedder/src/lib.rs`), `ChatModel` (`crates/llm/src/lib.rs`) — swappable Mock / Ollama / Cloud / FastEmbed
- **Provider factory:** `crates/config/src/providers.rs` builds implementations from `AppConfig`
- **Injectable subprocess runner:** `CommandRunner` trait in `crates/lark/src/runner.rs` (production: `ProcessRunner`; tests use mocks)
- **Hybrid retrieval:** FTS5 + sqlite-vec with reciprocal rank fusion (`crates/retriever/src/rrf.rs`, default `rrf_k = 60`)
- **Single DB owner:** Only `crates/store` opens SQLite connections

## Configuration

### Toolchain & manifests

| File | Purpose |
|------|---------|
| `rust-toolchain.toml` | Pin Rust `stable` + clippy/rustfmt |
| `Cargo.toml` | Workspace members, shared deps, MSRV 1.85 |
| `package.json` | npm scripts, frontend deps, app version |
| `tsconfig.json` | Strict TS, ES2020, `react-jsx`, bundler resolution |
| `tsconfig.node.json` | Vite config compilation |
| `vite.config.ts` | React + Tailwind plugins, port 1420, Vitest |
| `index.html` | Vite entry, title "Jarvis · 知识中枢" |

### Tauri

| File | Key settings |
|------|--------------|
| `src-tauri/tauri.conf.json` | `identifier: com.jarvis.app`, dev URL `http://localhost:1420`, `frontendDist: ../dist`, CSP `null` |
| `src-tauri/capabilities/default.json` | Window `main` permissions |
| `src-tauri/build.rs` | `tauri_build::build()` |

### Application config (runtime)

- Provider selection: `embedder` (mock | ollama | fastembed | cloud), `chat` (mock | ollama | cloud)
- Ollama: `ollama_base_url` (default `http://127.0.0.1:11434`), models, dims
- Cloud (OpenAI-compatible): `cloud_base_url`, `cloud_api_key`, embed/chat models
- FastEmbed: `fastembed_model` (default `bge-small-zh-v1.5`), `fastembed_dim`
- Lark: `lark_cli_bin` (default `lark-cli`), `lark_identity` (user | bot | auto)
- Cursor: `cursor_projects_root`
- Watch folders, scheduled sync, agent profiles, skills/hooks/plugins enable lists
- Types: `crates/config/src/types.rs`; load/save: `crates/config/src/file.rs`

### User data directories (created at startup)

| Path | Content |
|------|---------|
| `config.json` | `AppConfig` |
| `kb.sqlite` | Knowledge base DB |
| `skills/` | Agent skill markdown files |
| `hooks/` | Agent hook JSON (seeded with example) |
| `plugins/` | Plugin manifests (`plugin.json` per plugin) |

### npm scripts (`package.json`)

## Platform Requirements

### Development

| Requirement | Detail |
|-------------|--------|
| OS | **Windows** primary (E2E, msedgedriver, `.exe` binary paths in wdio config); Rust/Tauri also builds on Linux (CI rust/frontend jobs) |
| Rust | `stable` toolchain via `rust-toolchain.toml`; MSRV 1.85 |
| Node.js | **22** (CI); npm for frontend + E2E |
| System deps | For `fastembed`: ONNX model download on first use; bundled SQLite via rusqlite |
| Optional external | **Ollama** at `127.0.0.1:11434` when using Ollama providers |
| Optional external | **lark-cli** on PATH when syncing Feishu/Lark content |
| E2E tooling | `cargo install tauri-driver`, `msedgedriver-tool` (CI installs from GitHub) |

### Build outputs

- Frontend: `dist/` (Vite)
- Rust binary: `target/release/tauri-app.exe` (Windows) — referenced by `e2e/wdio.conf.ts` as default `JARVIS_E2E_APP`
- Tauri bundle: icons in `src-tauri/icons/`, targets `all` per `tauri.conf.json`

### Supported ingest formats

- Text: `.txt`, `.md`, `.markdown`, `.csv`
- Spreadsheets: `.xls`, `.xlsx`, `.xlsm`, `.ods` (via `calamine`)
- Lark/Feishu: via `lark-cli` subprocess (not direct HTTP from Jarvis)
- Cursor: `.jsonl` agent transcripts under configured projects root

### Default providers (fresh install)

- Embedder: **FastEmbed** (`bge-small-zh-v1.5`, dim 512)
- Chat: **Mock** (no network)
- Cloud defaults point to OpenAI (`https://api.openai.com/v1`) but `cloud_api_key` is empty until user configures

### Security notes

- CSP is `null` in Tauri config — relies on Tauri IPC boundary, not web CSP hardening
- `cloud_api_key` stored in local `config.json` (user machine only)
- Plugin permission `shell_exec` granted by default in config — plugins run subprocess commands when enabled

<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->

## Conventions

## Overview

| Layer | Location | Role |
|-------|----------|------|
| UI | `src/` | React + TypeScript; `invoke` / `listen` only |
| Shell | `src-tauri/` | Tauri commands, `AppState`, E2E hooks |
| Libraries | `crates/*` | Domain logic, traits, `thiserror` enums |
| Rules | `.cursor/rules/` | E2E, TDD, stack pins, frontend taste |

## Naming Patterns

### Rust

| Kind | Convention | Example |
|------|------------|---------|
| Crates | lowercase, single word | `store`, `embedder`, `rag` |
| Modules / files | `snake_case` | `crates/rag/src/ask.rs` |
| Types / traits | `PascalCase` | `Store`, `Embedder`, `ChatModel` |
| Functions / vars | `snake_case` | `index_path`, `build_embedder` |
| Error enums | `{Domain}Error` | `StoreError`, `IngestError`, `LlmError` |
| Result alias | `pub type Result<T>` per crate | `crates/ingest/src/error.rs` |
| Test modules | `mod tests` under `#[cfg(test)]` | inline in same file |
| Integration tests | `crates/{crate}/tests/*.rs` | `crates/store/tests/persistence.rs` |
| Mock types | `Mock{Trait}` | `MockEmbedder`, `MockChatModel` |
| Fake doubles | `Fake{Trait}` | `FakeRunner` in `crates/lark/src/runner.rs` |
| Config enums | `PascalCase` + serde | `EmbedderProvider`, `ChatProvider` |
| Tauri commands | `snake_case` fn names | `list_sources`, `get_index_status` |

### TypeScript / React

| Kind | Convention | Example |
|------|------------|---------|
| Components | `PascalCase` | `App.tsx` |
| Functions / hooks | `camelCase` | `parseCitations`, `sourceKindLabel` |
| Types / interfaces | `PascalCase` | `Citation` in `src/lib/citations.ts` |
| Lib utilities | `src/lib/*.ts` | pure helpers, colocated `*.test.ts` |
| E2E selectors | `data-testid` kebab-case | `nav-chat`, `ask-submit`, `library-stats` |
| IPC payloads | serde `camelCase` on Rust structs | `#[serde(rename_all = "camelCase")]` on events |

### E2E / env

| Variable | Purpose |
|----------|---------|
| `JARVIS_E2E=1` | Enable mock providers + fixtures |
| `JARVIS_E2E_FIXTURE` | Path to seeded markdown (`e2e/fixtures/sample.md`) |
| `JARVIS_E2E_APP` | Override packaged binary path |

## Code Style

### Rust

- **Toolchain:** `rust-toolchain.toml` → `channel = "stable"`; MSRV `1.85`, edition `2021` (`Cargo.toml` workspace).
- **Formatting:** `rustfmt` component installed; run `cargo fmt` before commit.
- **Linting:** `clippy` component installed; no project-wide `#![deny]` in surveyed files.
- **Async:** `tokio` with `#[tokio::test]` for async tests; `async-trait` on provider traits.
- **Dependencies:** workspace-pinned in root `Cargo.toml`; crate manifests use `{ workspace = true }` or path deps.
- **Serialization:** `serde` + `serde_json`; domain types in `crates/store/src/types.rs`.

### TypeScript

- **Compiler:** `tsconfig.json` — `strict: true`, `noUnusedLocals`, `noUnusedParameters`, `jsx: "react-jsx"`.
- **No ESLint / Prettier** configs in repo; style enforced by TypeScript compiler and convention.
- **Bundler:** Vite 7 + `@vitejs/plugin-react` + Tailwind v4 (`vite.config.ts`).
- **Imports:** ESM (`"type": "module"` in `package.json`); Tauri APIs from `@tauri-apps/api`.

### Frontend UI (`.cursor/rules/frontend-taste.mdc`)

- Dark tech + liquid-glass panels; Tabler icons (`@tabler/icons-react`).
- Motion via `motion/react`; honor `prefers-reduced-motion` / `prefers-reduced-transparency`.
- Every interactive surface needs loading, empty, and error states.

## Import Organization

### Rust crate `lib.rs` pattern

- **Private modules** (`mod schema;`) stay internal; **re-export** stable API from `lib.rs`.
- **Cross-crate:** depend on crate names (`use store::Store;`), not `src-tauri` from libraries.
- **`src-tauri`:** imports many crates + thin command wrappers (`src-tauri/src/lib.rs`).

### TypeScript

- React / hooks first, then `@tauri-apps/*`, then local `src/lib/*`.
- Vitest tests: `import { describe, expect, it } from "vitest"`.
- E2E specs: `import { askQuestion } from "../helpers.ts"` (explicit `.ts` extension).

## Error Handling

### Rust — `thiserror` per crate

#[derive(Debug, Error)]

- Use `#[from]` for transparent wrapping of lower-level errors.
- Prefer typed errors inside crates; **map to `String` at the Tauri boundary**:

#[tauri::command]

- Degrade per-source on ingest/index failures; do not panic on user data paths.
- E2E helpers (`src-tauri/src/e2e.rs`) also return `Result<_, String>` for startup seeding.

### TypeScript / frontend

- **Defensive parsing:** return empty defaults on bad input (`src/lib/citations.ts` — `parseCitations` catches JSON errors → `[]`).
- **IPC errors:** surfaced in UI via state + `data-testid="chat-error"`; E2E asserts on this selector (`e2e/helpers.ts`).
- **No shared error-type package** between Rust and TS; shapes follow serde-serialized structs from commands.

## Logging

- **No `tracing` / `log` crate usage** in surveyed application code.
- Diagnostics rely on:
- Tauri event emission for progress (`emit_index_progress` in `src-tauri/src/lib.rs`) rather than log lines.

## Comments

- **Sparse by design** (`karpathy-guidelines.mdc`): comment non-obvious business rules only.
- **Doc comments** on public traits and E2E entry points (`/// Deterministic embedder for tests` in `crates/embedder/src/mock.rs`).
- **E2E / spec headers:** file-level `/** … */` in `e2e/specs/*.spec.ts` describing journey scope.
- **Ignored live tests:** `#[ignore = "requires live lark-cli auth"]` in `crates/lark/src/sync.rs`.
- **Cursor rules** (`.cursor/rules/*.mdc`) carry policy, not inline code comments.

## Function / Module Design

### Architecture principles (`AGENTS.md`)

### Trait injection pattern

#[async_trait]

### `AppState` (`src-tauri/src/lib.rs`)

- Holds `Arc<Store>`, `Mutex<Arc<dyn Embedder>>`, `Mutex<Arc<dyn ChatModel>>`, configs, watcher/scheduler handles.
- E2E mode: `apply_e2e_config` forces `Mock` providers (`src-tauri/src/e2e.rs`).

### Module boundaries

| Crate | Responsibility |
|-------|------------------|
| `chunker` | Text splitting (pure, unit-tested) |
| `ingest` | Load files → `Document` |
| `indexer` | Chunk + embed + store |
| `retriever` | FTS + vector + RRF |
| `rag` | `ask` / `ask_stream` orchestration |
| `agent` | Tool loop, orchestration, plugins |
| `lark` | `lark-cli` via `CommandRunner` |
| `memory` | Chat learning, memory CRUD |
| `config` | Load/save `AppConfig`, provider factories |

### Frontend design

- **Single main component:** `src/App.tsx` — navigation, chat, library, settings, memory, agent UI.
- **Thin IPC:** `invoke("command_name", { args })`, `listen` for `index-progress` and streaming tokens.
- **Pure logic extracted** to `src/lib/` for Vitest coverage (`citations.ts`, `sourceDisplay.ts`).

### TDD workflow (`.cursor/rules/tdd-goal-driven.mdc`)

## Key Reference Files

| Topic | Path |
|-------|------|
| Agent / stack policy | `AGENTS.md` |
| Workspace deps / MSRV | `Cargo.toml` |
| Toolchain | `rust-toolchain.toml` |
| TS strict config | `tsconfig.json` |
| Vite + Vitest | `vite.config.ts` |
| Provider factories | `crates/config/src/providers.rs` |
| E2E mocks | `src-tauri/src/e2e.rs` |
| Tauri commands | `src-tauri/src/lib.rs` |
| Frontend taste rule | `.cursor/rules/frontend-taste.mdc` |
| E2E policy | `.cursor/rules/e2e-required.mdc` |
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->

## Architecture

## System Overview

```

```

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

## Layers

```

```

## Data Flow

### Index Pipeline

```

```

### Q&A (RAG)

```

```

### Agent Mode

```

```

## Key Abstractions

### `Embedder` (`crates/embedder/src/lib.rs`)

```rust
#[async_trait]
```

### `ChatModel` (`crates/llm/src/lib.rs`)

```rust
#[async_trait]
```

### `Store` (`crates/store/src/store.rs`)

- **Only** module that opens `rusqlite::Connection`.
- Tables: `sources`, `chunks`, vector virtual table, FTS5, `chat_sessions`, `chat_messages`, `tasks`, `meta`.
- Search: `search_vector`, `search_fts`; hybrid fusion happens in `retriever`, not store.
- Thread-safe via `Mutex<Connection>`.

### `CommandRunner` (`crates/lark/src/runner.rs`)

### `AppState` (`src-tauri/src/lib.rs`)

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

## Architectural Constraints

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

## Related Documents

- Product design: `docs/superpowers/specs/2026-06-12-personal-ai-knowledge-hub-design.md`
- Agent onboarding: `AGENTS.md`
- Directory reference: `.planning/codebase/STRUCTURE.md`

<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->

## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, `.github/skills/`, or `.codex/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->

## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:

- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->

<!-- GSD:profile-start -->

## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
