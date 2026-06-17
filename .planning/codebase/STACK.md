# Technology Stack

**Project:** Jarvis — Personal AI knowledge hub (Tauri 2 desktop app)  
**Version:** 1.8.0 (aligned across `package.json`, `Cargo.toml` workspace, `src-tauri/tauri.conf.json`)  
**Mapped:** 2026-06-17  
**Focus:** tech

---

## Languages

| Layer | Language | Version / target | Primary locations |
|-------|----------|------------------|-------------------|
| Backend / core logic | **Rust** | Edition **2021**, MSRV **1.85** (`Cargo.toml` workspace) | `src-tauri/`, `crates/*` |
| Frontend UI | **TypeScript** | `~5.8.3` (`package.json`) | `src/` |
| Frontend markup | **TSX (React)** | React **^19.1.0** | `src/App.tsx`, `src/main.tsx` |
| E2E tests | **TypeScript** | WebdriverIO **^9.20.0** | `e2e/specs/*.spec.ts`, `e2e/wdio.conf.ts` |
| Config / manifests | **JSON** | Tauri 2 schema | `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json` |
| Build scripts | **Rust build.rs** | Minimal | `src-tauri/build.rs` |

**Rust workspace members** (`Cargo.toml`):

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

---

## Runtime

| Component | Runtime | Notes |
|-----------|---------|-------|
| Desktop shell | **Tauri 2** (`tauri = "2"`, `tauri-build`, `tauri-cli`) | Native webview; Windows subsystem hides console in release (`src-tauri/src/main.rs`) |
| Rust toolchain | **`stable`** channel | `rust-toolchain.toml` — components: `rustfmt`, `clippy` |
| Async | **Tokio** `1.x` (features: `rt`, `macros`, `sync`, `time`, `fs`) | Used in embedder, llm, watcher, memory, agent tests |
| Dev frontend server | **Vite 7** on port **1420** | `vite.config.ts`; HMR on 1421 when `TAURI_DEV_HOST` set |
| E2E driver | **tauri-driver** + **msedgedriver** | Spawned by `e2e/wdio.conf.ts`; WebDriver port 4444 |
| Node (CI / dev) | **Node 22** | `.github/workflows/ci.yml` |

**Process model:** Single desktop process. Rust owns SQLite, indexing, RAG, and subprocess calls (`lark-cli`, agent hooks/plugins). React UI is thin — invokes Tauri commands via `@tauri-apps/api`.

---

## Frameworks

### Desktop

- **Tauri 2** — app lifecycle, IPC, events, bundling (`src-tauri/tauri.conf.json`)
- **Tauri plugins:**
  - `tauri-plugin-opener` — open URLs/files externally
  - `tauri-plugin-dialog` — native file/folder dialogs
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

---

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

Depends on all workspace library crates plus `tauri`, `tauri-plugin-opener`, `tauri-plugin-dialog`, `serde`, `serde_json`, `tokio`. Dev: `tempfile`.

### npm (`package.json`)

**Runtime:** `@tauri-apps/api` ^2, `@tauri-apps/plugin-dialog`, `@tauri-apps/plugin-opener`, `react`, `react-dom`, `motion`, `tailwindcss`, `@tailwindcss/vite`, `@tabler/icons-react`.

**Dev:** `@tauri-apps/cli` ^2, `typescript`, `vite`, `vitest`, `@vitejs/plugin-react`, WebdriverIO stack, `cross-env`.

### Architectural patterns

- **Trait-based providers:** `Embedder` (`crates/embedder/src/lib.rs`), `ChatModel` (`crates/llm/src/lib.rs`) — swappable Mock / Ollama / Cloud / FastEmbed
- **Provider factory:** `crates/config/src/providers.rs` builds implementations from `AppConfig`
- **Injectable subprocess runner:** `CommandRunner` trait in `crates/lark/src/runner.rs` (production: `ProcessRunner`; tests use mocks)
- **Hybrid retrieval:** FTS5 + sqlite-vec with reciprocal rank fusion (`crates/retriever/src/rrf.rs`, default `rrf_k = 60`)
- **Single DB owner:** Only `crates/store` opens SQLite connections

---

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

Persisted JSON at **`{app_data_dir}/config.json`** (`src-tauri/src/lib.rs`):

- Provider selection: `embedder` (mock | ollama | fastembed | cloud), `chat` (mock | ollama | cloud)
- Ollama: `ollama_base_url` (default `http://127.0.0.1:11434`), models, dims
- Cloud (OpenAI-compatible): `cloud_base_url`, `cloud_api_key`, embed/chat models
- FastEmbed: `fastembed_model` (default `bge-small-zh-v1.5`), `fastembed_dim`
- Lark: `lark_cli_bin` (default `lark-cli`), `lark_identity` (user | bot | auto)
- Cursor: `cursor_projects_root`
- Watch folders, scheduled sync, agent profiles, skills/hooks/plugins enable lists
- Types: `crates/config/src/types.rs`; load/save: `crates/config/src/file.rs`

### User data directories (created at startup)

Under Tauri `app_data_dir()` (or `jarvis-e2e/` when `JARVIS_E2E=1`):

| Path | Content |
|------|---------|
| `config.json` | `AppConfig` |
| `kb.sqlite` | Knowledge base DB |
| `skills/` | Agent skill markdown files |
| `hooks/` | Agent hook JSON (seeded with example) |
| `plugins/` | Plugin manifests (`plugin.json` per plugin) |

### npm scripts (`package.json`)

```bash
npm run dev              # Vite only
npm run build            # tsc && vite build
npm run tauri dev        # Desktop dev (via tauri CLI)
npm run tauri build      # Release bundle
npm test                 # Vitest
npm run test:e2e         # WebdriverIO (all specs)
npm run test:e2e:local   # E2E with JARVIS_E2E=1 + fixture
```

---

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

Defined in `crates/ingest/src/loader.rs`:

- Text: `.txt`, `.md`, `.markdown`, `.csv`
- Spreadsheets: `.xls`, `.xlsx`, `.xlsm`, `.ods` (via `calamine`)
- Lark/Feishu: via `lark-cli` subprocess (not direct HTTP from Jarvis)
- Cursor: `.jsonl` agent transcripts under configured projects root

### Default providers (fresh install)

From `AppConfig::default()` (`crates/config/src/types.rs`):

- Embedder: **FastEmbed** (`bge-small-zh-v1.5`, dim 512)
- Chat: **Mock** (no network)
- Cloud defaults point to OpenAI (`https://api.openai.com/v1`) but `cloud_api_key` is empty until user configures

### Security notes

- CSP is `null` in Tauri config — relies on Tauri IPC boundary, not web CSP hardening
- `cloud_api_key` stored in local `config.json` (user machine only)
- Plugin permission `shell_exec` granted by default in config — plugins run subprocess commands when enabled

---

*Stack map generated 2026-06-17. See `INTEGRATIONS.md` for external services and storage detail.*
