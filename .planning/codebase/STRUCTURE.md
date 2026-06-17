# Jarvis Directory Structure

> Generated: 2026-06-17  
> Version: 1.8.0  
> Repo root: `D:\Work\99_Code\02_Rust\Jarvis`

This document maps the repository layout, naming conventions, and guidance for where to add new code.

---

## Directory Layout

```
Jarvis/
├── .cursor/
│   └── rules/                    # Cursor agent rules (e2e, stack, TDD, frontend)
├── .github/
│   └── workflows/
│       └── ci.yml                # Rust test + Windows E2E job
├── .planning/
│   └── codebase/                 # Architecture docs (this folder)
├── crates/                       # Rust workspace libraries (domain + infra)
│   ├── agent/                    # Agent profiles, tools, skills, hooks, plugins
│   ├── chunker/                  # Text chunking
│   ├── config/                   # AppConfig + provider factories
│   ├── cursor/                   # Cursor transcript discovery/parse
│   ├── embedder/                 # Embedder trait + providers
│   ├── indexer/                  # Document → chunks → vectors pipeline
│   ├── ingest/                   # File loading → Document
│   ├── insights/                 # Summarize + task extraction
│   ├── lark/                     # Feishu via lark-cli subprocess
│   ├── llm/                      # ChatModel trait + providers
│   ├── memory/                   # Long-term memory CRUD + learning
│   ├── rag/                      # ask / ask_stream RAG orchestration
│   ├── retriever/                # Hybrid vector + FTS + RRF
│   ├── store/                    # SQLite sole owner
│   └── watcher/                  # Folder watch + scan
├── docs/
│   ├── superpowers/
│   │   ├── specs/                # Design spec (normative for product)
│   │   └── plans/                # Milestone implementation plans
│   ├── personal_agent_ui_mockup.html
│   └── personal_ai_agent_architecture.svg
├── e2e/
│   ├── fixtures/                 # Sample files for E2E index tests
│   ├── specs/                    # WebdriverIO spec files per feature area
│   ├── helpers.ts                # DOM/React test utilities
│   ├── wdio.conf.ts              # WebDriver + tauri-driver config
│   └── README.md
├── src/                          # React frontend (Vite)
│   ├── lib/                      # Shared TS utilities + Vitest tests
│   ├── App.tsx                   # Main UI (all views)
│   ├── main.tsx                  # React entry
│   ├── index.css                 # Tailwind v4 + global styles
│   └── vite-env.d.ts
├── src-tauri/                    # Tauri 2 application crate
│   ├── src/
│   │   ├── main.rs               # Binary entry (calls lib::run)
│   │   ├── lib.rs                # AppState, IPC commands, run()
│   │   ├── index_ops.rs          # Index/rebuild/sync orchestration
│   │   ├── sync_scheduler.rs     # Background scheduled sync
│   │   ├── insights_ops.rs       # Post-index insights glue
│   │   └── e2e.rs                # E2E mode + fixtures
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   └── icons/
├── AGENTS.md                     # Agent/engineer onboarding (normative)
├── Cargo.toml                    # Workspace root + shared dependencies
├── Cargo.lock
├── package.json                  # npm scripts, React deps
├── package-lock.json
├── rust-toolchain.toml           # channel = "stable"
├── tsconfig.json
├── vite.config.ts
└── vitest.config.ts              # (if present) frontend unit tests
```

---

## Directory Purposes

| Path | Purpose |
|------|---------|
| `crates/` | All reusable Rust logic. Each crate is one concern; published only within workspace. |
| `src-tauri/` | Tauri app crate (`tauri-app` / `tauri_app_lib`). Composition root: wires crates, exposes IPC. |
| `src/` | React UI. Intentionally thin — no business logic beyond display and IPC. |
| `e2e/` | End-to-end tests via WebdriverIO + `tauri-driver`. Required for user-facing changes. |
| `docs/` | Design specs and reference mockups. Spec in `docs/superpowers/specs/` overrides mockups. |
| `.cursor/rules/` | Enforced dev policies (E2E required, TDD, stack versions). |
| `.github/workflows/` | CI: `cargo test`, Windows E2E on `dev`/`master`. |
| `.planning/codebase/` | GSD codebase maps (architecture, structure). |

### Crate map (workspace members)

| Crate | `src/` layout | Notes |
|-------|---------------|-------|
| `store` | `lib.rs`, `store.rs`, `schema.rs`, `types.rs`, `vecext.rs` | FTS5 + sqlite-vec registration |
| `ingest` | `document.rs`, `loader.rs`, `hash.rs` | Supports md, txt, docx, xlsx, etc. |
| `chunker` | `lib.rs` (inline chunk logic) | Pure functions, heavily unit-tested |
| `embedder` | `mock.rs`, `ollama.rs`, `fastembed.rs`, `openai.rs` | `Embedder` trait in `lib.rs` |
| `indexer` | `lib.rs` | `index_document`, `index_path`, embed cache |
| `retriever` | `retrieve.rs`, `rrf.rs` | `RetrieverConfig` defaults |
| `llm` | `mock.rs`, `ollama.rs`, `openai.rs`, `types.rs` | `ChatModel` trait |
| `rag` | `ask.rs`, `types.rs` | No store writes |
| `config` | `types.rs`, `file.rs`, `providers.rs` | JSON config on disk |
| `watcher` | `scan.rs`, `service.rs` | `notify` debounced watch |
| `lark` | `runner.rs`, `sync.rs`, `health.rs`, `parse.rs`, `options.rs` | `CommandRunner` injectable |
| `cursor` | `discover.rs`, `parse.rs` | JSONL agent transcripts |
| `insights` | `summarize.rs`, `tasks.rs` | LLM post-processing |
| `memory` | `learn.rs` | Memories indexed as `SourceKind::Memory` |
| `agent` | `run.rs`, `tools.rs`, `skills.rs`, `hooks.rs`, `plugins.rs`, `orchestrate.rs`, `router.rs` | Tool loop + multi-agent |

---

## Key File Locations

### Frontend

| File | Role |
|------|------|
| `src/main.tsx` | React DOM mount |
| `src/App.tsx` | All UI views, state, Tauri `invoke` / `listen` / `Channel` |
| `src/lib/citations.ts` | Parse `[n]` citation markers in answers |
| `src/lib/sourceDisplay.ts` | Source kind labels, status formatting |
| `src/lib/*.test.ts` | Vitest unit tests |
| `src/index.css` | Tailwind v4 entry, theme tokens, glass styles |
| `vite.config.ts` | Vite + `@tailwindcss/vite` + React plugin |

### Tauri shell

| File | Role |
|------|------|
| `src-tauri/src/main.rs` | `fn main()` → `tauri_app_lib::run()` |
| `src-tauri/src/lib.rs` | `AppState`, ~50 `#[tauri::command]` fns, `run()` builder |
| `src-tauri/src/index_ops.rs` | `rebuild_all_sources`, `index_local_paths`, Lark/Cursor sync |
| `src-tauri/src/sync_scheduler.rs` | `spawn_scheduler`, `run_scheduled_sync` |
| `src-tauri/src/insights_ops.rs` | `maybe_run_insights_for_source`, bulk insights |
| `src-tauri/src/e2e.rs` | `is_e2e_mode`, `apply_e2e_config`, `seed_e2e_fixture` |
| `src-tauri/tauri.conf.json` | App id, window, bundle, permissions |
| `src-tauri/Cargo.toml` | Depends on all workspace crates |

### Configuration and data (runtime, not in repo)

| Location | Content |
|----------|---------|
| `{app_data}/config.json` | `AppConfig` (providers, watch folders, agents) |
| `{app_data}/jarvis.db` | SQLite database |
| `{app_data}/skills/*.md` | Agent skill definitions |
| `{app_data}/hooks/*.json` | Agent hook scripts |
| `{app_data}/plugins/*/plugin.json` | Plugin manifests |

### Testing

| File | Role |
|------|------|
| `e2e/wdio.conf.ts` | WebDriver capabilities, Tauri service |
| `e2e/helpers.ts` | `setReactInputValue`, `clickViaDom`, waits |
| `e2e/specs/smoke.spec.ts` | Minimal app launch |
| `e2e/specs/full-ui.spec.ts` | Primary user journey |
| `e2e/specs/qa.spec.ts` | Q&A flow |
| `e2e/specs/agent.spec.ts` | Agent mode + tools |
| `e2e/specs/lark.spec.ts` | Feishu sync (mocked) |
| `e2e/specs/memory.spec.ts` | Memory CRUD |
| `e2e/specs/settings.spec.ts` | Config + index rebuild |
| `e2e/specs/navigation.spec.ts` | View routing |
| `e2e/fixtures/sample.md` | Default `JARVIS_E2E_FIXTURE` |

### Documentation and policy

| File | Role |
|------|------|
| `AGENTS.md` | Stack, milestones, file boundaries |
| `docs/superpowers/specs/2026-06-12-personal-ai-knowledge-hub-design.md` | Product design spec |
| `.cursor/rules/e2e-required.mdc` | E2E mandatory policy |
| `.cursor/rules/tdd-goal-driven.mdc` | Red-green-refactor workflow |
| `.cursor/rules/jarvis-stack.mdc` | Pinned stack versions |

---

## Naming Conventions

### Rust

| Element | Convention | Example |
|---------|------------|---------|
| Crates | `snake_case` single word | `store`, `embedder`, `rag` |
| Modules | `snake_case` | `index_ops`, `sync_scheduler` |
| Types / structs | `PascalCase` | `AppConfig`, `SourceKind`, `ChunkHit` |
| Functions | `snake_case` | `index_document`, `build_embedder` |
| Traits | `PascalCase` noun | `Embedder`, `ChatModel`, `CommandRunner` |
| Errors | `PascalCase` + `Error` | `StoreError`, `IndexError` |
| Tauri commands | `snake_case` | `ask_in_session`, `list_sources` |
| Config JSON keys | `snake_case` (serde) | `watch_folders`, `active_agent_id` |
| Source kinds (DB) | `snake_case` string | `local_file`, `cursor_transcript` |

### TypeScript / React

| Element | Convention | Example |
|---------|------------|---------|
| Components | `PascalCase` | `App` (single root component) |
| Files | `PascalCase.tsx` or `camelCase.ts` | `App.tsx`, `sourceDisplay.ts` |
| Types / interfaces | `PascalCase` | `ChatSession`, `Source` |
| Functions | `camelCase` | `formatIndexedAt` |
| Test IDs | `kebab-case` in `data-testid` | `data-testid="nav-chat"` |
| IPC invoke names | `snake_case` (match Rust) | `invoke("list_sources")` |

### E2E

| Element | Convention | Example |
|---------|------------|---------|
| Spec files | `*.spec.ts` | `agent.spec.ts` |
| Env flags | `JARVIS_E2E`, `JARVIS_E2E_FIXTURE` | Set in `package.json` scripts |

### URIs (indexed sources)

| Kind | URI pattern |
|------|-------------|
| Local file | `file://` absolute path |
| Lark doc | Lark-specific URI from `lark` crate |
| Cursor transcript | `cursor://transcript/{session_id}` |
| Memory | `memory://{id}` |

---

## Where to Add New Code

### New user-facing feature (full stack)

1. **Domain logic** → new or existing `crates/<name>/`
2. **IPC surface** → `#[tauri::command]` in `src-tauri/src/lib.rs` (thin wrapper)
3. **Orchestration** (if multi-step) → `src-tauri/src/index_ops.rs` or new `*_ops.rs` module
4. **UI** → `src/App.tsx` + `data-testid` selectors
5. **Tests** → unit in crate `#[cfg(test)]`, E2E in `e2e/specs/<area>.spec.ts`
6. **Journey** → extend `e2e/specs/full-ui.spec.ts` if primary flow

### New document source type

1. Add variant to `SourceKind` in `crates/store/src/types.rs`
2. Implement fetch/load → `ingest::Document` (new crate module or extend `lark`/`cursor` pattern)
3. Wire sync in `src-tauri/src/index_ops.rs` (`rebuild_all_sources` match arm)
4. Add label in `src/lib/sourceDisplay.ts`
5. E2E with deterministic mock

### New embedder or chat provider

1. Implement trait in `crates/embedder/` or `crates/llm/`
2. Add enum variant in `crates/config/src/types.rs` (`EmbedderProvider` / `ChatProvider`)
3. Wire in `crates/config/src/providers.rs`
4. Expose in Settings UI (`App.tsx` config section)
5. Unit test with mock HTTP/subprocess

### New agent tool

1. Register in `crates/agent/src/tools.rs` (`execute_tool` match)
2. Document in `build_tools_prompt`
3. E2E in `e2e/specs/agent.spec.ts`

### New agent skill / hook / plugin

| Type | Location | Loader |
|------|----------|--------|
| Skill | `{app_data}/skills/<id>.md` | `agent::load_skills_from_dir` |
| Hook | `{app_data}/hooks/<id>.json` | `agent::load_hooks_from_dir` |
| Plugin | `{app_data}/plugins/<id>/plugin.json` | `agent::load_plugins_from_dir` |

Seed templates: `seed_skills_dir` / `seed_hooks_dir` / `seed_plugins_dir` in `lib.rs`.

### Schema / DB change

1. **Only** in `crates/store/src/schema.rs`
2. Add migration logic if existing user DBs affected (ask before shipping)
3. Integration tests in `crates/store/`
4. May require `reinit_and_rebuild_index` command path

### New workspace crate

1. Add member to root `Cargo.toml`
2. Add path dependency in `src-tauri/Cargo.toml`
3. Document in `AGENTS.md` milestones
4. **Ask first** per `AGENTS.md` file boundaries

### Config field

1. `crates/config/src/types.rs` — `AppConfig` field + serde defaults
2. `get_config` / `set_config` automatically serialize (already generic)
3. Settings UI in `src/App.tsx`
4. E2E in `settings.spec.ts` if user-visible

---

## File Boundary Quick Reference

| ✅ Do | 🚫 Don't |
|-------|---------|
| Add store methods in `crates/store` | Open SQLite elsewhere |
| Keep `App.tsx` as IPC + display | Put retrieval logic in React |
| Use `MockEmbedder` / `MockChatModel` in tests | Call live APIs in CI |
| Add E2E spec per feature area | Merge UI changes without `data-testid` |
| Match existing `snake_case` IPC names | Rename commands without migration plan |
| Delegate from `lib.rs` to crates | Grow 200-line commands in `lib.rs` |

---

## Related Documents

- Architecture patterns and data flow: `.planning/codebase/ARCHITECTURE.md`
- Engineer onboarding: `AGENTS.md`
- Design spec: `docs/superpowers/specs/2026-06-12-personal-ai-knowledge-hub-design.md`
