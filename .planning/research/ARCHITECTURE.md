# Architecture Research

**Domain:** Local-first knowledge hub — Related-docs + read-only MCP (v1.11)
**Researched:** 2026-07-25
**Confidence:** HIGH (codebase integration); MEDIUM (MCP host placement / `rmcp` ecosystem)

## Standard Architecture

### System Overview

Related-docs and read-only MCP are **two consumers of the same hybrid retrieval + store list surface**. Neither owns SQLite; neither mutates index/write paths. Citation authority and RAG `ask` stay unchanged.

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Presentation                                                            │
│  LibraryView: select source → Related panel (open / navigate)            │
│  External: Cursor / Claude Desktop (MCP client)                          │
├─────────────────────────────────────────────────────────────────────────┤
│  Surfaces                                                                │
│  ┌──────────────────────┐   ┌─────────────────────────────────────────┐ │
│  │ Tauri IPC (thin)     │   │ MCP host (stdio binary — recommended)   │ │
│  │ list_related_sources │   │ tools: search, list_sources (RO only)   │ │
│  └──────────┬───────────┘   └──────────────────┬──────────────────────┘ │
├─────────────┴──────────────────────────────────┴────────────────────────┤
│  Domain crates                                                           │
│  ┌────────────────────┐  ┌──────────────┐  ┌──────────────────────────┐ │
│  │ retriever          │  │ agent tools  │  │ kb_readonly (shared)     │ │
│  │ related_sources()  │  │ search_know… │  │ search + list_sources    │ │
│  │ retrieve() (RRF)   │  │ list_sources │←─│ used by MCP + agent      │ │
│  └─────────┬──────────┘  └──────┬───────┘  └────────────┬─────────────┘ │
│            │                    │                         │              │
│            └────────────────────┴─────────────────────────┘              │
│                              Embedder trait (Mock in tests/E2E)          │
├──────────────────────────────────────────────────────────────────────────┤
│  store (sole SQLite owner) · sources / chunks / FTS5 / vec0              │
│  source_chunk_text · list_sources · search_vector · search_fts           │
└──────────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| `retriever::related_sources` | For seed `source_id`, find other Indexed sources with chunk overlap | New fn in `crates/retriever`: build query from summary/title/`source_chunk_text` → `retrieve` → aggregate by `source_id` → exclude seed → rank |
| `kb_readonly` (shared) | Canonical read API: `search(query)` + `list_sources()` | Small module in `retriever` or thin `crates/mcp` lib that both agent tools and MCP call — avoid duplicating `execute_tool` formatting |
| `store` | SQLite only; chunk text + hybrid search primitives | **Unchanged open policy**; optional `PRAGMA journal_mode=WAL` for concurrent MCP+GUI |
| Tauri `list_related_sources` | Thin IPC for Library panel | Extend `commands/library.rs` |
| `LibraryView` | Selection + related panel + navigate | FE state; `data-testid` for E2E |
| MCP host | Expose RO tools to external agents | `rmcp` stdio server binary (see Host Placement) |
| Agent tools | Existing in-app `search_knowledge` / `list_sources` | Prefer calling shared `kb_readonly`; no mutate tools on MCP |
| RAG / indexer / insights | Write and Q&A paths | **Unchanged** for v1.11 |

## Recommended Project Structure

```
crates/
├── retriever/src/
│   ├── retrieve.rs           # existing hybrid retrieve
│   ├── related.rs            # NEW: related_sources(seed_id) → Vec<RelatedSource>
│   └── lib.rs                # re-export
├── agent/src/tools.rs        # MODIFIED: search_knowledge/list_sources → shared helper
├── mcp/                      # NEW workspace crate (protocol + tool handlers)
│   ├── src/lib.rs            # ServerHandler; tools search + list_sources only
│   ├── src/tools.rs          # map MCP args → kb_readonly
│   └── tests/…               # MockEmbedder + tempfile Store
└── (optional) bin jarvis-mcp # stdio entry: open Store + build_embedder from config
src-tauri/src/
├── commands/library.rs       # + list_related_sources
├── commands/mcp.rs           # OPTIONAL: status / enable flag only (not tool logic)
└── e2e.rs                    # fixture sources enough for related + MCP unit tests
src/
├── views/LibraryView.tsx     # selection + related-docs panel
├── hooks/useLibrary.ts       # invoke listRelatedSources
└── lib/tauri.ts              # listRelatedSources()
e2e/specs/
├── library.spec.ts or memory.spec.ts  # related panel journey
└── (MCP): cargo test -p mcp preferred; optional smoke if binary wired
```

### Structure Rationale

- **`related_sources` in `retriever`:** Overlap is a retrieval concern (same RRF/embed path as Q&A), not UI or MCP protocol. Reuses `retrieve` + `Store::source_chunk_text` (already used by insights).
- **Shared `kb_readonly` before MCP crate grows:** Agent already implements `search_knowledge` / `list_sources` in `crates/agent/src/tools.rs`. Extract the query+format core so MCP does not fork retrieval semantics.
- **New `crates/mcp` (not fat Tauri commands):** Protocol (`rmcp`) stays out of `src-tauri` command handlers; shell stays thin. Matches “domain in crates” invariant.
- **No new SQLite tables / graph store:** On-demand overlap for v1.11; graph UI / Louvain explicitly out of scope.
- **Library selection is FE-only:** Today `LibraryView` is a flat action list with no selected source — add local selection state; no schema change.

## Architectural Patterns

### Pattern 1: Source-overlap via hybrid retrieve (related-docs)

**What:** Treat the seed source’s summary (preferred) or truncated `source_chunk_text` as the query; run existing `retrieve`; group `ChunkHit`s by `source_id`; drop seed; return top-N related sources with score + excerpt.
**When to use:** Library “related” panel for an Indexed source.
**Trade-offs:** Cheap, reuses RRF/MockEmbedder, no graph DB. Weaker than entity-graph merge (deferred). May surface WikiPage neighbors — acceptable if UI labels kinds; do not change citation policy.

**Example:**
```rust
// crates/retriever/src/related.rs (sketch)
pub async fn related_sources(
    store: &Store,
    embedder: &dyn Embedder,
    source_id: &str,
    config: &RetrieverConfig,
    limit: usize,
) -> Result<Vec<RelatedSource>> {
    let source = store.get_source(source_id)?;
    let query = source.summary.filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| truncate(store.source_chunk_text(source_id).unwrap_or_default(), 500));
    let hits = retrieve(store, embedder, &query, config).await?;
    Ok(aggregate_by_source(hits, source_id, store, limit)?)
}
```

### Pattern 2: Shared read-only KB surface (MCP + agent)

**What:** One function pair powers in-app agent tools and external MCP tools: hybrid `search` + Indexed `list_sources`.
**When to use:** Any external or internal read of the KB.
**Trade-offs:** Slight refactor of `agent::execute_tool`; prevents MCP/agent drift. MCP tool names can be `search` / `list_sources` while agent keeps `search_knowledge` as alias.

### Pattern 3: Thin IPC, fat crates (existing)

**What:** `#[tauri::command]` only maps `AppState` → crate call → `Result<_, String>`.
**When to use:** `list_related_sources`; any future MCP status command.
**Trade-offs:** Testable without WebView; consistent with `list_sources` / `summarize_source_cmd`.

### Pattern 4: Read-only MCP allowlist

**What:** MCP server registers **only** `search` and `list_sources`. No `add_memory`, `complete_task`, `remove_source`, plugin `shell_exec`.
**When to use:** Entire v1.11 MCP surface.
**Trade-offs:** Safe default for Cursor/Claude; mutate tools deferred explicitly in PROJECT.md.

## Data Flow

### Related-docs (in-app)

```
User selects Indexed source in Library
    ↓ invoke list_related_sources({ sourceId })
Tauri command (library.rs)
    ↓ AppState: store + embedder() + retriever
retriever::related_sources
    ├─ store.get_source / source_chunk_text (or summary)
    ├─ embedder.embed(query)
    ├─ store.search_vector + search_fts → RRF
    └─ aggregate → Vec<RelatedSource> (exclude seed)
    ↓
Library related panel: title, kind, score, excerpt
    ↓ click → select / scroll-to / focus that source row (no new DB write)
```

### MCP `search` / `list_sources` (external)

```
Cursor/Claude spawns jarvis-mcp (stdio)  OR  connects to optional local HTTP
    ↓ tools/call search { query }  /  list_sources {}
crates/mcp handler
    ↓ open Store(db_path) + build_embedder(config)   // store crate only
kb_readonly::search / list_sources
    ↓ same retrieve / list_sources as agent
JSON text result (titles, kinds, excerpts) — no write APIs
```

### Key Data Flows

1. **Overlap discovery:** Seed source text → hybrid retrieve → source-level ranking → UI panel.
2. **External KB query:** MCP `search` → same RRF path as `search_knowledge` → citations/excerpts for the *external* agent (Jarvis RAG citations unchanged).
3. **Catalog:** MCP `list_sources` → `store.list_sources` filtered to Indexed (mirror agent tool).
4. **No write path:** MCP and related-docs never call `index_document`, `delete_source`, memory mutate, or wiki compile.

### State Management

```
Library FE: selectedSourceId (component/hook state)
    ↓ on change
listRelatedSources(selectedSourceId) → related[]
AppConfig (optional): mcp.enabled / mcp transport notes — only if Settings UX needed
kb.sqlite: unchanged schema for MVP (compute on read)
```

## MCP Host Placement

| Option | How it works | Pros | Cons | Verdict |
|--------|--------------|------|------|---------|
| **A. Stdio sidecar binary** (`jarvis-mcp` / `tauri-app --mcp`) | Cursor spawns process; `rmcp` + `transport-io`; opens `{app_data}/kb.sqlite` via `store` | Matches Cursor/Claude Desktop; testable with `cargo test`; thin protocol crate | Two processes may contend on SQLite unless WAL; must resolve app_data path | **Recommended for v1.11** |
| **B. In-process inside Tauri GUI** | Start MCP HTTP/SSE or named-pipe from `setup` while app runs | Single Store in `AppState`; no second open | Cursor expects stdio child; GUI lifetime couples to MCP; Windows console/stdio awkward for GUI subsystem | **Not primary** — optional later “when app open” bridge |
| **C. Sidecar that IPC-proxies to running Tauri** | MCP binary forwards tools over local socket to AppState | True single DB connection | Extra protocol + lifecycle (“app must be running”); higher scope | Defer unless WAL/contention bites |

**Recommendation:** Ship **Option A** — workspace crate `mcp` + stdio binary using `rmcp` (official Rust SDK, server + `transport-io`). Resolve data dir the same way as Tauri (`app_data_dir` / env override for tests). Enable **WAL** on `Store::open` so GUI + MCP can coexist safely. Keep tool allowlist read-only. Do **not** embed MCP stdio into the Windows GUI subsystem binary as the only mode.

**E2E note:** Prefer `cargo test -p mcp` with `MockEmbedder` + tempfile DB for tool happy paths. Full WebDriver E2E covers the Library related panel; MCP protocol need not drive the WebView.

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| Personal desktop (default) | On-demand related retrieve; stdio MCP per client session — fine |
| Large libraries (1k+ sources) | Cap related `final_k` / aggregation limit; truncate seed query text; optional cache later |
| Concurrent GUI + MCP | WAL + short Store mutex holds; avoid long embeds blocking writers |

### Scaling Priorities

1. **First bottleneck:** Embed latency on every related-panel open — reuse summary as query; debounce FE; Mock in tests.
2. **Second bottleneck:** SQLite lock between GUI index and MCP search — WAL + read-only tools only.

## Anti-Patterns

### Anti-Pattern 1: Second SQLite owner outside `store`

**What people do:** MCP binary uses `rusqlite` directly, or FE caches a parallel index.
**Why it's wrong:** Schema drift; breaks single-DB-owner rule.
**Do this instead:** Only `Store::open` / store methods; MCP depends on `store` + `retriever`.

### Anti-Pattern 2: MCP mutate / plugin tools in v1.11

**What people do:** Expose `add_memory`, `remove_source`, or `shell_exec` “for completeness.”
**Why it's wrong:** PROJECT out-of-scope; trust/safety regression for external agents.
**Do this instead:** Hard allowlist `search` + `list_sources`.

### Anti-Pattern 3: Related-docs as new graph tables / Louvain

**What people do:** Materialize edge table or community detection for “overlap.”
**Why it's wrong:** Graph UI deferred; high cost for MVP.
**Do this instead:** On-demand RRF aggregation over existing chunks.

### Anti-Pattern 4: Fat Tauri command with retrieve + aggregate + MCP protocol

**What people do:** Implement overlap and MCP JSON-RPC inside `lib.rs` / commands.
**Why it's wrong:** Untestable; duplicates agent tools.
**Do this instead:** Logic in `retriever` / `mcp` crates; commands thin.

### Anti-Pattern 5: Live LLM/embedder in CI for related or MCP

**What people do:** Call FastEmbed/Ollama in E2E for overlap.
**Why it's wrong:** Flaky, slow, non-deterministic.
**Do this instead:** `MockEmbedder` / `JARVIS_E2E=1`; deterministic fixture sources that share tokens.

### Anti-Pattern 6: In-process stdio MCP as sole host inside GUI exe

**What people do:** Assume Cursor can attach stdio to the running Tauri window process.
**Why it's wrong:** Desktop clients spawn a **child** stdio server; GUI subsystem binaries are a poor stdio host on Windows.
**Do this instead:** Dedicated stdio binary (Option A); optional later HTTP bridge when app is foreground.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| Embedder (Mock / FastEmbed / Ollama / Cloud) | Trait via `build_embedder` | Related + MCP `search` need embedder; Mock in tests |
| Cursor / Claude Desktop | MCP client → stdio `jarvis-mcp` | User config points at binary + args/env for data dir |
| `rmcp` | ServerHandler + stdio transport | Pin latest stable in workspace deps at implement time |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Library UI ↔ Tauri | `invoke("list_related_sources")` | camelCase payloads; Indexed-only |
| Tauri ↔ retriever | Direct async call | Pass `&Store`, `&dyn Embedder`, `&RetrieverConfig` |
| MCP ↔ retriever/store | Direct in mcp binary | Same crates; no Tauri runtime required for unit tests |
| Agent tools ↔ kb_readonly | Function call | Keep tool names for prompts; share implementation |
| Related / MCP ↔ RAG ask | None | Do not alter citation fusion or wiki soft-skip |

### New vs Modified (explicit)

| Status | Artifact |
|--------|----------|
| **New** | `crates/retriever/src/related.rs` (+ unit/integration tests) |
| **New** | `crates/mcp` (+ stdio bin) with RO tools only |
| **New** | Library related panel UI + `data-testid`s + E2E coverage |
| **New** | Optional shared `kb_readonly` helper module |
| **Modified** | `commands/library.rs` + `lib/tauri.ts` + `LibraryView` / `useLibrary` |
| **Modified** | `agent/src/tools.rs` to call shared search/list (surgical) |
| **Modified** | `Store::open` — strongly consider WAL for MCP coexistence |
| **Modified** | Workspace `Cargo.toml` members + deps (`rmcp`) |
| **Unchanged** | RAG ask core, indexer write path, wiki compile, mutate agent tools on MCP, graph UI |

## Suggested Build Order

Dependency-aware order for v1.11 phases:

| Phase | Focus | Depends on | Verify |
|-------|--------|------------|--------|
| **1** | `related_sources` in `retriever` + MockEmbedder tests | Existing `retrieve`, `source_chunk_text` | `cargo test -p retriever` |
| **2** | Tauri `list_related_sources` + Library selection/panel + Vitest | Phase 1 | Vitest; IPC smoke |
| **3** | E2E related-docs journey (`data-testid`) | Phase 2; seeded fixtures | `npm run test:e2e:local` focused spec |
| **4** | Extract shared `kb_readonly` (agent `search_knowledge` / `list_sources`) | Existing agent tools | `cargo test -p agent` |
| **5** | `crates/mcp` + stdio bin; tools `search` / `list_sources` only | Phase 4; `rmcp` | `cargo test -p mcp` |
| **6** | WAL (if needed) + docs for Cursor MCP config; optional Settings “MCP enabled” copy | Phase 5 | Manual Cursor smoke; no mutate tools registered |

**Do not** start MCP UI/Settings before shared search works. **Do not** block related-docs on MCP binary. Parallelizable after Phase 1: UI (2–3) vs MCP (4–5) with a short integration at the end.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Related-docs ↔ retriever/store/Library | HIGH | Spot-checked `retrieve`, `source_chunk_text`, `list_sources`, `LibraryView` (no selection yet) |
| Reuse of agent search/list for MCP | HIGH | `crates/agent/src/tools.rs` already implements both |
| MCP stdio sidecar vs in-process | MEDIUM | Matches Cursor spawn model + `rmcp` docs; WAL/contention needs implement-time validation |
| Exact `rmcp` version pin | MEDIUM | Re-check crates.io at implementation (`rmcp` 2.x line observed in docs) |

## Sources

- Milestone intent: `.planning/PROJECT.md` (v1.11 Related-docs + MCP)
- Plan backlog row: `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` Out of Scope → v1.11
- Retriever: `crates/retriever/src/retrieve.rs`
- Store list/search/chunk text: `crates/store/src/store.rs` (`list_sources`, `search_vector`, `search_fts`, `source_chunk_text`)
- Agent RO tools: `crates/agent/src/tools.rs` (`search_knowledge`, `list_sources`)
- Library UI: `src/views/LibraryView.tsx`, `src/lib/tauri.ts`, `src-tauri/src/commands/library.rs`
- App data paths: `src-tauri/src/state.rs` (`kb.sqlite` under app_data)
- Invariants: `AGENTS.md`, CLAUDE.md architecture, `.cursor/rules/e2e-required.mdc`
- MCP SDK: [modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk/) / `rmcp` (stdio server transport) — confidence MEDIUM (web, verified)

---
*Architecture research for: Related-docs + read-only MCP (Jarvis v1.11)*
*Researched: 2026-07-25*
