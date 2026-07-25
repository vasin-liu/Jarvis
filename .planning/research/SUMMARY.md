# Project Research Summary

**Project:** Jarvis (v1.11 Related-docs + MCP)
**Domain:** Local-first personal AI knowledge hub — source-overlap discovery + read-only MCP KB tools
**Researched:** 2026-07-25
**Confidence:** HIGH

## Executive Summary

Jarvis v1.11 adds two **read side-channels** over the existing hybrid RAG stack: (1) a Library **related-docs panel** that ranks other indexed sources near a selected seed via retrieve → source rollup, and (2) a **stdio MCP binary** exposing only `search` and `list_sources` so Cursor/Claude can query the same `kb.sqlite` without mutate tools. Experts build this as thin consumers of one retrieval brain—not a second vector DB, not a graph UI, and not an in-process stdio server inside the Windows GUI subsystem binary.

**Recommended approach:** Keep Tauri/React/SQLite unchanged. Add `retriever::related_sources` (summary/title/chunk text → hybrid RRF → aggregate by `source_id`, exclude seed). Extract a shared `kb_readonly` helper so agent tools and MCP never diverge. Ship a new `crates/mcp` console binary with official **`rmcp` 2.2.0** (stdio/`transport-io`)—**not** `rmcp` 3.x beta (MSRV 1.88). Panel talks via thin `list_related_sources` IPC; MCP opens Store via env/`JARVIS_DATA_DIR` pointing at app-data. Tests stay MockEmbedder + fixtures; panel gets WebDriver E2E, MCP stays cargo-level.

**Key risks:** treating `readOnlyHint` as a security boundary (must be a structural allowlist); SQLite lock contention / second connection outside `store`; garbage overlap UI that erodes trust; citation/RAG regressions from forked retrieve or changed defaults. Mitigate with compile-time tool registry, short Store locks (+ consider WAL), rank-based overlap with empty-preferring thresholds, shared retrieve helpers, and a Phase 19 qa/full-ui citation gate.

## Key Findings

### Recommended Stack

Core stack is **preserved** (Rust MSRV 1.85, Tauri 2, React 19, SQLite + FTS5 + sqlite-vec, existing `retriever` / `Embedder`). The only intentional addition is **`rmcp` 2.2.0** for a separate stdio MCP binary. No new npm packages for the panel. Do not bump MSRV solely for `rmcp` 3 beta; do not embed MCP stdio in the GUI exe; do not add graph crates or a second vector store. Details: [STACK.md](./STACK.md).

**Core technologies:**
- **`rmcp` 2.2.0** (`server` + `transport-io` + `macros`): official MCP Rust SDK for stdio sidecar — fits MSRV 1.85; avoid 3.0.0-beta.2
- **Existing `retriever` + RRF / `Store`**: related-docs overlap + MCP `search` / `list_sources` backends — no reimplementation
- **Separate console binary (`jarvis-mcp`)**: Cursor/Claude spawn child with usable stdio; GUI uses `windows_subsystem = "windows"`
- **MockEmbedder + `JARVIS_E2E=1`**: CI/E2E without live LLM or FastEmbed downloads

### Expected Features

Table stakes are a Library related-docs panel (source-level neighbors, exclude seed, navigate, empty/loading/error) backed by hybrid retrieve, plus read-only MCP `search` + `list_sources` over stdio with documented data-dir config and offline tests. Differentiators are same-RRF-as-RAG semantics, multi-kind corpus labels, and a thin facade over agent tools. Anti-features: write MCP, MCP `ask`, graph UI, second vector DB, raw cosine % UI. Details: [FEATURES.md](./FEATURES.md).

**Must have (table stakes):**
- Related-docs panel on Library source selection — top-N overlapping **sources**, exclude seed, open/navigate
- Hybrid-backed overlap (reuse `retrieve` + rollup) — empty state when no neighbors
- MCP `search` + `list_sources` only, `readOnlyHint: true` (policy + structural allowlist)
- Stdio local transport + docs for Cursor/Claude; unit + E2E/Vitest with mocks

**Should have (competitive):**
- Same hybrid RRF as in-app RAG/agent — no vector-only MCP fork
- Shared Rust handlers with agent `search_knowledge` / `list_sources`
- Multi-kind corpus (file, Lark, Cursor, memory, wiki) with existing kind labels
- Optional snippet/`loc` in MCP results (P2)

**Defer (v2+ / later milestones):**
- MCP write/mutate, MCP `read` full doc, remote HTTP MCP, graph/Louvain UI, Chat deep-link from related, WIKI-F01 auto-compile, hard WikiPage citation filter

### Architecture Approach

Related-docs and MCP are two consumers of the same hybrid retrieval + store list surface; neither owns SQLite nor mutates index/write paths. Place overlap in `retriever::related_sources`; shared `kb_readonly` for agent + MCP; thin Tauri IPC; new `crates/mcp` stdio binary (Option A host placement). Library needs FE selection state (none today). Prefer WAL on `Store::open` for GUI+MCP coexistence. Do not change RAG ask / citation fusion. Details: [ARCHITECTURE.md](./ARCHITECTURE.md).

**Major components:**
1. **`retriever::related_sources`** — seed → query text → hybrid retrieve → aggregate by `source_id` → exclude seed → top-N
2. **`kb_readonly` (shared)** — canonical `search` + Indexed `list_sources` for agent tools and MCP
3. **Tauri `list_related_sources` + LibraryView panel** — thin IPC; selection + navigate; `data-testid`s
4. **`crates/mcp` stdio binary** — `rmcp` ServerHandler; tools allowlist `{search, list_sources}` only
5. **`store` (sole SQLite owner)** — unchanged open policy; optional WAL for concurrent MCP+GUI

### Critical Pitfalls

Top risks from [PITFALLS.md](./PITFALLS.md):

1. **"Read-only" only via hints** — structural allowlist `{search, list_sources}`; never wrap full `agent::execute_tool`; unit-test tools/list
2. **Second SQLite connection / long lock holds** — only `Store::open`; unlock before embed; cap work; consider WAL
3. **Overlap scoring garbage** — multi-signal rank-based overlap; exclude self; empty > noise; no fake % cosine
4. **E2E needing live LLM / live IDE clients** — MockEmbedder; cargo MCP harness; WebDriver only for panel
5. **Citation / RAG trust regressions** — no changes to `rag::ask` defaults; shared retrieve; Phase 19 qa/full-ui gate
6. **Unbounded MCP payloads / non-loopback HTTP** — cap list/search; prefer stdio; no remote MCP in v1.11

## Implications for Roadmap

Phase numbering continues after v1.10 (phases 07–14). Suggested **v1.11 phases start at 15**. Related-docs and MCP share Phase 15 foundations then can parallelize (UI 16 vs MCP 17–18) before a joint gate at 19.

### Phase 15: Overlap scoring API (`related_sources`)
**Rationale:** All UI and quality depend on correct, tested rollup; no MCP or FE blocked correctly without this.
**Delivers:** `retriever::related_sources` (+ unit/integration with MockEmbedder); seed query from summary → chunk text; exclude seed; top-N + empty-preferring threshold; short Store lock scopes.
**Addresses:** Hybrid-backed overlap; exclude seed; empty when no neighbors (table stakes P1).
**Avoids:** Overlap garbage (Pitfall 3); embed-under-mutex / second SQLite owner (Pitfall 2); RAG default changes (Pitfall 5).
**Verify:** `cargo test -p retriever` — related vs unrelated fixtures.

### Phase 16: Related-docs Library panel UI
**Rationale:** User-facing half can ship independently of MCP once Phase 15 API exists; Library currently has no selection model.
**Delivers:** Library selection state; `list_related_sources` IPC; related panel (title, kind, snippet); loading/empty/error + `data-testid`s; navigate/open; Vitest helpers as needed.
**Addresses:** Panel + navigate (P1); honest UI (no raw cosine %).
**Uses:** Existing React/Tauri/`sourceDisplay`; no new npm deps.
**Implements:** Thin IPC + LibraryView panel (Architecture Patterns 1 + 3).
**Avoids:** Graph scope creep (Pitfall 10); UI stale/empty/error gaps (Pitfall 8); silent inject into ask context.
**Verify:** Vitest + focused IPC smoke; E2E can start here or complete in 19.

### Phase 17: MCP transport + structural read-only scaffold
**Rationale:** Transport and allowlist must exist before tool bodies; separates “can’t mutate” from “tools work.”
**Delivers:** Workspace `crates/mcp` + stdio bin (`jarvis-mcp`); pin `rmcp` 2.2.0; register **only** empty/stub tools or registry with fixed names; data-dir resolution (`JARVIS_DATA_DIR` / `--db`); docs sketch; optional Settings “MCP” copy / default-safe enable story; strongly consider WAL on `Store::open`.
**Addresses:** Stdio packaging (P1); read-only policy foundation.
**Avoids:** Hint-only read-only (Pitfall 1); transport exposure (Pitfall 7); GUI-in-process stdio (Anti-pattern 6); MCP auto-enable surprise (Pitfall 11).
**Verify:** `tools/list` === `{search, list_sources}` unit test; no mutate names; no `rusqlite` outside `store`.

### Phase 18: MCP tools `search` + `list_sources` (shared helpers)
**Rationale:** Tools must wrap the same APIs as agent tools after scaffold; extract `kb_readonly` here (or end of 17) so semantics can’t drift.
**Delivers:** Shared `kb_readonly` used by agent `search_knowledge` / `list_sources` and MCP handlers; hybrid `search` + Indexed `list_sources`; excerpt/top-k caps; `cargo test -p mcp` + `cargo test -p agent`; payload size bounds.
**Addresses:** MCP search/list (P1); shared handlers (differentiator); snippet/`loc` if cheap (P2).
**Avoids:** Divergent search semantics (Pitfall 9); unbounded payloads (Pitfall 6); wrapping full agent tool loop (debt table).
**Verify:** Golden fixture hit-ordering shared across agent + MCP; output length caps.

### Phase 19: E2E gate + citation/RAG regression
**Rationale:** User-facing panel + Core Value protection require a joint ship gate; MCP protocol stays cargo-verified.
**Delivers:** `related-docs` (or Library) E2E journey with fixtures (≥2 overlapping docs); update e2e-required spec map; re-run `qa.spec` / `full-ui` / wiki citation trust; offline `JARVIS_E2E=1`; README Cursor `mcp.json` snippet; “looks done” checklist from PITFALLS.
**Addresses:** E2E/Vitest coverage (P1); citation trust preservation.
**Avoids:** Live LLM E2E (Pitfall 4); shipping with citation regressions (Pitfall 5); missing testids/spec map (Pitfall 12).
**Verify:** `npm run test:e2e:local` green for related-docs + existing qa/full-ui; MCP cargo suite green.

### Phase Ordering Rationale

- **API before UI before E2E** — overlap quality is the product; panel without good scoring ships distrust.
- **MCP scaffold (17) before tool bodies (18)** — structural allowlist first so “reuse agent tools” cannot accidentally register writes.
- **UI (16) ∥ MCP (17–18) after Phase 15** — architecture explicitly allows parallel tracks; integrate at Phase 19.
- **Do not** start Settings/MCP UX before shared search works; **do not** block related-docs on the MCP binary.
- **Citation gate last** — Core Value regressions are ship blockers, not nice-to-haves.

### Research Flags

Phases likely needing deeper research during planning (`/gsd-plan-phase --research`):
- **Phase 17:** Exact `rmcp` 2.2.0 ServerHandler/`#[tool]` patterns + Windows stdio packaging; WAL vs busy_timeout when GUI+MCP concurrent (MEDIUM confidence today).
- **Phase 15 (light):** Seed-query composition (summary vs chunk concat) and WikiPage/Memory demotion policy — product choice, not stack.

Phases with standard patterns (skip deep research-phase):
- **Phase 16:** Existing Library IPC + FE patterns; selection + panel is conventional Tauri/React work.
- **Phase 18:** Thin wrappers over known `retriever` / `Store::list_sources` / agent tools.
- **Phase 19:** Established E2E harness + Mock providers; follow e2e-required map.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | `rmcp` 2.2.0 vs 3.x MSRV verified via crates.io; GUI stdio limitation from codebase |
| Features | HIGH | Aligns with PROJECT.md + competitor/MCP precedents; clear anti-features |
| Architecture | HIGH (integration) / MEDIUM (MCP host) | Codebase spot-check strong; WAL/contention needs implement-time validation |
| Pitfalls | HIGH (invariants) / MEDIUM (MCP security ecosystem) | Store ownership + E2E rules solid; annotation/auth patterns from community |

**Overall confidence:** HIGH

### Gaps to Address

- **WAL enablement:** Strongly recommended for GUI+MCP; confirm migration safety / busy_timeout during Phase 17 planning.
- **WikiPage / Memory in related list:** Soft label vs demote vs exclude — decide in Phase 15/16; do not hard-filter RAG citations.
- **MCP enable UX:** Stdio is client-spawned (may need docs only) vs Settings toggle if any long-running side server appears — keep default safe.
- **Path redaction:** Whether MCP `list_sources` redacts absolute `file://` paths to basename — product decision in Phase 18.
- **`rmcp` pin at implement time:** Re-check crates.io; stay on 2.x until intentional MSRV ≥ 1.88.

## Sources

### Primary (HIGH confidence)

- crates.io / `cargo info rmcp@2.2.0` — stable pin; 3.0.0-beta.2 rust-version 1.88
- [modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk) — official stdio server transport
- Workspace: `Cargo.toml`, `src-tauri/src/main.rs`, `state.rs`, `crates/agent/src/tools.rs`, `crates/retriever`, `crates/store`
- `.planning/PROJECT.md` — v1.11 goals and out-of-scope write MCP / graph
- `.cursor/rules/e2e-required.mdc`, `AGENTS.md` — E2E + single-DB-owner invariants
- Research files: [STACK.md](./STACK.md), [FEATURES.md](./FEATURES.md), [ARCHITECTURE.md](./ARCHITECTURE.md), [PITFALLS.md](./PITFALLS.md)

### Secondary (MEDIUM confidence)

- [Cursor MCP docs](https://cursor.com/docs/mcp) — stdio `command`/`args`/`env` host config
- [modelcontextprotocol.io — Tools](https://modelcontextprotocol.io/docs/concepts/tools) — annotations advisory; validation guidance
- Curiosity semantic similarity docs — top-N, cutoff, exclude seed UX
- Community KB MCP servers (RAG-MCP, KnowledgeMCP, KnowledgeStack) — search/list patterns; write tools as anti-pattern
- MCP security writeups (non-loopback binds, over-privileged tools) — ecosystem

### Tertiary (LOW confidence)

- Exact concurrent SQLite behavior under dual-process without WAL — validate in Phase 17/19 stress
- Whether Settings MCP toggle is required for stdio-only — confirm with packaging decision

---
*Research completed: 2026-07-25*
*Ready for roadmap: yes*
