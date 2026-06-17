# External Integrations

**Project:** Jarvis — Personal AI knowledge hub  
**Version:** 1.8.0  
**Mapped:** 2026-06-17  
**Focus:** tech

---

## APIs & External Services

Jarvis is **local-first**. Network calls are optional and provider-specific. Feishu/Lark is integrated **indirectly** via the `lark-cli` subprocess, not via embedded Feishu SDK.

### LLM chat providers (`crates/llm`)

| Provider | Config key | Endpoint | Client | File |
|----------|------------|----------|--------|------|
| **Mock** | `chat: "mock"` | None | In-process | `crates/llm/src/mock.rs` |
| **Ollama** | `chat: "ollama"` | `{ollama_base_url}/api/chat` | `reqwest` | `crates/llm/src/ollama.rs` |
| **Cloud (OpenAI-compatible)** | `chat: "cloud"` | `{cloud_base_url}/chat/completions` | `reqwest` + Bearer token | `crates/llm/src/openai.rs` |

**Defaults** (`crates/config/src/types.rs`):

- Ollama base: `http://127.0.0.1:11434`, model `llama3.2`
- Cloud base: `https://api.openai.com/v1`, model `gpt-4o-mini`
- Per-agent override: `AgentProfile.chat_provider` resolved in `crates/config/src/providers.rs`

**Streaming:** `complete_stream` uses SSE/chunk parsing (`futures-util::StreamExt`) for Ollama and OpenAI-compatible APIs. UI receives tokens via Tauri `Channel` (`ask_in_session_stream`, `ask_agent_in_session_stream` in `src-tauri/src/lib.rs`).

### Embedding providers (`crates/embedder`)

| Provider | Config key | Endpoint / runtime | File |
|----------|------------|-------------------|------|
| **Mock** | `embedder: "mock"` | Deterministic vectors | `crates/embedder/src/mock.rs` |
| **Ollama** | `embedder: "ollama"` | `POST {base}/api/embeddings` | `crates/embedder/src/ollama.rs` |
| **FastEmbed** | `embedder: "fastembed"` | Local ONNX (`fastembed` crate); models downloaded on first use | `crates/embedder/src/fastembed.rs` |
| **Cloud** | `embedder: "cloud"` | `POST {cloud_base_url}/embeddings` | `crates/embedder/src/openai.rs` |

**Defaults:** FastEmbed model `bge-small-zh-v1.5` (512-dim); cloud embed model `text-embedding-3-small` (1536-dim).

**Embed cache:** `embed_cache` table in SQLite stores `text_hash → vector` to avoid re-embedding identical text (`crates/store/src/schema.rs`).

### Feishu / Lark (via `lark-cli`)

Integration is **subprocess-based** through injectable `CommandRunner` (`crates/lark/src/runner.rs`).

| Operation | lark-cli invocation | Jarvis command | Source kind |
|-----------|---------------------|----------------|-------------|
| Auth status | `lark-cli auth status` | `check_lark_connection` | — |
| Docs | `lark-cli docs +fetch ...` | `sync_lark_doc` | `lark_doc` |
| Sheets | `lark-cli sheets +read ...` | `sync_lark_sheet` | `lark_sheet` |
| Mail | `lark-cli mail +message ...` | `sync_lark_mail` | `lark_mail` |
| IM chat | `lark-cli im ...` | `sync_lark_im` | `lark_msg` |
| URL resolve | `lark-cli drive +inspect ...` then typed fetch | `sync_lark_url` | doc/sheet/file/wiki |

Implementation: `crates/lark/src/sync.rs`, `crates/lark/src/health.rs`, `crates/lark/src/parse.rs`.

**Config:**

- `lark_cli_bin` — default `"lark-cli"` (auto-detect: `detect_lark_cli` in `crates/lark`)
- `lark_identity` — `user` | `bot` | `auto` (default `user`)

**Auth model:** Tokens and app credentials live in **lark-cli's own config**, not in Jarvis. Jarvis only checks status via CLI JSON output. User hint when unauthenticated: run `lark-cli config init` and `lark-cli auth login` (`crates/lark/src/health.rs`).

**E2E:** No live Lark in CI — `src-tauri/src/e2e.rs` mocks auth and fixture documents when `JARVIS_E2E=1`.

### Cursor agent transcripts (local filesystem)

No network API. Reads JSONL files from Cursor projects directory.

| Setting | Purpose |
|---------|---------|
| `cursor_projects_root` | e.g. `%USERPROFILE%\.cursor\projects` |

Crate: `crates/cursor` — `discover_transcripts`, `load_transcript`, URI scheme `cursor://transcript/{session_id}`. Indexed as `SourceKind::CursorTranscript` (`crates/store/src/types.rs`). Command: `sync_cursor_transcripts_cmd`.

### Local filesystem

| Integration | Mechanism | Crate |
|-------------|-----------|-------|
| Watch folders | `notify` crate, debounced events | `crates/watcher` |
| Manual index | `index_file`, `index_local_paths` | `src-tauri/src/index_ops.rs` |
| File formats | md, txt, csv, xlsx, etc. | `crates/ingest` |

### Agent extensions (local subprocess, not HTTP)

| Extension | Load path | Format | Execution |
|-----------|-----------|--------|-----------|
| **Skills** | `{app_data}/skills/` | `*.md` | Injected into agent system prompt |
| **Hooks** | `{app_data}/hooks/` | `*.json` | Shell command on events: `before_tool_call`, `after_tool_call`, `before_answer` |
| **Plugins** | `{app_data}/plugins/` | `*/plugin.json` | External commands as agent tools; permission `shell_exec` |

Code: `crates/agent/src/hooks.rs`, `crates/agent/src/plugins.rs`, `crates/agent/src/skills.rs`. Seeded example hook: `seed_hooks_dir` in `src-tauri/src/lib.rs`.

---

## Data Storage

### Primary database: SQLite (`kb.sqlite`)

**Owner:** `crates/store` only — no other module opens SQLite.

**Location:** `{app_data_dir}/kb.sqlite` (`src-tauri/src/lib.rs`). E2E: under `{app_data}/jarvis-e2e/kb.sqlite`.

**Stack:**

- `rusqlite` 0.32 with `bundled` + **FTS5**
- `sqlite-vec` 0.1 — `vec0` virtual table for embeddings

**Schema** (`crates/store/src/schema.rs`, version **3**):

| Table / object | Purpose |
|----------------|---------|
| `sources` | Indexed documents (kind, uri, title, hash, status, summary) |
| `chunks` | Text chunks per source |
| `chunks_fts` | FTS5 virtual table on chunk text |
| `vec_chunks` | `vec0(embedding float[dim])` — dimension set at open time |
| `embed_cache` | Dedup embeddings by text hash |
| `meta` | Key-value (schema version, embedder id) |
| `chat_sessions` / `chat_messages` | Persisted chat history + citations JSON |
| `tasks` | LLM-extracted tasks linked to sources |

**Source kinds** (`crates/store/src/types.rs`): `local_file`, `lark_doc`, `lark_msg`, `lark_sheet`, `lark_mail`, `lark_file`, `cursor_transcript`, `memory`.

**Migrations:** Inline in `init_schema` / `migrate()` — no external migration tool.

### Application config storage

| File | Format | Content |
|------|--------|---------|
| `{app_data}/config.json` | JSON | Full `AppConfig` including API keys, provider settings, agent profiles |

Load/save: `crates/config/src/file.rs`. Missing file → `AppConfig::default()`.

### FastEmbed model cache

Managed by `fastembed` / ONNX runtime — model weights downloaded to user cache on first `FastEmbedder::try_new` (not a Jarvis-managed path; standard fastembed behavior).

### No cloud database

Jarvis does not sync KB to a remote database. All RAG state is local SQLite.

---

## Authentication

Jarvis does **not** implement user login for the desktop app itself. Authentication applies to **external providers**:

### Cloud LLM (OpenAI-compatible)

| Field | Storage | Usage |
|-------|---------|-------|
| `cloud_api_key` | `config.json` (plaintext, local) | `Authorization: Bearer` header in `OpenAiChat` / `OpenAiEmbedder` |
| `cloud_base_url` | `config.json` | Any OpenAI-compatible API (default OpenAI v1) |

Set via Settings UI → `set_config` Tauri command.

### Ollama

No API key. Assumes trusted local instance at `ollama_base_url`. No TLS requirement for default localhost.

### Feishu / Lark

Delegated entirely to **lark-cli**:

- User OAuth / bot app credentials configured in lark-cli
- Jarvis reads `auth status` JSON only (`check_auth` in `crates/lark/src/health.rs`)
- Identity selection via `lark_identity` config field

### Agent plugins

Permission gate: `granted_plugin_permissions` (default includes `shell_exec`). Plugins declaring `permissions` in `plugin.json` must be granted before tools are exposed (`crates/agent/src/plugins.rs`).

### E2E test mode

`JARVIS_E2E=1` forces Mock providers and mock Lark auth (`src-tauri/src/e2e.rs` — `apply_e2e_config`, `lark_auth_status`).

---

## Monitoring

**No production APM, metrics backend, or error tracking service** is integrated.

| Mechanism | Scope |
|-----------|-------|
| Rust `println!` / stderr | CLI subprocess failures (lark-cli) |
| Tauri events | `IndexProgressEvent` emitted during indexing (`src-tauri/src/index_ops.rs`) |
| In-app status | `get_index_status`, `get_sync_status`, `IndexHealth` struct |
| CI logs | GitHub Actions job output only |

**Not present:** Sentry, OpenTelemetry, Prometheus, structured logging crate (e.g. `tracing`), health HTTP endpoints.

---

## CI/CD

**Workflow:** `.github/workflows/ci.yml`

| Job | Runner | Steps |
|-----|--------|-------|
| **rust** | `ubuntu-latest` | `cargo test --workspace` |
| **frontend** | `ubuntu-latest` | `npm ci`, `npm test`, `npm run build` |
| **e2e** | `windows-latest` | Depends on rust + frontend; `npm run tauri build`, `npm run test:e2e` |

**Triggers:** push/PR to `master`, `dev`; `workflow_dispatch`.

**E2E env (CI):**

```
JARVIS_E2E=1
JARVIS_E2E_FIXTURE={workspace}/e2e/fixtures/sample.md
```

**E2E tooling installed in CI:**

- `cargo install tauri-driver --locked`
- `msedgedriver-tool` from `https://github.com/chippers/msedgedriver-tool`

**E2E specs:** `e2e/specs/*.spec.ts` — smoke, navigation, qa, agent, memory, lark, settings, full-ui.

**No deploy/release workflow** in repo at mapping time — build artifacts produced locally via `npm run tauri build`.

---

## Environment Configuration

### Documented environment variables

| Variable | Used by | Purpose |
|----------|---------|---------|
| `JARVIS_E2E` | `src-tauri/src/e2e.rs` | `"1"` enables E2E mode (mock providers, isolated data dir) |
| `JARVIS_E2E_FIXTURE` | `e2e.rs`, `e2e/wdio.conf.ts` | Path to markdown file seeded into KB at startup |
| `JARVIS_E2E_APP` | `e2e/wdio.conf.ts` | Override path to built `tauri-app.exe` |
| `MSEDGEDRIVER_PATH` | `e2e/wdio.conf.ts` | Override Edge WebDriver binary |
| `TAURI_DEV_HOST` | `vite.config.ts` | Remote/mobile dev — sets Vite host and HMR |

### Config file vs env

**Primary configuration is `config.json`**, not environment variables. API keys and provider URLs belong in `AppConfig` persisted to disk.

### `.env` files

**No `.env` files present** in the repository at mapping time. Do not commit secrets; `cloud_api_key` is user-supplied via app Settings and stored in `config.json` only.

### Tauri app data path

Resolved at runtime via `app.path().app_data_dir()` with identifier `com.jarvis.app` (`src-tauri/tauri.conf.json`). Typical Windows location: `%APPDATA%\com.jarvis.app\`.

E2E override: subdirectory `jarvis-e2e/` under the same base (`e2e_data_dir` in `src-tauri/src/e2e.rs`).

---

## Webhooks

### Incoming HTTP webhooks

**Not implemented.** Jarvis is a desktop app with no embedded HTTP server for receiving external webhook callbacks.

### Outgoing HTTP

Only to configured LLM/embed endpoints (Ollama, OpenAI-compatible) via `reqwest`. No webhook dispatch to third-party URLs.

### Agent "hooks" (not webhooks)

The agent hook system (`crates/agent/src/hooks.rs`) runs **local shell commands** on lifecycle events:

| Event | Trigger |
|-------|---------|
| `before_tool_call` | Before agent executes a tool |
| `after_tool_call` | After tool returns |
| `before_answer` | Before final answer is returned |

Hook definitions are JSON files in `{app_data}/hooks/` with a `command` field executed via `std::process::Command`. These are **not** HTTP webhooks — they are OS subprocess hooks for user automation.

### Scheduled sync (internal timer, not webhook)

`sync_scheduler` (`src-tauri/src/sync_scheduler.rs`) runs periodic background sync (watch folders, Lark, Cursor) based on `scheduled_sync_*` config fields. This is an in-process Tokio timer, not an external cron or webhook trigger.

---

## Integration summary matrix

| System | Protocol | Required | Default in fresh install |
|--------|----------|----------|--------------------------|
| FastEmbed (ONNX) | Local + model download | For default embedder | Yes (default embedder) |
| OpenAI-compatible API | HTTPS + API key | Optional | No (empty key) |
| Ollama | HTTP localhost | Optional | No (not default chat) |
| lark-cli / Feishu | Subprocess + Lark APIs (via CLI) | Optional | No |
| Cursor transcripts | Local filesystem | Optional | No (empty root path) |
| SQLite | Local file | Yes | Yes (`kb.sqlite`) |
| Edge WebDriver | Local (E2E only) | CI / manual E2E | No |

---

*Integration map generated 2026-06-17. See `STACK.md` for languages, frameworks, and build configuration.*
