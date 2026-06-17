# Jarvis Code Conventions

> Mapped: 2026-06-17  
> Sources: `AGENTS.md`, `.cursor/rules/`, workspace manifests, and representative source files.

---

## Overview

Jarvis is a Tauri 2 desktop app with a **thin React frontend** and **business logic in Rust workspace crates**. Conventions favor testability (trait injection), surgical diffs, and a single SQLite owner (`crates/store`).

| Layer | Location | Role |
|-------|----------|------|
| UI | `src/` | React + TypeScript; `invoke` / `listen` only |
| Shell | `src-tauri/` | Tauri commands, `AppState`, E2E hooks |
| Libraries | `crates/*` | Domain logic, traits, `thiserror` enums |
| Rules | `.cursor/rules/` | E2E, TDD, stack pins, frontend taste |

---

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

---

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

---

## Import Organization

### Rust crate `lib.rs` pattern

```rust
// crates/embedder/src/lib.rs
mod error;
mod mock;
// … impl modules

pub use error::{EmbedError, Result};
pub use mock::MockEmbedder;
// … public re-exports

use async_trait::async_trait;
// trait definition
```

- **Private modules** (`mod schema;`) stay internal; **re-export** stable API from `lib.rs`.
- **Cross-crate:** depend on crate names (`use store::Store;`), not `src-tauri` from libraries.
- **`src-tauri`:** imports many crates + thin command wrappers (`src-tauri/src/lib.rs`).

### TypeScript

- React / hooks first, then `@tauri-apps/*`, then local `src/lib/*`.
- Vitest tests: `import { describe, expect, it } from "vitest"`.
- E2E specs: `import { askQuestion } from "../helpers.ts"` (explicit `.ts` extension).

---

## Error Handling

### Rust — `thiserror` per crate

Each library defines a domain error enum and `Result<T>` alias:

```rust
// crates/store/src/error.rs
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("dimension mismatch: expected {expected}, got {got}")]
    DimMismatch { expected: usize, got: usize },
}
pub type Result<T> = std::result::Result<T, StoreError>;
```

Same pattern in `crates/ingest/src/error.rs`, `crates/memory/src/error.rs`, `crates/agent/src/error.rs`, `crates/llm/src/error.rs`, etc.

**Rules:**

- Use `#[from]` for transparent wrapping of lower-level errors.
- Prefer typed errors inside crates; **map to `String` at the Tauri boundary**:

```rust
// src-tauri/src/lib.rs
#[tauri::command]
fn list_sources(state: tauri::State<'_, AppState>) -> Result<Vec<Source>, String> {
    state.store.list_sources().map_err(|e| e.to_string())
}
```

- Degrade per-source on ingest/index failures; do not panic on user data paths.
- E2E helpers (`src-tauri/src/e2e.rs`) also return `Result<_, String>` for startup seeding.

### TypeScript / frontend

- **Defensive parsing:** return empty defaults on bad input (`src/lib/citations.ts` — `parseCitations` catches JSON errors → `[]`).
- **IPC errors:** surfaced in UI via state + `data-testid="chat-error"`; E2E asserts on this selector (`e2e/helpers.ts`).
- **No shared error-type package** between Rust and TS; shapes follow serde-serialized structs from commands.

---

## Logging

- **No `tracing` / `log` crate usage** in surveyed application code.
- Diagnostics rely on:
  - Returned error strings to the UI
  - Test assertions
  - CI stdout (`reporters: ["spec"]` in `e2e/wdio.conf.ts`)
- Tauri event emission for progress (`emit_index_progress` in `src-tauri/src/lib.rs`) rather than log lines.

When adding observability, prefer existing event channels before introducing a new logging stack.

---

## Comments

- **Sparse by design** (`karpathy-guidelines.mdc`): comment non-obvious business rules only.
- **Doc comments** on public traits and E2E entry points (`/// Deterministic embedder for tests` in `crates/embedder/src/mock.rs`).
- **E2E / spec headers:** file-level `/** … */` in `e2e/specs/*.spec.ts` describing journey scope.
- **Ignored live tests:** `#[ignore = "requires live lark-cli auth"]` in `crates/lark/src/sync.rs`.
- **Cursor rules** (`.cursor/rules/*.mdc`) carry policy, not inline code comments.

---

## Function / Module Design

### Architecture principles (`AGENTS.md`)

1. **`store` is the sole SQLite owner** — no other crate opens `rusqlite` connections.
2. **Narrow traits, swappable providers** — `Embedder`, `ChatModel`, `CommandRunner`.
3. **Tauri commands stay thin** — delegate to `crates/*` functions.
4. **Feishu / Lark content → `Document`** — same indexer pipeline as local files.

### Trait injection pattern

```rust
// crates/embedder/src/lib.rs
#[async_trait]
pub trait Embedder: Send + Sync {
    fn id(&self) -> &str;
    fn dim(&self) -> usize;
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}
```

Built at runtime via `crates/config/src/providers.rs` → `build_embedder`, `build_chat_model`, with profile overrides.

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

1. State goal + acceptance checks.
2. Red → Green → Refactor.
3. User-facing work requires E2E before merge.

---

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
