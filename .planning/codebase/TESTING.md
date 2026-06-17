# Jarvis Testing Guide

> Mapped: 2026-06-17  
> Sources: `AGENTS.md`, `.cursor/rules/`, `package.json`, `.github/workflows/ci.yml`, `e2e/`, and crate test modules.

---

## Overview

Jarvis uses a **three-layer test pyramid** mandated by `.cursor/rules/tdd-goal-driven.mdc`:

| Layer | Tool | When |
|-------|------|------|
| Unit | `cargo test`, Vitest | Pure logic, parsers, RRF, display helpers |
| Integration | `cargo test` + `tempfile` + mocks | Store, index→retrieve pipelines |
| E2E | WebdriverIO + `tauri-driver` | **Every user-facing feature** |

CI (`.github/workflows/ci.yml`) runs all three: `cargo test --workspace`, `npm test`, and Windows `e2e` job.

---

## Test Frameworks

### Rust — `cargo test`

```bash
cargo test --workspace          # CI / full suite
cargo test -p store             # single crate
cargo test -p rag ask_returns   # filter by test name
```

- **Sync tests:** `#[test]` for pure functions (e.g. `crates/retriever/src/rrf.rs`).
- **Async tests:** `#[tokio::test]` for embedder, indexer, rag, agent paths.
- **Dev-deps:** `tempfile` for on-disk SQLite; declared per-crate `Cargo.toml`.

### Frontend — Vitest

Configured in `vite.config.ts`:

```typescript
test: {
  environment: "node",
  include: ["src/**/*.test.ts"],
},
```

```bash
npm test    # vitest run (CI frontend job)
```

- **No** `@testing-library/react` component tests in repo yet — current Vitest coverage is **pure `src/lib/*` helpers**.
- Tests import from `vitest`: `describe`, `it`, `expect`.

### E2E — WebdriverIO + tauri-driver

```bash
npm run tauri build                              # release binary required
npm run test:e2e:local                         # JARVIS_E2E=1 + fixture
npm run test:e2e                               # CI equivalent
```

| File | Role |
|------|------|
| `e2e/wdio.conf.ts` | Runner config, spawns `tauri-driver`, sets env |
| `e2e/helpers.ts` | React-safe input helpers, navigation, chat flows |
| `e2e/specs/*.spec.ts` | Mocha `describe` / `it` journeys |
| `e2e/fixtures/sample.md` | Seeded knowledge for Q&A tests |
| `e2e/README.md` | Prerequisites, spec map, checklist |

**Framework:** Mocha BDD, 180s timeout, `maxInstances: 1`, Edge WebDriver on Windows.

---

## Test File Organization

```
Jarvis/
├── crates/
│   ├── store/
│   │   ├── src/store.rs          # #[cfg(test)] mod tests { … }
│   │   ├── src/schema.rs         # migration tests inline
│   │   └── tests/persistence.rs  # integration tests (crate root)
│   ├── rag/src/ask.rs            # pipeline integration tests inline
│   ├── embedder/src/mock.rs      # mock + its own unit tests
│   ├── lark/src/sync.rs          # FakeRunner CLI tests + #[ignore] live
│   └── …                         # same inline pattern per module
├── src/
│   └── lib/
│       ├── citations.ts
│       ├── citations.test.ts     # colocated Vitest
│       └── sourceDisplay.test.ts
├── src-tauri/src/
│   ├── e2e.rs                    # E2E mock logic + unit test for fixtures
│   └── index_ops.rs              # command-layer tests with MockEmbedder
└── e2e/specs/
    ├── full-ui.spec.ts           # primary journey (required extension point)
    ├── smoke.spec.ts
    ├── qa.spec.ts
    ├── lark.spec.ts
    ├── memory.spec.ts
    ├── agent.spec.ts
    ├── settings.spec.ts
    └── navigation.spec.ts
```

**Convention:** prefer `#[cfg(test)] mod tests` at bottom of the source file under test; use `crates/*/tests/` for cross-module integration (e.g. store persistence).

---

## Test Structure — Real Examples

### Unit test (pure RRF)

`crates/retriever/src/rrf.rs`:

```rust
#[test]
fn fuses_two_lists_by_chunk_id() {
    let vector = vec![hit(1, "a", 0.1), hit(2, "b", 0.2)];
    let fts = vec![hit(2, "b", -1.0), hit(3, "c", -2.0)];
    let fused = rrf_fuse(&vector, &fts, 60, 10);
    assert_eq!(fused[0].chunk_id, 2);
}
```

### Integration test (index → ask pipeline)

`crates/rag/src/ask.rs`:

```rust
#[tokio::test]
async fn ask_returns_citations_for_indexed_doc() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("kb.sqlite");
    let file = dir.path().join("kb.md");
    fs::write(&file, "Jarvis uses sqlite-vec for embeddings.").unwrap();

    let store = Store::open(&db, 4).unwrap();
    let embedder = MockEmbedder::new(4);
    index_path(&store, &embedder, &ChunkerConfig::default(), &file)
        .await
        .unwrap();

    let resp = ask(
        &store,
        &embedder,
        &MockChatModel,
        &RetrieverConfig::default(),
        "sqlite-vec embeddings",
    )
    .await
    .unwrap();

    assert!(!resp.citations.is_empty());
    assert!(!resp.answer.is_empty());
}
```

### Store integration test (persistence)

`crates/store/tests/persistence.rs`:

```rust
#[test]
fn data_survives_reopen_and_is_searchable() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("kb.sqlite");
    // … upsert, insert_chunks, drop connection …
    let store = Store::open(&path, 4).unwrap();
    let vhits = store.search_vector(&[0.9, 0.1, 0.0, 0.0], 1).unwrap();
    assert_eq!(vhits[0].text, "rust vector search");
}
```

### Vitest (frontend helper)

`src/lib/citations.test.ts`:

```typescript
describe("parseCitations", () => {
  it("returns empty on invalid json", () => {
    expect(parseCitations("{bad")).toEqual([]);
  });
});
```

### E2E smoke

`e2e/specs/smoke.spec.ts`:

```typescript
describe("Jarvis smoke", () => {
  it("shows chat navigation", async () => {
    const nav = await $('[data-testid="nav-chat"]');
    await expect(nav).toBeDisplayed();
  });
});
```

### E2E Q&A with fixture

`e2e/specs/qa.spec.ts` uses `askQuestion` from `e2e/helpers.ts`; expects Mock answer containing `"Mock"` after asking about `xyzzy-plugh` (keyword in `e2e/fixtures/sample.md`).

---

## Mocking

### `MockEmbedder` (`crates/embedder/src/mock.rs`)

- Deterministic SHA256-based vectors; same text → same embedding.
- `MockEmbedder::new(dim)` — E2E uses `dim = 4` via `apply_e2e_config`.
- Implements `Embedder` trait; used in rag, agent, memory, indexer tests.

### `MockChatModel` (`crates/llm/src/mock.rs`)

- Rule-based responses keyed on system-prompt substrings (agent tools, router, summaries, memory extraction).
- `NO_CONTEXT` user message → Chinese “not found” string (matches `rag` empty-retrieval path).
- `complete_stream` splits on spaces for token streaming tests.

### Provider wiring (`crates/config/src/providers.rs`)

```rust
EmbedderProvider::Mock => Ok(Arc::new(MockEmbedder::new(config.mock_embed_dim))),
ChatProvider::Mock => Arc::new(MockChatModel),
```

### `FakeRunner` (`crates/lark/src/runner.rs`)

- In-memory `HashMap` of `"program args" → stdout` for `lark-cli` tests.
- Used in `crates/lark/src/sync.rs` — no network, no real Feishu.

```rust
let runner = FakeRunner::new();
runner.insert(
    "lark-cli docs +fetch --api-version v2 --doc tok123 …",
    r#"{"ok":true,"data":{…}}"#,
);
let doc = fetch_doc(&runner, &opts(), "tok123").unwrap();
```

### E2E mode (`JARVIS_E2E=1`)

`src-tauri/src/e2e.rs`:

| Function | Behavior |
|----------|----------|
| `is_e2e_mode()` | Checks env var |
| `apply_e2e_config` | Forces Mock embedder + chat, clears watch folders |
| `seed_e2e_fixture` | Indexes `JARVIS_E2E_FIXTURE` at startup |
| `lark_auth_status` | Fake connected Feishu user |
| `lark_fixture_from_input` | Maps E2E URLs → mock `Document` |

E2E Lark URL: `https://e2e.feishu.cn/file/e2e-mock-token` (marker `xyzzy-lark-e2e`).

### React E2E helpers (`e2e/helpers.ts`)

WebDriver cannot reliably set React controlled inputs — use:

- `setReactInputValue` / `setReactCheckbox` / `setReactRadio`
- `clickViaDom` — DOM click when WebDriver `click()` misses handlers
- `openNav`, `askQuestion`, `waitForAssistantMessage`, `submitChatForm`

---

## Fixtures

| Fixture | Path | Used by |
|---------|------|---------|
| E2E markdown | `e2e/fixtures/sample.md` | `JARVIS_E2E_FIXTURE`, Q&A + library stats |
| Lark E2E doc | Built in `e2e.rs` `lark_document()` | `lark.spec.ts`, `full-ui.spec.ts` |
| Temp SQLite | `tempfile::tempdir()` in Rust tests | store, rag, agent, retriever |
| In-memory DB | `Store::open_in_memory(4)` | fast tests without disk |

Fixture content (`e2e/fixtures/sample.md`) includes keyword **xyzzy-plugh** for deterministic retrieval assertions.

---

## Coverage

- **No `cargo-tarpaulin`, Istanbul, or codecov** configuration in repo.
- Coverage is **policy-driven**, not metric-gated:
  - All crates participate in `cargo test --workspace` (CI).
  - Vitest runs all `src/**/*.test.ts` (currently 2 files).
  - E2E runs **all** `e2e/specs/**/*.spec.ts` on Windows CI.
- **Live tests** opt out via `#[ignore]` (e.g. `live_fetch_drive_xls` in `crates/lark/src/sync.rs`).

When extending coverage, add tests at the **lowest appropriate layer** first (unit before E2E).

---

## Test Types

### Unit

| Area | Location | Example |
|------|----------|---------|
| Chunking | `crates/chunker/src/lib.rs` | token/paragraph splits |
| RRF fusion | `crates/retriever/src/rrf.rs` | rank fusion math |
| FTS query build | `crates/retriever/src/retrieve.rs` | `build_fts_query` |
| Mock embedder | `crates/embedder/src/mock.rs` | determinism |
| Mock chat | `crates/llm/src/mock.rs` | NO_CONTEXT path |
| Display helpers | `src/lib/sourceDisplay.test.ts` | label maps |
| Citation parse | `src/lib/citations.test.ts` | JSON edge cases |
| Store types | `crates/store/src/types.rs` | serde round-trips |

### Integration

| Area | Location | Pattern |
|------|----------|---------|
| Store CRUD / search | `crates/store/src/store.rs`, `tests/persistence.rs` | tempfile DB |
| Full RAG ask | `crates/rag/src/ask.rs` | index_path + ask |
| Agent tool loop | `crates/agent/src/run.rs` | `run_agent` + MockChatModel |
| Orchestration | `crates/agent/src/orchestrate.rs` | multi-agent mock |
| Lark CLI parse | `crates/lark/src/sync.rs` | FakeRunner |
| Hybrid retrieve | `crates/retriever/src/retrieve.rs` | index + retrieve |
| Memory learn | `crates/memory/src/learn.rs` | store + mocks |
| Tauri index ops | `src-tauri/src/index_ops.rs` | MockEmbedder |

### E2E (user-facing — mandatory)

| Spec | Coverage |
|------|----------|
| `e2e/specs/smoke.spec.ts` | App load, settings panel |
| `e2e/specs/navigation.spec.ts` | All nav views |
| `e2e/specs/qa.spec.ts` | Seeded Q&A, library stats |
| `e2e/specs/memory.spec.ts` | Add / edit / forget memory |
| `e2e/specs/agent.spec.ts` | Agent mode, orchestration toggles |
| `e2e/specs/settings.spec.ts` | Index status, rebuild |
| `e2e/specs/lark.spec.ts` | Feishu mock connect + sync |
| `e2e/specs/full-ui.spec.ts` | End-to-end primary journey |

**Definition of done** (`.cursor/rules/e2e-required.mdc`):

1. Spec under `e2e/specs/`
2. `data-testid` on new UI (`src/App.tsx`)
3. E2E mocks in `src-tauri/src/e2e.rs` if external API involved
4. `npm run test:e2e:local` green
5. Extend `full-ui.spec.ts` if part of primary journey

---

## Common Patterns

### Arrange–Act–Assert with tempfile

```rust
let dir = tempfile::tempdir().unwrap();
let store = Store::open(&dir.path().join("kb.sqlite"), 4).unwrap();
// act
// assert
```

### Inject traits, not concrete cloud clients

Tests never call live Ollama, OpenAI, or Feishu in CI. Use `MockEmbedder`, `MockChatModel`, or `FakeRunner`.

### `open_in_memory` for speed

`crates/rag/src/ask.rs` — `ask_without_hits_returns_no_context` uses `Store::open_in_memory(4)` when persistence is irrelevant.

### E2E waits on app readiness

`e2e/wdio.conf.ts` `before` hook waits for `data-testid="app-root"` and `nav-chat` (120s / 30s timeouts).

### E2E chat flow

1. Wait for `data-testid="e2e-active"` and `chat-session-ready`
2. `setReactInputValue` on `chat-input`
3. `submitChatForm` → waits for `ask-handle-count` increment or `ask-busy`
4. `waitForAssistantMessage` — fails fast on `chat-error`

### Ignored live tests

```rust
#[test]
#[ignore = "requires live lark-cli auth"]
fn live_fetch_drive_xls() { … }
```

Run locally only: `cargo test -p lark live_fetch_drive_xls -- --ignored`.

### TDD checklist (from rules)

```
Goal: [what user can do]
Verify:
  - [ ] unit tests pass
  - [ ] integration tests pass (if applicable)
  - [ ] E2E spec added/updated (user-facing)
  - [ ] npm run test:e2e:local passes (user-facing)
```

---

## CI Commands Summary

| Job | Runner | Command |
|-----|--------|---------|
| `rust` | ubuntu-latest | `cargo test --workspace` |
| `frontend` | ubuntu-latest | `npm ci` → `npm test` → `npm run build` |
| `e2e` | windows-latest | `npm run tauri build` → `npm run test:e2e` with `JARVIS_E2E=1` |

---

## Key Reference Files

| Topic | Path |
|-------|------|
| CI workflow | `.github/workflows/ci.yml` |
| npm scripts | `package.json` |
| Vitest config | `vite.config.ts` |
| WDIO config | `e2e/wdio.conf.ts` |
| E2E helpers | `e2e/helpers.ts` |
| E2E docs | `e2e/README.md` |
| E2E mocks | `src-tauri/src/e2e.rs` |
| Mock embedder | `crates/embedder/src/mock.rs` |
| Mock chat | `crates/llm/src/mock.rs` |
| Fake lark runner | `crates/lark/src/runner.rs` |
| TDD rule | `.cursor/rules/tdd-goal-driven.mdc` |
| E2E rule | `.cursor/rules/e2e-required.mdc` |
