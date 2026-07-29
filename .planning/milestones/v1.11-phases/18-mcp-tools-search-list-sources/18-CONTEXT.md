# Phase 18: MCP tools search + list_sources - Context

**Gathered:** 2026-07-28
**Status:** Ready for planning
**Mode:** --auto (recommended defaults from v1.11 research + Phase 17 locks + agent tool patterns)

<domain>
## Phase Boundary

Replace Phase 17 **stub** MCP handlers with **real** read-only KB tools so external hosts get the **same hybrid search and Indexed source inventory semantics** as in-app agent tools. Extract a shared **`kb_readonly`** (or equivalent) API that both `agent::search_knowledge` / `agent::list_sources` and MCP `search` / `list_sources` call — **no forked retrieval**.

**In scope:** MCP-01, MCP-02; shared helper extraction; payload bounds; MCP embedder wiring from config; `cargo test -p mcp` + agent tests for shared helpers; update `docs/mcp.md` stub note.

**Out of scope:** New MCP tool names (MCP-04 stays `{search, list_sources}`); write/mutate tools; MCP-F01 get_chunk/read; MCP-F02 Settings; MCP-F03 HTTP/SSE; WebDriver MCP E2E / citation trust gate (Phase 19); related-docs UI changes.

</domain>

<decisions>
## Implementation Decisions

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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase / milestone
- `.planning/ROADMAP.md` — Phase 18 goal, success criteria, MCP-01/MCP-02
- `.planning/REQUIREMENTS.md` — MCP-01, MCP-02; Out of Scope (no write tools)
- `.planning/PROJECT.md` — core value; stack constraints
- `.planning/research/SUMMARY.md` — Phase 18 deliverables; path-redaction product note; payload caps
- `.planning/research/ARCHITECTURE.md` — `kb_readonly` placement; agent ↔ MCP sharing; data flow
- `.planning/research/STACK.md` — MCP search → `retriever::retrieve`
- `.planning/research/PITFALLS.md` — Pitfall 6 unbounded payloads; Pitfall 9 semantic drift; excerpt ~200
- `.planning/phases/17-mcp-transport-read-only-scaffold/17-CONTEXT.md` — D-01..D-09 transport locks (do not reopen)
- `.planning/phases/17-mcp-transport-read-only-scaffold/17-VERIFICATION.md` — stub → Phase 18 handoff

### Code
- `crates/agent/src/tools.rs` — `search_knowledge` / `list_sources` (Indexed filter, retrieve, ~200 excerpt / citations)
- `crates/retriever/src/retrieve.rs` — `retrieve`, `RetrieverConfig::default`
- `crates/mcp/src/server.rs` — stub `JarvisMcp` tools to replace
- `crates/mcp/src/main.rs` — Store open + stdio; extend for embedder
- `crates/config/src/providers.rs` — `build_embedder`
- `docs/mcp.md` — host config; update stub status

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `retriever::retrieve` + `RetrieverConfig::default()` — hybrid RRF path already used by agent
- `Store::list_sources` + `IndexStatus::Indexed` filter in agent tools
- Phase 17 `JarvisMcp` + `SearchParams` + allowlist tests — replace stub bodies
- `config::load_config` + `build_embedder` for MCP process (GUI parity)
- `MockEmbedder` for offline cargo tests

### Established Patterns
- Thin MCP handlers; fat crates for domain logic
- Trait injection (`Embedder`) for test doubles
- Single SQLite owner (`store`); WAL already enabled (Phase 17)
- Agent returns Chinese text + `Citation`; MCP should return JSON without forking retrieve

### Integration Points
- Refactor `agent::execute_tool` branches for `search_knowledge` / `list_sources` → shared helper
- `JarvisMcp::search` / `list_sources` → same helper → JSON
- `main.rs` must construct embedder before serve
- Docs + allowlist tests update

</code_context>

<specifics>
## Specific Ideas

[--auto] Selected all gray areas: Shared kb_readonly placement, Search top-k/excerpt, list_sources Indexed+caps, Path privacy, Embedder wiring, Response shape, Testing bar.

[auto] Shared API — Q: "Where does kb_readonly live?" → Selected: "Shared structured helpers callable by agent + mcp without mcp→agent tool-loop dep" (recommended; research Architecture)
[auto] Search — Q: "Same hybrid path / k?" → Selected: "retriever::retrieve + RetrieverConfig::default(); clamp optional limit" (recommended)
[auto] Excerpts — Q: "Truncate how?" → Selected: "~200 chars like agent citations" (recommended; Pitfall 6)
[auto] list_sources — Q: "Which sources / fields?" → Selected: "Indexed only; id+title+kind; hard cap; no absolute paths" (recommended)
[auto] Paths — Q: "Expose file://?" → Selected: "No absolute paths in MCP payloads" (recommended privacy)
[auto] Embedder — Q: "How MCP embeds?" → Selected: "config::build_embedder from sibling config.json; Mock in tests" (recommended)
[auto] Response — Q: "Agent text vs MCP JSON?" → Selected: "Structured shared core; MCP JSON / agent text formatters" (recommended)
[auto] Tests — Q: "Verification?" → Selected: "cargo test -p mcp + agent; no WebDriver this phase" (recommended; D-09 Phase 17 continuity)

</specifics>

<deferred>
## Deferred Ideas

- **Phase 19:** MCP happy-path cargo harness + related-docs E2E / citation trust regression
- **MCP-F01:** `get_chunk` / `read` document tools
- **MCP-F02:** Settings toggle for MCP
- **MCP-F03:** HTTP/SSE transport
- Agent-visible URI in `list_sources` text (not requested; keep omit)
- Changing RAG ask / citation fusion for MCP excerpts

</deferred>

---

*Phase: 18-mcp-tools-search-list-sources*
*Context gathered: 2026-07-28*
