# Phase 18: MCP tools search + list_sources - Research

**Researched:** 2026-07-28
**Domain:** Shared read-only KB helpers (`kb_readonly`) + MCP `search` / `list_sources` over hybrid retrieve
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
### Shared `kb_readonly` (anti-drift)
- **D-01:** Extract a **shared read-only KB API** used by both agent tools and MCP so hybrid search / Indexed listing cannot diverge. Prefer placement that lets **`crates/mcp` depend on store + retriever + embedder/config — not on the full `agent` tool loop / plugins / memory mutate**. Recommended shape: structured helpers (e.g. under `retriever` or a small `kb_readonly` module owned by a lib both can call) returning data; agent formats LLM text + `Citation`s; MCP formats JSON `CallToolResult`.
- **D-02:** Agent tool **names** stay `search_knowledge` / `list_sources` for prompts; MCP tool names stay `search` / `list_sources`. Shared implementation underneath — name alias only at the edges.

### Search semantics (MCP-01)
- **D-03:** MCP `search` must call the same hybrid path as agent knowledge search: `retriever::retrieve` + **`RetrieverConfig::default()`** (same vector_k / fts_k / final_k / rrf_k as in-app). Do **not** invent a second index or alternate fusion.
- **D-04:** Optional MCP `limit` (from Phase 17 `SearchParams`) **clamps** result count to ≤ default `final_k` (or an explicit shared max) — never unbounded top-k.
- **D-05:** Excerpt truncation matches agent citation practice (~**200** Unicode chars + ellipsis). Empty query → structured error (align with `RetrieveError::EmptyQuery`), not a panic.

### `list_sources` semantics (MCP-02)
- **D-06:** Inventory only sources with **`IndexStatus::Indexed`** (match agent `list_sources` filter).
- **D-07:** Bound payload size: hard cap on number of sources returned (researcher/planner pick a concrete N with truncation indicator if needed). Prefer fields **`id`, `title`, `kind`** — same information richness as agent lines, plus `id` for hosts.
- **D-08:** **Do not expose absolute filesystem / `file://` paths** in MCP payloads (privacy / exfil). Agent today omits URI in list text; MCP must not reintroduce full paths. If a URI field is ever needed, basename-only — default is omit path.

### MCP runtime wiring
- **D-09:** `JarvisMcp` holds live `Arc<Store>` plus an **`Embedder`** built via **`config::build_embedder`** from sibling `config.json` (same as GUI). Tests use **`MockEmbedder`**. Fail closed with clear errors if embedder/config cannot be built for search — do not silently return empty as success when the provider is broken (empty KB hits remain a valid empty result).
- **D-10:** Keep Phase 17 path / WAL / allowlist locks: still exactly `{search, list_sources}`; only `store` opens SQLite; no GUI subsystem on `jarvis-mcp`.

### Response shape
- **D-11:** MCP returns **structured JSON** tool content (results array / source array), not the Chinese agent prompt strings. Shared core is structured; presentation may differ by consumer; retrieval inputs/filters/limits stay identical.

### Testing & docs
- **D-12:** Verify with **`cargo test -p mcp`** and **`cargo test -p agent`** (shared helper coverage) using MockEmbedder + tempfile fixtures. Update Phase 17 allowlist/stub tests to assert real semantics. **No WebDriver MCP E2E** this phase (Phase 19). Update **`docs/mcp.md`** to remove “stub only” wording and document real tool behavior + caps.

### Claude's Discretion
- Exact module path / crate for `kb_readonly` (`retriever::…` vs thin dedicated module) as long as D-01 dependency constraint holds
- Exact JSON field names and list-sources hard-cap N
- Whether MCP search includes citation-like `chunk_id` / `loc` fields (useful for hosts) vs title+excerpt only — prefer including stable ids when cheap
- Whether agent string formatting is refactored surgically to call shared helpers in the same plan wave

### Deferred Ideas (OUT OF SCOPE)
- **Phase 19:** MCP happy-path cargo harness + related-docs E2E / citation trust regression
- **MCP-F01:** `get_chunk` / `read` document tools
- **MCP-F02:** Settings toggle for MCP
- **MCP-F03:** HTTP/SSE transport
- Agent-visible URI in `list_sources` text (not requested; keep omit)
- Changing RAG ask / citation fusion for MCP excerpts
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MCP-01 | External MCP clients can call `search` using the same hybrid retrieval path as in-app agent knowledge search | Place `search_kb` in `retriever::kb_readonly` calling `retrieve` + `RetrieverConfig::default()`; MCP `search` + agent `search_knowledge` both consume it; clamp `SearchParams.limit` ≤ `final_k` (8) |
| MCP-02 | External MCP clients can call `list_sources` to inventory indexed sources | Place `list_indexed_sources` filtering `IndexStatus::Indexed`, fields `id`/`title`/`kind`, hard-cap **200** + `truncated`, **no URI/path** in MCP JSON |
</phase_requirements>

## Summary

Phase 17 already shipped `jarvis-mcp` stdio, exact tool allowlist `{search, list_sources}`, WAL on `Store::open`, and stub handlers. Phase 18 replaces stubs with real read-only KB semantics by extracting a **structured** shared API so agent tools and MCP cannot diverge (Pitfall 9). The hybrid brain already exists: `retriever::retrieve` with defaults `vector_k=20`, `fts_k=20`, `final_k=8`, `rrf_k=60.0`. Agent `search_knowledge` / `list_sources` in `crates/agent/src/tools.rs` currently inline those calls and format Chinese LLM text + `Citation`s — that logic must move into structured helpers; presentation stays at the edges.

**Primary recommendation:** Add `crates/retriever/src/kb_readonly.rs` (re-export from `retriever::lib`), wire agent + MCP in the same milestone waves, use MockEmbedder + tempfile for `cargo test -p mcp` / `-p agent`, and return MCP results as JSON text content (`CallToolResult::success` + `ContentBlock::text`) matching Phase 17 test parsing — with hard-cap **N=200** for list and search clamp to **final_k=8**.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Hybrid KB search | Library / Domain (`retriever`) | MCP host binary | Retrieval + RRF ownership stays in `retriever`; MCP is a thin protocol adapter |
| Indexed source inventory | Library / Domain (`retriever` + `store` list) | MCP host binary | Filter/cap in shared helper; `store` remains sole SQLite owner |
| Embedder construction | Config / providers | MCP `main` | `config::build_embedder` from sibling `config.json` (GUI parity) |
| Agent tool presentation | Agent crate | — | Format Chinese strings + `Citation` from structured hits |
| MCP tool presentation | MCP crate (`server.rs`) | — | JSON `CallToolResult`; allowlist unchanged |
| Protocol / stdio transport | MCP binary | — | Phase 17 complete; do not reopen |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `retriever` (workspace) | path | `retrieve` + new `kb_readonly` | Existing hybrid RRF brain; already used by agent `[VERIFIED: crates/retriever/src/retrieve.rs]` |
| `store` (workspace) | path | `list_sources`, `get_source`, `ChunkHit` | Sole SQLite owner `[VERIFIED: crates/store]` |
| `embedder` + `MockEmbedder` | path | Query vectors | Trait injection; CI/offline tests `[VERIFIED: crates/embedder]` |
| `config` (`load_config`, `build_embedder`) | path | Dim + provider wiring | Same config as GUI `[VERIFIED: crates/config/src/providers.rs]` |
| `rmcp` | **2.2.0** (workspace pin) | MCP tool handlers / `CallToolResult` | Phase 17 lock; do **not** bump to 3.x beta `[VERIFIED: Cargo.toml workspace.dependencies]` |
| `serde` / `serde_json` | workspace | MCP JSON payloads | Already in `crates/mcp` `[VERIFIED: crates/mcp/Cargo.toml]` |
| `tokio` | workspace | Async tool handlers + tests | Already required by mcp binary |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tempfile` | 3 (dev) | Fixture DB dirs | Integration tests in mcp/agent/retriever |
| `indexer` (dev) | path | Seed indexed docs for search tests | Same pattern as `retriever` retrieve tests |
| `thiserror` | workspace | Map `RetrieveError` in helpers | Keep typed errors inside crates |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `retriever::kb_readonly` | New `crates/kb_readonly` | Extra workspace member for ~100 LOC — refuse unless deps force it |
| `retriever::kb_readonly` | Helpers inside `crates/mcp` only | Agent would need `mcp` dep or fork — violates D-01 / Pitfall 9 |
| Shared helpers | Wrap `agent::execute_tool` from MCP | Pulls mutate tools / plugins — Pitfall 1; **forbidden** |
| `CallToolResult::structured` | Text JSON only (Phase 17 style) | Structured is nicer but Phase 17 tests parse `ContentBlock::text`; keep text JSON for continuity `[CITED: docs.rs/rmcp CallToolResult]` |

**Installation:** No new crates. Add path deps only:

```toml
# crates/mcp/Cargo.toml — add
retriever = { path = "../retriever" }
embedder = { path = "../embedder" }
```

**Version verification:** `rmcp` workspace pin `2.2.0` confirmed in root `Cargo.toml`. `cargo search rmcp` surfaces newer **3.x beta** — **do not adopt** (REQUIREMENTS / Phase 17 D-02; MSRV). `[VERIFIED: Cargo.toml]`

## Package Legitimacy Audit

> No new external packages in this phase. `rmcp` already installed/pinned in Phase 17.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| `rmcp` | crates | pre-existing pin 2.2.0 | n/a (seam signals unknown) | modelcontextprotocol/rust-sdk | seam=[SUS] unknown signals; project-locked OK | **No install** — already approved Phase 17; keep 2.2.0 |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** `rmcp` seam metadata incomplete — **not actionable**; do not re-litigate pin

## Architecture Patterns

### System Architecture Diagram

```
External MCP client (Cursor / Claude Desktop)
        │ stdio JSON-RPC (Phase 17)
        ▼
 jarvis-mcp main
   load_config(sibling config.json)
   build_embedder(cfg) ──fail closed──► exit / tool error
   Store::open(kb.sqlite, dim)   // store only
        │
        ▼
 JarvisMcp { store, embedder }
   search(query, limit?) ──► kb_readonly::search_kb
   list_sources()        ──► kb_readonly::list_indexed_sources
        │
        ▼
 crates/retriever::kb_readonly
   search_kb ──► retrieve(store, embedder, query, &RetrieverConfig::default())
                 then map ChunkHit → KbHit (title via get_source, excerpt ≤200 chars)
                 then apply limit clamp ≤ final_k
   list_indexed_sources ──► store.list_sources()
                 filter IndexStatus::Indexed
                 map → {id, title, kind}  // NO uri
                 hard-cap 200 + truncated flag
        │
        ├──────────────► MCP: JSON CallToolResult (D-11)
        └──────────────► agent::execute_tool: Chinese text + Citation (D-02 names)
```

### Recommended Project Structure

```
crates/retriever/src/
├── kb_readonly.rs      # NEW: search_kb, list_indexed_sources, types, caps
├── retrieve.rs         # unchanged hybrid path
├── related.rs          # untouched (related-docs)
└── lib.rs              # pub use kb_readonly::{...}

crates/agent/src/tools.rs
└── search_knowledge / list_sources → call kb_readonly, then format

crates/mcp/
├── src/server.rs       # JarvisMcp holds Arc<Store> + Arc<dyn Embedder>; real handlers
├── src/main.rs         # build_embedder before serve; fail closed
├── Cargo.toml          # + retriever, embedder
└── tests/allowlist.rs  # keep name allowlist; replace stub body asserts
docs/mcp.md             # remove stub wording; document caps + JSON shape
```

### Pattern 1: Structured core, dual presenters

**What:** `kb_readonly` returns plain Rust structs (`KbHit`, `KbSourceList`). Agent builds prompt lines + `rag::Citation`; MCP builds `serde_json::Value` → text content block.  
**When to use:** Always for MCP-01/02.  
**Example:**

```rust
// crates/retriever/src/kb_readonly.rs (prescriptive sketch)
pub const EXCERPT_MAX_CHARS: usize = 200;
pub const LIST_SOURCES_MAX: usize = 200;

pub struct KbHit {
    pub chunk_id: i64,
    pub source_id: String,
    pub title: String,
    pub loc: String,
    pub excerpt: String,
}

pub async fn search_kb(
    store: &Store,
    embedder: &dyn Embedder,
    query: &str,
    limit: Option<usize>,
) -> Result<Vec<KbHit>> {
    let mut cfg = RetrieverConfig::default();
    let max = cfg.final_k; // 8
    if let Some(n) = limit {
        cfg.final_k = n.clamp(1, max);
    }
    let hits = retrieve(store, embedder, query, &cfg).await?;
    // map + unicode truncate to EXCERPT_MAX_CHARS + '…'
    // ...
}
```

### Pattern 2: Fail closed on embedder; empty hits ≠ error

**What:** Startup / handler: if `build_embedder` fails → process exit or `CallToolResult::error` / `ErrorData`. Empty retrieval → success with `"results": []`.  
**When to use:** D-09.  
**Anti-pattern:** Catching embed errors and returning `[]` as if the KB had no matches.

### Pattern 3: Surgical agent refactor (same phase)

**What:** In `execute_tool` arms for `search_knowledge` / `list_sources`, call helpers then format. Keep plugin/mutate arms untouched.  
**When to use:** Claude's Discretion — **recommend yes in same plan wave** so golden hit-ordering tests can assert one implementation.  
**Note:** Today agent LLM lines use full `hit.text` while `Citation.excerpt` is truncated `[VERIFIED: crates/agent/src/tools.rs:69-86]`. Prefer formatting LLM lines from **shared excerpt** to reduce prompt bloat and align with D-05.

### Anti-Patterns to Avoid

- **Calling `agent::execute_tool` from MCP:** registers write surface risk; pulls plugins/memory.
- **mcp → agent direct dep for helpers:** transitive `config → agent` already exists for profiles; do **not** add intentional tool-loop coupling.
- **Forking retrieve with different k/RRF:** Pitfall 9 / D-03.
- **Returning `uri` / absolute paths in MCP JSON:** D-08 / Pitfall 6.
- **Leaving stub `not_implemented` tests green:** Phase 18 must replace them (D-12).
- **Byte-slicing excerpts:** use Unicode `chars()` (historical hang note in `agent/tests/real_embed_hang.rs`).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Hybrid search | Custom vector-only or FTS-only MCP path | `retriever::retrieve` + `RetrieverConfig::default()` | Same RRF as RAG/agent; Pitfall 5/9 |
| MCP protocol framing | Hand-rolled JSON-RPC | `rmcp` 2.2.0 `#[tool]` / `CallToolResult` | Spec drift; Phase 17 already wired |
| Embedder selection | Hardcoded FastEmbed in mcp | `config::build_embedder` | GUI parity; Mock in tests |
| SQLite access | `rusqlite` in mcp | `store::Store` | Single DB owner rule |
| Truncation / caps | Unbounded dumps | `EXCERPT_MAX_CHARS=200`, `LIST_SOURCES_MAX=200`, `final_k` clamp | Pitfall 6 |
| Read-only policy | Trust `readOnlyHint` only | Keep compile-time allowlist of 2 tools | Pitfall 1 / MCP-04 |

**Key insight:** Phase 18 is an extraction + wiring phase, not a new retrieval algorithm.

## Common Pitfalls

### Pitfall A: Semantic drift (agent vs MCP)

**What goes wrong:** Different top-k, status filters, or Wiki handling.  
**Why:** Copy-paste from tools.rs into server.rs.  
**How to avoid:** Single `kb_readonly` module; golden fixture test shared or duplicated assertions on same helper.  
**Warning signs:** Two `retrieve(` call sites with different configs.  
`[CITED: .planning/research/PITFALLS.md Pitfall 9]`

### Pitfall B: Unbounded payloads / path exfil

**What goes wrong:** Full library dump or `file://C:\Users\...` in MCP results.  
**How to avoid:** Hard-cap 200; fields id/title/kind only; never serialize `Source.uri` for MCP.  
`[CITED: .planning/research/PITFALLS.md Pitfall 6]` `[VERIFIED: CONTEXT D-07/D-08]`

### Pitfall C: Silent empty on broken embedder

**What goes wrong:** Host thinks KB is empty when FastEmbed/Ollama failed.  
**How to avoid:** Fail closed at `main` build_embedder; map runtime embed errors to tool error content.  
`[VERIFIED: CONTEXT D-09]`

### Pitfall D: Constructor / test break from Phase 17 stubs

**What goes wrong:** `JarvisMcp::new(None)` stub tests assume no embedder.  
**How to avoid:** Change to `JarvisMcp::new(store, embedder)`; rewrite allowlist body tests with tempfile + MockEmbedder; keep name-allowlist test intact.  
`[VERIFIED: crates/mcp/tests/allowlist.rs]`

### Pitfall E: Dim mismatch Store vs Embedder

**What goes wrong:** Open Store with dim A, embed with dim B → vector search failures.  
**How to avoid:** One `AppConfig` load drives both `embedding_dim()` and `build_embedder`; tests use Mock dim consistently.  
`[VERIFIED: crates/mcp/src/main.rs + config::embedding_dim]`

### Pitfall F: Wrapping full agent tool loop

**What goes wrong:** Accidental mutate tools via shared execute.  
**How to avoid:** Structural allowlist remains 2 tools; helpers are read-only functions only.  
`[CITED: PITFALLS Pitfall 1]`

## Code Examples

### Shared search + list (prescriptive)

```rust
// Source: project code seams — retrieve.rs + tools.rs patterns [VERIFIED]
use embedder::Embedder;
use store::{IndexStatus, Store};
use crate::retrieve::{retrieve, RetrieverConfig};
use crate::error::Result;

pub const EXCERPT_MAX_CHARS: usize = 200;
pub const LIST_SOURCES_MAX: usize = 200;

pub fn truncate_excerpt(text: &str) -> String {
    if text.chars().count() > EXCERPT_MAX_CHARS {
        format!("{}…", text.chars().take(EXCERPT_MAX_CHARS).collect::<String>())
    } else {
        text.to_string()
    }
}

pub struct KbSourceEntry {
    pub id: String,
    pub title: String,
    pub kind: String, // SourceKind::as_str()
}

pub struct KbSourceList {
    pub sources: Vec<KbSourceEntry>,
    pub truncated: bool,
    pub total_indexed: usize,
}

pub fn list_indexed_sources(store: &Store) -> Result<KbSourceList> {
    let indexed: Vec<_> = store
        .list_sources()?
        .into_iter()
        .filter(|s| s.status == IndexStatus::Indexed)
        .collect();
    let total_indexed = indexed.len();
    let truncated = total_indexed > LIST_SOURCES_MAX;
    let sources = indexed
        .into_iter()
        .take(LIST_SOURCES_MAX)
        .map(|s| KbSourceEntry {
            id: s.id,
            title: s.title,
            kind: s.kind.as_str().to_string(),
        })
        .collect();
    Ok(KbSourceList { sources, truncated, total_indexed })
}
```

### MCP JSON response shape (recommended)

```json
// search success
{
  "results": [
    {
      "chunk_id": 42,
      "source_id": "local:…",
      "title": "Sample",
      "loc": "L1-20",
      "excerpt": "…"
    }
  ]
}

// list_sources success
{
  "sources": [
    { "id": "local:…", "title": "Sample", "kind": "local_file" }
  ],
  "truncated": false,
  "total_indexed": 1
}
```

Include `chunk_id` + `loc` (Claude's Discretion — **yes**, cheap from `ChunkHit`, useful for hosts / future MCP-F01). Omit `uri`, `score`, full chunk `text`. `[ASSUMED]` field names are discretion; planner should lock these in PLAN.

### MCP handler wiring

```rust
// Source: Phase 17 server.rs + docs.rs rmcp CallToolResult [CITED: docs.rs/rmcp]
// Keep ContentBlock::text(JSON) so existing first_text() test helper keeps working.
Ok(CallToolResult::success(vec![ContentBlock::text(
    serde_json::to_string(&payload).unwrap_or_else(|_| "{\"error\":\"serialize\"}".into()),
)]))

// Empty query / retrieve EmptyQuery → tool-level error (caller-visible), not panic:
Ok(CallToolResult::error(vec![ContentBlock::text(
    r#"{"error":"empty_query","message":"query must be non-empty"}"#.into(),
)]))
```

### main.rs embedder (fail closed)

```rust
// Source: extend crates/mcp/src/main.rs [VERIFIED pattern]
let cfg = load_config(&config_path).unwrap_or_else(|_| AppConfig::default());
let dim = cfg.embedding_dim();
let embedder = config::build_embedder(&cfg).map_err(|e| {
    eprintln!("failed to build embedder: {e}");
    process::exit(1);
})?;
let store = Arc::new(Store::open(&db_path, dim)?);
let service = JarvisMcp::new(store, embedder).serve(stdio()).await?;
```

Prefer: if `config.json` is required for non-mock providers, document that missing config falls back to `AppConfig::default()` (today's dim behavior) but FastEmbed default may still fail-closed at build — acceptable.

### Agent consumer (surgical)

```rust
// search_knowledge arm
let hits = retriever::search_kb(store, embedder, query, None).await?;
if hits.is_empty() {
    return Ok(("未找到相关内容。".into(), vec![]));
}
// format lines from hits; build Citation from hit fields (excerpt already truncated)
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Phase 17 stub `not_implemented` | Real `kb_readonly` + JSON | Phase 18 | MCP-01/02 |
| Inline agent retrieve/list | Shared structured helpers | Phase 18 | Anti-drift |
| Unbounded agent `list_sources` | Shared hard-cap 200 | Phase 18 | Agent also capped (behavior change — intentional) |

**Deprecated/outdated:**
- Stub JSON `{status:not_implemented, phase:18}` — remove from runtime and tests
- Byte-index excerpt truncation (if any residual) — use `chars()`

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | JSON field names `results` / `sources` / `truncated` / `total_indexed` / `chunk_id` / `loc` / `excerpt` | Code Examples | Host docs mismatch — lock in PLAN |
| A2 | List hard-cap **200** is the right product N | Standard Stack / D-07 | Too small for huge libraries — raise only with docs + tests |
| A3 | Applying list cap to agent tool output is acceptable | Architecture | Longer agent lists truncated — document in mcp.md + changelog note |
| A4 | Prefer `CallToolResult::success(text JSON)` over `structured` for 2.2.0 continuity | Standard Stack | Hosts that prefer `structured_content` get text-only — still parseable |

## Open Questions (RESOLVED)

1. **Agent list truncation messaging** — RESOLVED: append one-line footer when `truncated` (e.g. Chinese `（已截断，共 N 条）`).
2. **Startup vs per-call embedder failure** — RESOLVED: per-call embed/`retrieve` errors → MCP error JSON; never empty-success.
3. **Ordering of Indexed list** — RESOLVED: preserve `Store::list_sources` order; no new sort in Phase 18.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable / cargo | All tests | ✓ (project toolchain) | MSRV 1.85 | — |
| `rmcp` 2.2.0 | MCP handlers | ✓ pinned | 2.2.0 | none (do not use 3.x) |
| Live FastEmbed / Ollama | Production search | optional | — | MockEmbedder in tests; fail closed if build fails |
| WebDriver / tauri-driver | MCP E2E | n/a this phase | — | Deferred Phase 19 |

**Missing dependencies with no fallback:** none for Phase 18 cargo scope  
**Step 2.6:** External IDE MCP clients not required for verification (D-12).

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` + `tokio::test` |
| Config file | workspace `Cargo.toml` / crate manifests |
| Quick run command | `cargo test -p retriever kb_readonly -- --nocapture` |
| Full suite command | `cargo test -p retriever -p mcp -p agent` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MCP-01 | `search_kb` uses default RetrieverConfig / same hits as retrieve | unit/integration | `cargo test -p retriever` | ❌ Wave 0 — add `kb_readonly` tests |
| MCP-01 | MCP `search` returns JSON results, clamps limit, empty query error | integration | `cargo test -p mcp` | ❌ replace stub tests in `tests/allowlist.rs` or new `tests/tools_kb.rs` |
| MCP-02 | Indexed-only; no uri; cap 200 + truncated | unit + mcp | `cargo test -p retriever` / `-p mcp` | ❌ Wave 0 |
| MCP-04 continuity | tools/list still exactly `{search, list_sources}` | unit | `cargo test -p mcp tool_allowlist` | ✅ keep |
| Shared anti-drift | agent arms call helpers (or helper golden) | unit/integration | `cargo test -p agent` | ❌ add focused tests |
| TRUST / E2E | WebDriver MCP | — | — | Deferred Phase 19 (D-12) |

### Sampling Rate

- **Per task commit:** `cargo test -p retriever` (helpers) or `cargo test -p mcp` (handlers)
- **Per wave merge:** `cargo test -p retriever -p mcp -p agent`
- **Phase gate:** Full three-crate suite green; `docs/mcp.md` updated; no WebDriver required

### Wave 0 Gaps

- [ ] `crates/retriever/src/kb_readonly.rs` + unit/integration tests (Indexed filter, cap, excerpt chars, empty query)
- [ ] `crates/mcp/tests/` — replace stub `not_implemented` asserts with tempfile Store + MockEmbedder + indexed fixture
- [ ] `crates/agent` tests covering `search_knowledge` / `list_sources` via helpers (or thin wrapper tests)
- [ ] Framework install: none — cargo already present

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | stdio trust = host process (Phase 17) |
| V3 Session Management | no | — |
| V4 Access Control | partial | Structural tool allowlist only `{search, list_sources}` |
| V5 Input Validation | yes | Non-empty query; clamp `limit`; serde `SearchParams` |
| V6 Cryptography | no | — |

### Known Threat Patterns for MCP + local KB

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path / home-dir exfil via `uri` | Information Disclosure | Omit paths (D-08); id/title/kind only |
| Unbounded list DoS / context flood | Denial of Service | Hard-cap 200 + search final_k |
| Mutate via MCP | Tampering | Allowlist; no execute_tool wrap |
| SQL injection | Tampering | Parameterized queries inside `store` only |
| Broken embedder → false empty | Spoofing / Misleading | Fail closed (D-09) |

## Project Constraints (from .cursor/rules/)

| Rule | Directive relevant to Phase 18 |
|------|--------------------------------|
| `tdd-goal-driven.mdc` | Goal → failing test → implement; inject `Embedder`; bugfix = regression test |
| `karpathy-guidelines.mdc` | Surgical diffs; no speculative features; honor locked CONTEXT |
| `jarvis-stack.mdc` | MSRV 1.85; only `store` opens SQLite; core logic in crates |
| `e2e-required.mdc` | User-facing E2E mandatory — **MCP WebDriver deferred to Phase 19 by CONTEXT D-12**; this phase = cargo verification only. Do not skip Phase 19 trust gate. |

Also honor workspace invariants: thin MCP handlers; Mock providers in CI; no live LLM in unit tests.

## Sources

### Primary (HIGH confidence)
- `crates/agent/src/tools.rs` — search_knowledge / list_sources semantics `[VERIFIED]`
- `crates/retriever/src/retrieve.rs` — `RetrieverConfig::default` / `EmptyQuery` `[VERIFIED]`
- `crates/mcp/src/server.rs`, `main.rs`, `tests/allowlist.rs` — Phase 17 stubs `[VERIFIED]`
- `crates/config/src/providers.rs` — `build_embedder` `[VERIFIED]`
- `.planning/phases/18-…/18-CONTEXT.md` — D-01..D-12 `[VERIFIED]`
- `.planning/research/{SUMMARY,ARCHITECTURE,STACK,PITFALLS}.md` — milestone guidance `[VERIFIED]`

### Secondary (MEDIUM confidence)
- docs.rs / GitHub `rmcp` `CallToolResult::success` / `error` / `structured` `[CITED: docs.rs/rmcp]`
- Cursor MCP host config (Phase 17 docs) — unchanged this phase

### Tertiary (LOW confidence)
- Exact host preference for `structured_content` vs text JSON — assume text JSON OK `[ASSUMED]`

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — reuse pinned crates; no new registry deps
- Architecture: HIGH — code seams + CONTEXT locks align; module placement recommended with clear dep graph
- Pitfalls: HIGH — mapped from milestone PITFALLS + verified agent/mcp gaps

**Research date:** 2026-07-28  
**Valid until:** 2026-08-28 (stable domain; re-check only if `rmcp` pin changes)

---

## Plan Wave Hints (for gsd-planner)

| Wave | Focus | Verify |
|------|-------|--------|
| **0 / 1** | Add `retriever::kb_readonly` + failing→passing unit tests (excerpt, Indexed filter, cap, empty query) | `cargo test -p retriever` |
| **2** | Refactor agent `search_knowledge` / `list_sources` to helpers; agent tests | `cargo test -p agent` |
| **3** | Extend `JarvisMcp` with embedder; real handlers; replace stub tests; `main` fail-closed build_embedder; mcp Cargo.toml deps | `cargo test -p mcp` |
| **4** | Update `docs/mcp.md` (remove stub; document JSON + caps + limit) | doc review + suite green |

**Discretion locks for PLAN.md:** module = `crates/retriever/src/kb_readonly.rs`; **N=200**; JSON shapes as above; include `chunk_id`/`loc`; agent refactor in Wave 2 (same phase).
