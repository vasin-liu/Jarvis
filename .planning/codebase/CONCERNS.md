# Jarvis Codebase Concerns

> Generated: 2026-06-17  
> Version: 1.8.0  
> Scope: Technical debt, bugs, security, performance, fragile integrations, scaling, dependencies, gaps

This document catalogs known risks in the Jarvis codebase. It is intended for planning and prioritization—not a blocker list. Items are ordered by severity within each section where possible.

---

## Tech Debt

### Monolithic frontend (`src/App.tsx`, ~2,798 lines)

All views (Chat, Library, Tasks, Memory, Settings), state, IPC calls, and event handlers live in a single React component. Navigation, Lark sync, agent orchestration, insights, hooks/plugins UI, and config editing are intertwined.

**Impact:** High change cost, difficult unit testing, merge conflicts, slow onboarding.  
**Fix approach:** Extract route-level views (`ChatView`, `LibraryView`, etc.), a thin `useJarvisConfig` hook, and shared layout primitives. Keep IPC wrappers in `src/lib/tauri.ts`. Migrate incrementally; E2E specs provide regression safety.

### Monolithic Tauri shell (`src-tauri/src/lib.rs`, ~1,317 lines)

`lib.rs` owns `AppState`, ~40 Tauri commands, provider lifecycle, watcher/scheduler wiring, Lark/memory/agent/insights IPC, and E2E bootstrap. `index_ops.rs` adds ~791 lines of index/sync orchestration.

**Impact:** Hard to reason about command boundaries; every feature touches the same files.  
**Fix approach:** Split into `commands/{chat,index,lark,memory,agent,config}.rs` modules; keep `lib.rs` as registration + `AppState` only. Mirror existing crate boundaries.

### No inline TODO/FIXME markers—but implicit debt

A repo-wide scan found **zero** `TODO`/`FIXME`/`HACK` comments in Rust/TS source (only in planning docs). Debt is structural rather than annotated: large files, duplicated orchestration patterns between `index_ops.rs` and `sync_scheduler.rs`, and config surface growth in `crates/config/src/types.rs` (~207 lines, 30+ fields).

**Fix approach:** Adopt lightweight `// TODO(issue):` comments when deferring work; consider splitting `AppConfig` into nested structs (`EmbeddingConfig`, `SyncConfig`, `AgentConfig`) with serde flattening.

### Embedding dimension change is destructive

`Store::reinit_vectors` (`crates/store/src/store.rs`) drops all chunks, FTS rows, embed cache, and marks every source `pending`. Triggered via `reinit_and_rebuild_index` in `src-tauri/src/lib.rs` when switching embedder model/dim.

**Impact:** User must fully re-index after any dim change; no incremental migration.  
**Fix approach:** Document UX clearly (already partially in Settings); long-term consider separate vec tables per embedder id or versioned vector columns.

### Agent tool protocol is ad-hoc XML+JSON

`crates/agent/src/tools.rs` parses `<tool_call>{"name":...}</tool_call>` from raw LLM text via `find` + `serde_json::from_str`. No schema validation beyond serde; malformed or partial tags silently fail (`parse_tool_call` returns `None`).

**Impact:** Fragile with real models; easy to get stuck loops or skipped tools.  
**Fix approach:** Structured output / JSON mode from providers; or a strict regex + error surfacing to UI; increase `MAX_TOOL_ROUNDS` configurability (`crates/agent/src/run.rs`, hardcoded `3`).

### Memory modeled as `SourceKind::Memory` only

`crates/memory/src/learn.rs` stores memories as regular sources/chunks—no dedicated `memories` table. Title/URI collision resolution uses fuzzy `contains` matching in `resolve_memory_id`.

**Impact:** Memory list mixed with library sources in DB; ambiguous forget/update if titles overlap.  
**Fix approach:** Dedicated table or stricter URI scheme (`memory://{uuid}`); deprecate title-based resolution.

### Config persisted as plaintext JSON

`crates/config/src/file.rs` reads/writes `config.json` with `cloud_api_key` in cleartext. No encryption, no OS keychain integration.

**Impact:** API keys exposed on disk and in backups.  
**Fix approach:** Store secrets via `keyring` crate or platform credential store; keep non-secret fields in JSON.

### `sqlite-vec` registration uses `unsafe` transmute

`crates/store/src/vecext.rs` registers the extension via `sqlite3_auto_extension` and `transmute`. Required by sqlite-vec API but brittle across rusqlite/sqlite-vec upgrades.

**Fix approach:** Pin versions deliberately; add integration test that fails if registration breaks; watch upstream for safe bindings.

---

## Known Bugs

### Embedder dim config fallback can desync store

`AppConfig::embedding_dim()` (`crates/config/src/types.rs`) uses `fastembed_model_dim(...).unwrap_or(self.fastembed_dim)` when model name is unknown. If `fastembed_dim` in config disagrees with actual model output, indexing fails at runtime with `DimMismatch` rather than at startup.

**Fix approach:** Validate dim at provider build time in `crates/config/src/providers.rs`; refuse startup if unknown model and dim mismatch.

### Lark live integration test ignored in CI

`crates/lark/src/sync.rs` has `#[ignore = "requires live lark-cli auth"]` on `fetch_from_url` test. CLI JSON shape regressions only caught by `FakeRunner` unit tests, not real `lark-cli` output.

**Fix approach:** Record golden JSON fixtures from lark-cli; optional manual/nightly job with credentials.

### FastEmbed integration test ignored in CI

`crates/embedder/src/fastembed.rs` — `#[ignore = "downloads ONNX model from network"]`. Production default embedder (`EmbedderProvider::FastEmbed` in config defaults) is not exercised in `cargo test` on CI.

**Fix approach:** Cache ONNX in CI artifact or use `MockEmbedder` default in test profile; add smoke job with model cache.

### Scheduled sync errors swallowed

`src-tauri/src/sync_scheduler.rs` — `let _ = rt.block_on(run_scheduled_sync(...))` discards errors. Failures are invisible except via partial `index_progress` events.

**Fix approach:** Log errors, persist `last_scheduled_sync_error` in `meta`, surface in Settings UI.

### IM chat fetch capped at 50 messages

`crates/lark/src/sync.rs` — `fetch_im_chat` uses `--page-size 50` with no pagination loop.

**Impact:** Long chat histories silently truncated in index.  
**Fix approach:** Paginate until empty page or configurable cap.

### Ingest rejects PDF and many formats

`crates/ingest/src/loader.rs` supports `txt`, `md`, `csv`, `xls/xlsx/ods` only. PDF test confirms rejection (`%PDF` → `UnsupportedType`).

**Impact:** Common document type unavailable despite design spec mentioning broad document support.  
**Fix approach:** Add `pdf-extract` or similar behind feature flag.

---

## Security Considerations

### API keys in config file and process memory

`cloud_api_key` stored in `AppConfig` (`crates/config/src/types.rs`), displayed in Settings (`src/App.tsx` password input but saved to JSON), passed to `OpenAiChat` / `OpenAiEmbedder` (`crates/llm/src/openai.rs`, `crates/embedder/src/openai.rs`).

**Impact:** Disk leakage, shoulder-surfing in devtools, accidental commit if config path copied.  
**Fix approach:** Keychain storage; never log config; redact in error messages.

### Default plugin permission grants `shell_exec`

`default_granted_plugin_permissions()` returns `vec!["shell_exec"]` (`crates/config/src/types.rs`). E2E seeds a demo plugin with shell permission (`src-tauri/src/lib.rs`).

**Impact:** Agent can execute arbitrary shell via enabled plugins without explicit user opt-in on fresh install.  
**Fix approach:** Default to empty granted list; require explicit toggle (UI exists but default is permissive).

### Hooks execute arbitrary shell commands

`crates/agent/src/hooks.rs` — enabled hooks run `command` via `cmd /C` or `sh -c` with env vars only. No sandbox, timeout, or allowlist.

**Impact:** Malicious or compromised `hooks/*.json` = full user privilege execution during agent runs.  
**Fix approach:** Opt-in per hook; timeouts; optional command allowlist; document threat model in AGENTS.md.

### Plugin tools execute arbitrary commands

`crates/agent/src/plugins.rs` — `execute_plugin_tool` spawns `tool.command` with `JARVIS_TOOL_ARGS` env. Permission gate is coarse (`shell_exec` only).

**Impact:** Same as hooks; plugins are explicitly designed for shell extension.  
**Fix approach:** Treat as trusted code only; warn in UI; consider Tauri capability restrictions.

### Lark CLI subprocess trust boundary

All Feishu access delegates to external `lark-cli` (`crates/lark/src/runner.rs`). Jarvis passes user tokens/URLs as CLI args; parses JSON stdout.

**Impact:** Dependency on lark-cli security; command injection if refs are not sanitized (partially handled by `normalize_lark_ref` in `crates/lark/src/parse.rs`).  
**Fix approach:** Audit argument encoding; keep `CommandRunner` injection for tests.

### No network TLS pinning for cloud providers

`reqwest` with `rustls-tls` trusts system CAs only—standard but no extra hardening.

---

## Performance Bottlenecks

### Single `Mutex<Connection>` on Store

`crates/store/src/store.rs` — all reads and writes serialize through one `Mutex<Connection>`. Embedding, search, chat history, and indexing contend on the same lock.

**Impact:** UI stalls during bulk index; agent tool loops block chat.  
**Fix approach:** `rusqlite` WAL mode + `PRAGMA journal_mode=WAL`; read connection pool; or move long index work to dedicated thread with batch commits.

### FastEmbed global mutex

`crates/embedder/src/fastembed.rs` — `Arc<Mutex<TextEmbedding>>`; `embed()` uses `spawn_blocking` but serializes all embed calls.

**Impact:** Parallel indexing of many files is effectively single-threaded for embeddings.  
**Fix approach:** Pool of embedders or batch larger `texts` slices per call in `indexer`.

### Full re-embed on content hash change only

`indexer` skips re-index when hash matches (`crates/indexer/src/lib.rs`), but rebuild all sources still re-embeds everything (`index_ops` rebuild path).

**Fix approach:** Embed cache (`embed_cache` table) is populated but ensure rebuild respects cache hits across full library.

### Synchronous folder scan in scheduled sync

`sync_scheduler.rs` calls `scan_folder` then `index_local_paths` sequentially for all watch folders—no parallelism, no file count cap.

### Chunker token count is word-split proxy

`crates/chunker/src/lib.rs` uses `split_whitespace().count()`—not a real tokenizer. Mismatch with LLM context limits for CJK content.

**Fix approach:** Use `tiktoken` or model-specific counter for budget-aware chunking.

### No SQLite WAL / busy_timeout

Only `PRAGMA foreign_keys = ON` set at open. Missing `busy_timeout` risks `SQLITE_BUSY` under concurrent access from scheduler + UI.

---

## Fragile Areas

### lark-cli JSON contract

`crates/lark/src/parse.rs` — `parse_cli_json`, `extract_title_and_text` assume specific JSON shapes from docs/sheets/mail/im/drive commands. `crates/lark/src/sync.rs` adds drive `+inspect` and file download paths.

**Impact:** Any lark-cli version bump can break sync silently or with parse errors.  
**Fix approach:** Version pin lark-cli in docs; fixture tests per resource type; health check (`crates/lark/src/health.rs`) before sync.

### sqlite-vec dimension fixed at table creation

`vec_chunks` created with `embedding float[{dim}]` (`crates/store/src/schema.rs`). Dimension baked into virtual table definition; change requires `reinit_vectors`.

### sqlite-vec + FTS5 dual-write consistency

`insert_chunks` writes `chunks`, `vec_chunks`, and `chunks_fts` in one transaction (`crates/store/src/store.rs`). `delete_chunks_for_source` must keep all three in sync—bugs here cause orphan FTS or vector rows.

### Tauri IPC string errors

Most commands return `Result<T, String>` (`src-tauri/src/lib.rs`). Error taxonomy lost at UI boundary; harder to branch on error type in `src/App.tsx`.

### E2E mode branching scattered

`src-tauri/src/e2e.rs` + `#[cfg]` / env checks throughout `lib.rs` and Lark sync. Risk of E2E mocks diverging from production paths (e.g. `lark_fixture_from_input` short-circuit in `sync_lark_doc`).

### Path resolution heuristics for ingest

`crates/ingest/src/loader.rs` — `resolve_existing_path` walks up to 4 parent dirs from cwd. Tauri dev cwd vs packaged app cwd can cause "file not found" in production only.

### Agent orchestration modes

`crates/agent/src/orchestrate.rs`, `router.rs` — pipeline and router modes add multi-LLM-call paths. Router uses keyword heuristics (`keyword_route_agent`)—brittle for non-Chinese/English queries.

### Cursor transcript discovery

`crates/cursor/src/discover.rs` depends on `cursor_projects_root` config and JSONL layout under `agent-transcripts`. Cursor IDE changes directory structure without notice.

---

## Scaling Limits

| Area | Practical limit | Bottleneck |
|------|-----------------|------------|
| Chunk count | ~low millions (untested) | Single SQLite file, no sharding; vec0 ANN quality/speed degrades |
| Source count | Thousands | `list_sources` loads all rows; Library UI renders full list in `App.tsx` |
| Embedding throughput | ~serial | FastEmbed mutex + blocking ONNX |
| IM / mail sync | 50 messages/chat | Hardcoded page size in `sync.rs` |
| Agent tool rounds | 3 | `MAX_TOOL_ROUNDS` in `run.rs` |
| Chunk size | 800 chars default | `ChunkerConfig` fixed in Tauri state, not user-configurable in UI |
| Config size | Unbounded | `watch_folders`, `agents`, skill lists grow JSON without validation |
| DB file | Single `kb.sqlite` | No vacuum/compact UX; no archive strategy |

### sqlite-vec scaling

Vector search uses `MATCH ... k = ?` (`store.rs`)—appropriate for personal KB but not validated at 100k+ chunks. No HNSW tuning exposed.

### Full library rebuild

`rebuild_index` re-processes every source—O(sources × chunks × embed_latency). No checkpoint/resume beyond per-source status in `sources.status`.

---

## Dependencies at Risk

| Dependency | Location | Risk |
|------------|----------|------|
| `lark-cli` (external binary) | `crates/lark/*`, config `lark_cli_bin` | Not a Cargo dep; version/API drift; Windows `.cmd` fallback fragile |
| `fastembed` 5.x | `crates/embedder/src/fastembed.rs` | ONNX model downloads; MSRV/native dep weight; model enum manual mapping |
| `sqlite-vec` 0.1 | `crates/store/src/vecext.rs` | Pre-1.0 API; `unsafe` registration; vec0 dim migration painful |
| `rusqlite` 0.32 bundled | `crates/store` | SQLite version tied to crate; extension compatibility |
| `reqwest` + cloud APIs | `llm`, `embedder` | OpenAI-compatible surface assumed; streaming parse in `openai.rs` |
| `tauri-driver` / Edge WebDriver | `e2e/`, CI | CI installs from git (`msedgedriver-tool`); Edge version skew breaks E2E |
| `calamine` | `ingest/loader.rs` | Spreadsheet-only; no PDF |
| `notify` 7 | `crates/watcher` | Platform-specific watch behavior; debounce in `service.rs` |

### Untracked `msedgedriver.exe` in repo root

Git status shows untracked `msedgedriver.exe`—should be gitignored (not in `.gitignore` currently) to avoid accidental commit of binary.

---

## Missing Critical Features

Per `AGENTS.md` milestones, the following are **designed but partial** or **non-goal for v1**—still gaps users may expect:

| Feature | Status | Notes |
|---------|--------|-------|
| PDF / DOCX ingest | Missing | `ingest/loader.rs` rejects unknown types |
| Full lark mail/im pagination | Partial | 50-message cap |
| Plugin sandbox | Missing | By design non-goal; shell exec is full trust |
| DAG / multi-agent orchestration | Partial | Pipeline/router exist; no DAG |
| Per-agent embedder override | Partial | `embed_resolver.rs` exists; UI coverage unclear |
| Encrypted config / keychain | Missing | Plaintext `config.json` |
| Index resume / partial rebuild UX | Partial | Per-source retry exists; no global pause |
| Cross-platform E2E | Missing | CI E2E Windows-only (`.github/workflows/ci.yml`) |
| Linux/macOS packaging validation | Unknown | Primary dev appears Windows-centric |
| Real tokenizer-based chunking | Missing | Word-split proxy only |
| Structured agent tool calls | Missing | Text protocol only |

---

## Test Coverage Gaps

### Rust unit/integration

| Crate / area | Tests | Gap |
|--------------|-------|-----|
| `llm` (`openai.rs`, `ollama.rs`) | Mock only in `mock.rs` | **No tests** for HTTP clients, streaming, error mapping |
| `embedder` (`openai.rs`, `ollama.rs`) | Mock + fastembed dim lookup | **No tests** for cloud/ollama; fastembed embed ignored in CI |
| `watcher` | `scan.rs`, `service.rs` basic tests | No integration with real `notify` events |
| `src-tauri` | `e2e.rs` one test | **No tests** for `index_ops.rs`, `sync_scheduler.rs`, `lib.rs` command handlers |
| `lark` | FakeRunner tests | Live CLI test ignored; no drive URL fixture for all paths |
| `agent` hooks/plugins | Manifest load tests | **No tests** for hook execution, plugin shell spawn |
| `insights` | summarize/tasks unit tests | No E2E for `run_insights_all_cmd` |
| `config` | file roundtrip | No validation tests for agent config permutations |

### Frontend (Vitest)

Only `src/lib/sourceDisplay.test.ts` and `src/lib/citations.test.ts`—**no component tests** for `App.tsx` (2,798 lines).

### E2E (WebdriverIO)

| Spec | Covers | Does **not** cover |
|------|--------|---------------------|
| `smoke.spec.ts` | App load | — |
| `navigation.spec.ts` | Nav + task list visible | Task CRUD, toggle status |
| `qa.spec.ts` | Q&A with fixture | Streaming edge cases |
| `memory.spec.ts` | Add/edit/forget | Auto-learn from chat |
| `agent.spec.ts` | Agent mode, orchestration toggles | Tool call UI, plugin invocation |
| `lark.spec.ts` | Mock URL sync | Sheet/mail/im, live CLI |
| `settings.spec.ts` | Index status, rebuild button | Reinit vectors, scheduled sync, cloud API key |
| `full-ui.spec.ts` | Journey smoke | Insights, per-source summarize/extract, hooks/plugins |

**Policy vs reality:** `.cursor/rules/e2e-required.mdc` requires E2E for all user-facing features. Gaps: **Tasks CRUD**, **Insights/summarize**, **Hooks/Plugins toggles**, **Cursor transcript sync**, **scheduled sync**, **Library per-source actions** (`data-testid="summarize-*"`, `extract-tasks-*` exist but untested).

### CI split

- `cargo test --workspace` runs on **ubuntu-latest** without FastEmbed network test or Windows-specific path logic.
- E2E runs on **windows-latest** only—Rust platform-specific code in `lark/runner.rs` (`.cmd` fallback) not tested on Linux CI.

---

## Recommended Prioritization (2026-06-17)

1. **Security:** Default-deny `shell_exec`; keychain for `cloud_api_key`.
2. **Reliability:** Scheduled sync error surfacing; lark-cli fixture expansion.
3. **Maintainability:** Split `App.tsx` and `lib.rs` before next major feature.
4. **Performance:** SQLite WAL + reduce FastEmbed lock contention.
5. **Testing:** E2E for tasks, insights, and settings scheduled sync; un-ignore fastembed in CI with model cache.

---

*This document should be refreshed when milestones ship or after major refactors.*
