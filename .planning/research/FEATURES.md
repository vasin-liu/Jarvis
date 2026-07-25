# Feature Research

**Domain:** Related-docs (source overlap) panel + read-only MCP knowledge tools for a local-first desktop KB (Jarvis v1.11)
**Researched:** 2026-07-25
**Confidence:** HIGH

## Feature Landscape

### How related-docs / source-overlap typically works

Pattern (Curiosity “similar documents”, RAG Library side-panels, Obsidian backlinks-lite without a graph):

1. **Seed → neighbors** — User selects one indexed source; system returns other sources that are “about the same thing.”
2. **Chunk → source aggregation** — Similarity is computed on **chunks** (embeddings ± lexical), then rolled up to **source** rows so the UI lists documents, not raw chunks.
3. **Exclude self** — Never show the seed source (or its own WikiPage mirror) as a hit; looks broken if you do.
4. **Bounded list** — Top **3–8** related sources; optional score cutoff so weak neighbors stay empty rather than noisy.
5. **Explain lightly** — Title, kind label, short overlapping snippet (or “N shared themes”); prefer human labels over raw cosine floats for power users who still don’t want ML UI.
6. **Navigate** — Click → open/select that source in Library (and optionally jump to chat with that source scoped later — not required for v1.11).

**Overlap vs backlinks:** True backlinks need explicit links/`[[wikilinks]]`. Source-overlap for a RAG KB is **retrieval neighborhood** (same hybrid stack as Q&A), not a graph. Jarvis already has `retrieve` (vector + FTS → RRF); related-docs is “retrieve using seed text/embedding, group by `source_id`.”

**Jarvis stance:** Panel is a **Library companion** for discovery among already-indexed sources. No new vector DB. No graph UI. WikiPage sources may appear as neighbors when they overlap — treat like any other `SourceKind`.

### How read-only MCP knowledge tools typically work

Pattern (MCP tools spec; RAG-MCP / KnowledgeStack / KnowledgeMCP community servers; Cursor/Claude Desktop stdio):

1. **External agent hosts Jarvis as a tool server** — Client spawns a local process (stdio JSON-RPC) or connects to a local endpoint; LLM discovers tools via `tools/list`, calls via `tools/call`.
2. **Minimal read surface** — Almost every KB MCP ships **`search`** (query → ranked hits with provenance) and **`list_sources`** (inventory). Optional later: `read` / `get_chunk`. Write/ingest tools are a separate product risk.
3. **Reuse the same retrieval brain** — MCP `search` should call the same hybrid path as in-app agent `search_knowledge` / RAG (`retriever::retrieve` + Store), not a second index.
4. **Annotations** — Mark tools `readOnlyHint: true`, `destructiveHint: false` so clients can auto-approve; annotations are **hints**, not enforcement — real safety is “don’t implement mutate tools.”
5. **Local-first contract** — Open the same `kb.sqlite` (via `store` only); no cloud exfil beyond what the host LLM already does with returned text. Stdio must keep protocol on stdout (logs → stderr).
6. **Human trust** — MCP security guidance: validate inputs, sanitize outputs, rate-limit; UI/docs should make clear Jarvis is exposing **read** of the personal KB to Cursor/Claude.

**Jarvis stance:** v1.11 exposes **read-only** `search` + `list_sources` mirroring existing agent tools (`crates/agent/src/tools.rs`). No ingest, delete, memory mutate, or plugin `shell_exec` over MCP. In-app agents keep their full tool set; MCP is the **external read API**.

### Table Stakes (Users Expect These)

Features users assume exist for v1.11. Missing = milestone feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Related-docs panel on Library source selection | “Show me what else covers this” is standard KB UX | MEDIUM | Seed = selected `Source`; empty/loading/error states; `data-testid`s |
| Results are other **sources** (not only chunks) | Users navigate Library by document | MEDIUM | Aggregate chunk hits → unique `source_id`; exclude seed |
| Open / navigate to related source | Discovery without action is useless | LOW | Select in Library list / scroll-into-view |
| Hybrid-backed overlap (reuse retriever) | Trust that “related” matches how Q&A finds evidence | MEDIUM | Query from seed title + summary or top chunks; `retrieve` + rollup |
| Empty state when no neighbors | Sparse corpora must not fake relevance | LOW | Cutoff or `final_k` with empty UI copy |
| MCP `search` (hybrid query → hits + titles/uris) | External agents need one retrieval tool | MEDIUM | Thin wrapper over `retrieve` + `get_source`; cite-friendly fields |
| MCP `list_sources` (indexed inventory) | Agents need corpus map before/alongside search | LOW | Mirror agent tool: filter `IndexStatus::Indexed`; title + kind (+ uri id) |
| Read-only only (no mutate tools) | Local KB + MCP = high blast radius if writable | LOW (policy) | Explicit milestone constraint; annotate `readOnlyHint` |
| Stdio (or documented) local transport for Cursor/Claude | How desktop MCP is consumed | MEDIUM | Prefer `rmcp` stdio binary or `jarvis mcp` subcommand; path to `kb.sqlite` / app-data |
| Unit + E2E/Vitest coverage | Project rule: user-facing + IPC must be verified | MEDIUM | Panel journey in Library E2E; MCP happy path with MockEmbedder (no live LLM) |

### Differentiators (Competitive Advantage)

Not required of a generic MCP KB, but valuable *here*.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Same hybrid RRF as RAG/agent** | External Cursor agents see the same evidence neighborhood as in-app Q&A | LOW (leverage) | Do not ship vector-only MCP while UI uses hybrid |
| **Multi-kind corpus** (file, Lark, Cursor, memory, wiki) | One `list_sources` / related panel across Feishu + transcripts + notes | LOW | Kind labels already in `sourceDisplay` |
| **Panel beside mature Library** | Discovery without leaving the app users already trust | MEDIUM | LibraryView selection hook; no new nav section required |
| **MCP as thin facade over agent tools** | One implementation path; less drift | LOW–MEDIUM | Shared Rust fn used by `execute_tool` and MCP handlers |
| **Local-first + Mock CI** | Ship MCP without live providers in E2E | MEDIUM | Align with `JARVIS_E2E=1` / MockEmbedder patterns |
| Optional snippet + `loc` in MCP search results | Agents can cite like in-app citations | LOW | Match `Citation` / agent excerpt shape (~200 chars) |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| MCP write / ingest / delete / `add_memory` | “Full remote control of Jarvis” | Accidental wipe from host agents; trust collapse; violates milestone | Read-only v1.11; mutate stays in-app with UI |
| MCP `ask` / full RAG answer generation | One-shot Q&A from Cursor | Couples host LLM + Jarvis LLM; cost; non-deterministic CI; duplicates Chat | Host LLM uses `search` results itself |
| Knowledge graph / Louvain / backlink graph UI | Obsidian parity | High FE cost; out of wiki plan + PROJECT Out of Scope | Related list + export wiki to Obsidian |
| Second vector DB / Lance / HTTP-only remote MCP as default | “Proper” RAG-MCP clones | Breaks single-DB-owner; network attack surface | Same SQLite via `store`; stdio local |
| Raw cosine score as primary UI | Transparency | Users misread floats; embedding-model dependent | Rank order + optional “strong/weak” band |
| Related-docs over **all** kinds including Failed stubs | Completeness | Noise, broken open actions | Indexed-only (same as agent `list_sources`) |
| Hard WikiPage filter in related/MCP | Citation purity debates from v1.10 | Scope creep; already deferred | Soft: treat WikiPage as normal source |
| Auto-approve mutate later “because annotations” | Convenience | Annotations are untrusted hints | Never ship mutate over MCP without separate milestone + confirm UX |
| Per-chunk related panel as primary UX | “More precise” | Cognitive overload vs Library mental model | Source rollup; show one snippet per source |
| Bidirectional sync / clipper / Deep Research | Competitor checklist | Explicit later milestones (v1.12+) | Stay on overlap + read MCP |

## Feature Dependencies

```
Indexed sources in Store
    └──requires──> existing ingest/index pipeline (shipped)
    └──feeds──> list_sources (IPC + MCP + agent)

retriever::retrieve (hybrid RRF)
    └──requires──> Store search_vector + search_fts + Embedder
    └──feeds──> agent search_knowledge (shipped)
    └──feeds──> MCP search (v1.11)
    └──feeds──> related-docs rollup (v1.11)

related-docs service
    └──requires──> seed Source (+ chunks or summary text)
    └──requires──> retrieve OR source-embedding neighborhood
    └──requires──> exclude seed / optional exclude same-uri mirrors
    └──enhances──> LibraryView selection panel

Library IPC list_sources / get_source
    └──enhances──> panel open/navigate
    └──already──> commands/library.rs

MCP server (stdio)
    └──requires──> path to app-data kb.sqlite (config)
    └──requires──> build_embedder (or Mock in tests)
    └──requires──> shared search + list_sources handlers
    └──conflicts──> mutate tools (same milestone)

E2E / Vitest
    └──requires──> panel data-testids + fixture corpus with ≥2 overlapping docs
    └──requires──> MCP unit/integration with mocks (spawn or in-process)
```

### Dependency Notes

- **Related-docs requires retriever + Store:** Prefer seeding query from source title + `summary` (insights) or concatenated top chunks; embedding the whole document is optional later.
- **MCP search requires Embedder:** Same factory as app; E2E/tests use Mock — never live FastEmbed download in CI if avoidable.
- **MCP list_sources requires Store only:** Cheap; good smoke tool before search.
- **Panel requires Library selection model:** If Library has no stable “selected source” today, add selection state first (LOW) then panel.
- **Shared handlers prevent drift:** Agent `search_knowledge` / `list_sources` and MCP tools should call one crate-level API.
- **Read-only conflicts with write MCP:** Do not phase “just one write tool” into v1.11.
- **Single DB owner:** MCP process must use `store::Store::open` — no second connection layer outside the crate.

## MVP Definition

### Launch With (v1.11)

Minimum to validate “discover overlap in-app + query KB from Cursor/Claude read-only.”

- [ ] Related-docs panel for selected Library source — top-N overlapping **sources**, exclude seed, open/navigate
- [ ] Overlap backed by existing hybrid retrieve + source rollup (no new DB)
- [ ] Empty / loading / error states + `data-testid`s
- [ ] MCP tools: `search` + `list_sources` only, `readOnlyHint: true`
- [ ] Shared Rust path with agent tools (or extracted shared module)
- [ ] Documented local launch (stdio + path to user data / env)
- [ ] Tests: unit (rollup/exclude), MCP handler mocks, E2E panel visibility + navigate; no live LLM

### Add After Validation (v1.x)

- [ ] MCP `read` / get chunk by id — when agents need full text beyond excerpts
- [ ] Related-docs score cutoff settings — if noise reported on large corpora
- [ ] “Ask about these sources” deep-link into Chat with prefilled scope
- [ ] WIKI-F01 bulk/auto-compile — deferred from v1.10; separate from MCP
- [ ] Soft related-wiki preference (show WikiPage neighbors with badge) — after trust review

### Future Consideration (v2+ / later)

- [ ] Graph / Louvain UI — export to Obsidian instead
- [ ] MCP write/mutate surface — only with explicit confirm + permissions model
- [ ] Remote Streamable HTTP MCP — multi-machine; security review first
- [ ] PDF/MinerU + Deep Research skill — plan’s v1.12
- [ ] Hard WikiPage RAG citation filter — still deferred

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Library related-docs panel (source rollup) | HIGH | MEDIUM | P1 |
| Exclude seed + empty state | HIGH | LOW | P1 |
| Open/navigate related source | HIGH | LOW | P1 |
| Hybrid retrieve reuse for overlap | HIGH | MEDIUM | P1 |
| MCP `search` (hybrid + provenance) | HIGH | MEDIUM | P1 |
| MCP `list_sources` (indexed) | HIGH | LOW | P1 |
| Read-only policy + annotations | HIGH | LOW | P1 |
| Shared handlers with agent tools | HIGH | LOW–MEDIUM | P1 |
| E2E panel + MCP mock tests | HIGH | MEDIUM | P1 |
| Stdio / `rmcp` packaging docs | HIGH | MEDIUM | P1 |
| Snippet/`loc` in MCP results | MEDIUM | LOW | P2 |
| Related score cutoff config | MEDIUM | LOW | P2 |
| MCP `read` full document | MEDIUM | MEDIUM | P2 |
| Chat deep-link from related | MEDIUM | MEDIUM | P3 |
| Graph UI / write MCP / remote HTTP | LOW (Core Value) | HIGH | defer / anti |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | Curiosity / RAG UIs | Community KB MCP (RAG-MCP, KnowledgeMCP, KS-MCP) | Jarvis v1.11 approach |
|---------|---------------------|--------------------------------------------------|------------------------|
| Similar / related docs | Top-N embedding neighbors; cutoff; exclude self | Rarely a first-class desktop panel | Library panel via hybrid rollup |
| List corpus | Library / KB browser | `list_sources` / `knowledge-show` / `list_contents` | MCP `list_sources` + existing Library |
| Search | Hybrid / vector in-product | `search` / `knowledge-search` / `search_knowledge` | MCP `search` = same `retrieve` as agent |
| Read full doc | Viewer | Often `read` / `knowledge-show` | **Defer** — excerpts enough for MVP |
| Write / ingest via agent host | Usually in-app only | Many servers include add/remove | **Anti** — read-only |
| Citations / provenance | Source chips in chat | Vary; KS has cite helpers | Return title/uri/loc/excerpt like agent tools |
| Graph | Sometimes | Rare | **Out of scope** |
| Transport | N/A | stdio + sometimes HTTP | **stdio local** first |
| Trust model | In-app only | Mix of read-only vs full CRUD | Explicit read-only; local SQLite |

## Expected Behavior (local-first desktop KB)

**Related-docs panel**
1. User opens Library, selects an indexed source.
2. Panel loads related sources (async): other indexed docs whose chunks rank near the seed under hybrid retrieval.
3. Each row: title, kind, one short snippet; click selects/opens that source.
4. If corpus is tiny or orthogonal → empty state, not filler.
5. Does not mutate index, wiki, or memory.

**Read-only MCP**
1. User configures Cursor/Claude to launch Jarvis MCP against their app-data DB.
2. Host agent calls `list_sources` → sees indexed titles/kinds.
3. Host agent calls `search` with a natural-language query → ranked hits with enough provenance to cite.
4. No tool can ingest, delete, complete tasks, or write memory.
5. Failures return tool errors (`isError`) without crashing the host; empty search is a successful empty result.

## Complexity & Dependency Summary (for requirement scoping)

| Area | Complexity | Depends on (existing) | New surface |
|------|------------|----------------------|-------------|
| Source-overlap algorithm | MEDIUM | `retriever`, `store` chunks/vectors, Embedder | Rollup + exclude + optional cutoff |
| Related-docs UI | MEDIUM | `LibraryView`, `list_sources` IPC, sourceDisplay | Selection + panel + E2E |
| MCP `search` / `list_sources` | MEDIUM | Agent tools + retriever + Store + config paths | `rmcp` (or equiv) stdio binary/subcommand |
| Read-only guarantee | LOW | Milestone policy | Tool allowlist + annotations; no write handlers |
| Packaging / DX | MEDIUM | App-data layout, provider factory | Docs + stable CLI entry for hosts |

**Do not depend on for v1.11:** wiki graph, WIKI-F01 auto-compile, new vector engines, cite-filter changes, mutate MCP.

## Sources

- Project brief: `.planning/PROJECT.md` (v1.11 Related-docs + MCP; Out of Scope write MCP / graph)
- Plan deferral: `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` — Out of Scope table row v1.11
- Prior research: `.planning/research/FEATURES.md` (v1.10 wiki; listed related-docs/MCP as post-validation)
- In-repo: `crates/agent/src/tools.rs` (`search_knowledge`, `list_sources`); `crates/retriever` hybrid RRF; `src-tauri/src/commands/library.rs`; `crates/store` list/search
- MCP tools & security: [modelcontextprotocol.io — Tools](https://modelcontextprotocol.io/docs/concepts/tools) (human-in-the-loop, validation)
- MCP annotations practice: readOnlyHint / destructiveHint as UX hints, not enforcement (Salesforce / Outreach / community explainers)
- Similar-docs UX: [Curiosity Semantic Similarity](https://docs.curiosity.ai/workspace-build/ai-and-agents/ai-integrations/semantic-similarity) (top-N, cutoff, exclude seed)
- KB MCP precedents: KnowledgeStack ks-mcp (mostly read-only search/list/read); RAG-MCP (`search_documentation`, `list_sources`); KnowledgeMCP (local search + list; also ships writes — anti-pattern for Jarvis)
- Rust MCP: official `rmcp` stdio server pattern for desktop hosts

---
*Feature research for: Jarvis v1.11 Related-docs + read-only MCP*
*Researched: 2026-07-25*
