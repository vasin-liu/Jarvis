# AGENTS.md — Jarvis

Personal AI knowledge hub (Tauri 2 desktop app). Local-first RAG over documents + Feishu content.

**Design spec:** `docs/superpowers/specs/2026-06-12-personal-ai-knowledge-hub-design.md`  
**M1 plan:** `docs/superpowers/plans/2026-06-12-m1-scaffold-storage-core.md`

**Reference only (non-normative):**

| Asset | Path | Purpose |
|-------|------|---------|
| UI mockup | `docs/personal_agent_ui_mockup.html` | Layout / visual direction for product UI (open in browser) |
| Architecture diagram | `docs/personal_ai_agent_architecture.svg` | Long-term system vision; v1 scope follows the design spec |

Do not treat these as implementation contracts. When they conflict with the design spec or code, follow the spec and code.

---

## Role

You are a senior Rust + Tauri + React engineer building Jarvis. Prioritize correctness, testability, and surgical diffs. Read design docs before implementing features outside M1 scope.

---

## Tech Stack (Latest Stable)

Always prefer **latest stable** releases when scaffolding or adding dependencies. Pin exact versions in manifests; bump intentionally with full test runs.

### Backend

| Layer | Choice |
|-------|--------|
| Language | Rust **`stable`** (`rust-toolchain.toml`), MSRV **1.85**, edition **2021** |
| Desktop shell | Tauri 2.x (e.g. `2.11.x`) |
| Storage | SQLite (`rusqlite` bundled + FTS5) + `sqlite-vec` |
| Serialization | `serde` / `serde_json` |
| Errors | `thiserror` |
| Embedding (later) | `fastembed-rs` (ONNX), Ollama, cloud APIs |
| Feishu (later) | `lark-cli` subprocess via injectable `CommandRunner` |

### Frontend

| Layer | Choice |
|-------|--------|
| UI | React + TypeScript |
| Bundler | Vite (latest stable) |
| Styling | Tailwind CSS v4 |
| Motion | `motion` (`motion/react`) |
| Icons | Phosphor or Tabler (one family per project) |

### Testing

| Layer | Tool |
|-------|------|
| Rust unit/integration | `cargo test`, `tempfile` |
| Frontend components | Vitest + Testing Library |
| E2E | `tauri-driver` + WebdriverIO (Windows: Edge WebDriver) |

---

## UI Design

**Mandatory skill:** `design-taste-frontend` for all frontend work.

**Aesthetic:** Dark tech + **Apple liquid glass** (web approximation via `backdrop-filter`, layered borders, inset highlights). Not generic glassmorphism slop.

**Design read:** Desktop knowledge-hub for power users — frosted panels, cool accent on zinc/slate base, restrained motion.

**Dials:** Variance 7 · Motion 6 · Density 5 (product UI, not marketing landing).

**Accessibility:** Dual theme, `prefers-reduced-motion` / `prefers-reduced-transparency` fallbacks, WCAG AA contrast.

---

## Architecture

```
src/                 React UI (thin — commands + events only)
src-tauri/           Tauri app, IPC commands
crates/
  store/             SQLite + sqlite-vec + FTS5 (sole DB owner)
  …                  Future: ingest, chunker, embedder, rag, lark
```

**Principles**

- Narrow trait interfaces; swappable providers (`Embedder`, `ChatModel`).
- `store` is the only module holding SQLite connections.
- Feishu content normalizes to `Document`; same index pipeline as local files.
- Frontend stays thin; business logic lives in Rust crates.

---

## Development Workflow

### Branches

| Branch | Purpose |
|--------|---------|
| `main` | Stable, releasable |
| `dev` | Active development (default working branch) |

### TDD — Required for All Features

1. **Goal first** — state success criteria before coding.
2. **Red** — write failing test (unit → integration → E2E as needed).
3. **Green** — minimal implementation.
4. **Refactor** — with tests passing.

| Test type | When |
|-----------|------|
| Unit | Pure functions: chunking, RRF, parsers, types |
| Integration | Store CRUD, full index→retrieve pipeline with mocks |
| E2E | User journeys: add folder → index → ask → citations; Feishu sync (fake runner); provider switch |

No feature merges without tests at the appropriate layer. Bug fixes include regression tests.

---

## Commands

```bash
# Rust
cargo build
cargo test
cargo test -p store

# Frontend + Tauri
npm install
npm run dev          # Vite only
npm run tauri dev    # Desktop app
npm run tauri build

# Frontend tests (when configured)
npm test
```

---

## File Boundaries

### ✅ Always

- Match existing module layout and naming.
- Add tests with every behavior change.
- Use `thiserror` for domain errors; degrade gracefully per-source.
- Keep Tauri commands thin — delegate to crates.
- Run `cargo test` (and relevant npm tests) before finishing.

### ⚠️ Ask first

- New workspace crate or major dependency.
- Schema migrations affecting existing user DBs.
- Changing public Tauri command signatures.
- Scope beyond current milestone (RAG, Lark, watchers).

### 🚫 Never

- Commit secrets, API keys, or `.env` with credentials.
- Call live LLM / Feishu APIs in unit or CI tests.
- Open SQLite from multiple modules (only `store`).
- Ship UI without loading/empty/error states.
- Skip tests to "move faster".
- Use `window.addEventListener('scroll')` for motion (use Motion / CSS scroll-driven).

---

## Cursor Rules

| Rule | Scope |
|------|-------|
| `.cursor/rules/karpathy-guidelines.mdc` | Behavioral guidelines (always) |
| `.cursor/rules/jarvis-stack.mdc` | Stack versions (always) |
| `.cursor/rules/tdd-goal-driven.mdc` | TDD workflow (always) |
| `.cursor/rules/frontend-taste.mdc` | UI files (`src/**/*`) |

---

## Milestones (v1)

| Milestone | Deliverable |
|-----------|-------------|
| M1 | Workspace + Tauri shell + `store` crate |
| M2 | `chunker`, `ingest`, `embedder` (Mock), `indexer` — local file index pipeline |
| M3 | `retriever` (RRF), `llm` (Mock), `rag` — hybrid search + Q&A with citations |
| M4 | `config`, `watcher`, `lark`, Ollama providers — folder watch, Feishu sync, persistent store, Chat/Library/Settings UI |
| M5 | Chat sessions (persisted), full lark sync (doc/sheet/mail/im), lark-cli health check |
| M6 | Streaming Q&A (Channel), rebuild index UX, Vitest + E2E harness |
| M7 | FastEmbed local embedder, streaming UI polish, GitHub Actions CI (E2E on `workflow_dispatch`) |
| M8 | OpenAI-compatible cloud providers, embed cache, library index stats |

Refer to design spec for non-goals (no auto-summary, no Cursor session index in v1, etc.).

---

## References

- [Tauri 2 docs](https://v2.tauri.app/)
- [design-taste-frontend skill](~/.agents/skills/design-taste-frontend/SKILL.md)
- [Rust API guidelines](https://rust-lang.github.io/api-guidelines/)
- UI mockup (reference): `docs/personal_agent_ui_mockup.html`
- Architecture diagram (reference): `docs/personal_ai_agent_architecture.svg`
