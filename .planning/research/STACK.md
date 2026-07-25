# Stack Research

**Domain:** Local-first knowledge hub — Related-docs panel + read-only MCP (v1.11)
**Researched:** 2026-07-25
**Confidence:** HIGH (versions via `cargo info` / crates.io API; transport pattern via official MCP + Cursor docs)

> **Scope:** Stack *additions/changes* for v1.11 only. Core Tauri/React/SQLite stack unchanged. Wiki (v1.10) history preserved in [Appendix A](#appendix-a--v110-wiki-compile-stack-history).

## Recommended Stack

### Core Technologies (preserved — do not replace)

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust stable | MSRV **1.85** | Domain logic + MCP binary | Workspace pin; do **not** bump solely for `rmcp` 3.x beta |
| Tauri 2 | workspace pin | Desktop shell / **related-docs IPC** | Panel talks to Rust via `invoke` only |
| React 19 + Vite 7 | package.json | Related-docs panel UI | Existing FE; no new UI framework |
| SQLite + FTS5 + sqlite-vec | rusqlite **0.32** | Overlap queries + MCP `list_sources` / search | Single DB owner via `store` |
| `retriever` + RRF | existing crate | MCP `search` backend | Reuse hybrid search; do not reimplement |
| `Embedder` / Mock | crates/embedder | MCP search embeddings | Trait injection + E2E/CI mocks |
| Tokio 1.x | workspace | MCP server async runtime | Already required; `rmcp` is tokio-native |

### Supporting Libraries (NEW for v1.11)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **`rmcp`** | **`2.2.0`** (pin exact; latest **stable**) | Official MCP Rust SDK — server + tool macros | MCP binary only. Features: `server`, `transport-io` (stdio). Pulled macros via default/`macros` |
| `rmcp-macros` | **`2.2.0`** (transitive) | `#[tool]` / `#[tool_router]` | Do not pin separately unless version resolution forces it |
| `schemars` | **1.x** (transitive via `rmcp` `server`) | JSON Schema for tool args | Prefer transitive; add workspace pin only if mcp crate defines args structs needing it directly |
| Hand-rolled related-docs query | — | Source overlap ranking | SQL / chunk-neighbor logic in `store` or thin `retriever` helper — **no** graph crate |

**Version gate (critical):** crates.io newest `rmcp` is **`3.0.0-beta.2`** with **`rust-version = 1.88`**. That **breaks** Jarvis MSRV 1.85. Prefer stable **`2.2.0`** (edition 2024, compatible with rustc ≥ 1.85; published 2026-07-08). Revisit `rmcp` 3.x only after an intentional MSRV bump.

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `cargo test -p store` / `-p retriever` | Related-docs overlap unit/integration | tempfile DB + fixtures |
| `cargo test -p mcp` (or new crate name) | MCP tool handlers with `MockEmbedder` | No live LLM; optional in-process stdio client test |
| Vitest | Panel helpers / `data-testid` wiring if extracted | Existing `npm test` |
| WebdriverIO + `JARVIS_E2E=1` | Related-docs panel visibility + navigate | Extend Library-focused or new `e2e/specs/related-docs.spec.ts`; MCP happy path can stay **cargo**-level (stdio), not WebDriver |

## Installation

```toml
# workspace Cargo.toml — members
# add: "crates/mcp"   # suggested name: console binary `jarvis-mcp`

# [workspace.dependencies]
rmcp = { version = "2.2.0", default-features = false, features = ["server", "transport-io", "macros"] }

# crates/mcp/Cargo.toml
# rmcp = { workspace = true }
# store / retriever / embedder / config / tokio / serde / serde_json / thiserror = path or workspace
```

```bash
# No new npm packages for related-docs UI (React + @tauri-apps/api + Tabler already present).

# Host config example (Cursor / Claude Desktop) — document in README, not a dependency:
# {
#   "mcpServers": {
#     "jarvis": {
#       "command": "path/to/jarvis-mcp.exe",
#       "args": [],
#       "env": { "JARVIS_DATA_DIR": "%APPDATA%/com.jarvis.app" }
#     }
#   }
# }
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `rmcp` **2.2.0** | `rmcp` **3.0.0-beta.2** | Only after bumping workspace MSRV to **≥ 1.88** and accepting pre-release |
| `rmcp` (official) | `rust-mcp-sdk` **0.10.0** (MSRV 1.80) | Fallback if `rmcp` 2.x proves incompatible in practice; community, not official |
| Separate **stdio** binary | Embed MCP inside Tauri GUI process | Never for stdio — release GUI uses `windows_subsystem = "windows"` (no usable stdio for hosts) |
| stdio (`transport-io`) | Streamable HTTP / SSE (`transport-streamable-http-server`) | Later if multi-client remote access is required; adds auth/surface area; not needed for Cursor/Claude local spawn |
| Tauri IPC for panel | Frontend-only overlap heuristic | Overlap must use indexed chunks/embeddings — keep logic in Rust |
| SQL / retrieve-based overlap | Neo4j / petgraph / Louvain crates | Out of scope (graph UI deferred); overlap ≠ full knowledge graph |
| Reuse `agent::execute_tool` | Duplicate search/list in MCP | Prefer thin shared helpers calling `retriever::retrieve` + `Store::list_sources` — MCP tool names stay `search` / `list_sources` |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `rmcp` **3.x beta** under MSRV 1.85 | Declared rustc **1.88** — build break | Pin **`rmcp` 2.2.0** |
| MCP **write / mutate** tools | Explicit v1.11 non-goal; trust / data-loss risk | Read-only `search` + `list_sources` only |
| Graph UI libs (`react-force-graph`, cytoscape, etc.) | Scope creep; deferred | Simple related list in Library panel |
| LanceDB / second vector store | Dual index; breaks single-store rule | Existing sqlite-vec + FTS |
| `axum` / HTTP MCP in-process for v1.11 | Extra deps + open port + auth story | stdio binary spawned by host |
| New npm MCP client SDKs in the React app | Panel is in-app UX, not an MCP host | `@tauri-apps/api` `invoke` |
| Hand-rolled JSON-RPC MCP protocol | Spec drift; Cursor/Claude expect SDK-compliant servers | Official `rmcp` |
| Live LLM / live FastEmbed downloads in MCP CI | Flaky / slow | `MockEmbedder` + fixture DB |
| Opening SQLite from MCP without `store` | Schema drift | `store::Store::open` only |
| Bumping workspace `thiserror` to 2 solely for `rmcp` | Unnecessary churn (`rmcp` brings `thiserror` ^2 transitively) | Leave workspace at `thiserror` 1 unless crates adopt 2 deliberately |

## Stack Patterns by Variant

**Related-docs panel (in-app):**
- New thin Tauri command (e.g. `list_related_sources { sourceId, limit }`) → overlap algorithm in `store`/`retriever`.
- React Library detail (or chat citation context) calls `invoke`; Tabler icons for list rows; stable `data-testid`s for E2E.
- No new crates unless overlap logic outgrows a module — prefer `store` method + unit tests first.

**Read-only MCP (external hosts):**
- New **console** workspace binary (suggested: `crates/mcp` → `jarvis-mcp`), **not** the Tauri GUI target.
- Resolve DB via env (`JARVIS_DATA_DIR` / explicit `--db`) pointing at the same `kb.sqlite` as the app (`app_data_dir`).
- Build embedder from `config` (respect Mock when `JARVIS_E2E=1` or a dedicated MCP test flag).
- Tools: `search` → `retriever::retrieve`; `list_sources` → `Store::list_sources` (indexed filter matches agent tool semantics).
- Ship with docs snippet for Cursor `mcp.json` / Claude Desktop config.

**If MSRV is later raised to ≥ 1.88:**
- Re-evaluate `rmcp` 3.x stable when non-beta; still keep features minimal (`server` + `transport-io`).

**If hosts require HTTP later:**
- Add `transport-streamable-http-server` behind a flag; default remains stdio for local desktop.

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| `rmcp` **2.2.0** | Rust **≥ 1.85** (edition 2024) | Fits current MSRV; **do not** take 3.0.0-beta.2 |
| `rmcp` 2.2 `server` | `schemars` **1.x**, `tokio` **1.x** | Aligns with workspace tokio |
| `rmcp` (optional HTTP features) | `reqwest` **^0.13** | Avoid enabling HTTP client features to prevent dual reqwest 0.12/0.13 pressure unless needed |
| MCP binary | `store` + `retriever` + `embedder` + `config` | Same paths as GUI app data dir |
| Related-docs IPC | Existing `@tauri-apps/api` **^2** | No FE dep bump |
| Wiki `zip` **7.2** (workspace) | MSRV 1.85 | Already shipped in v1.10; unchanged by v1.11 |

## Integration Points (for roadmap)

| Capability | Stack touch | Reuse |
|------------|-------------|-------|
| Related-docs UI | `src/` Library view + `src-tauri` command module | `list_sources` patterns, `sourceDisplay` labels |
| Overlap ranking | `crates/store` and/or `crates/retriever` | Chunks / FTS / vector neighbors — no new DB |
| MCP `search` | New mcp binary → `retriever::retrieve` | Same as agent `search_knowledge` guts |
| MCP `list_sources` | New mcp binary → `Store::list_sources` | Same as agent `list_sources` |
| Tests | cargo + optional E2E panel | Mocks; never live Feishu/LLM |

## Sources

- crates.io / `cargo info rmcp@2.2.0` — stable **2.2.0**; newest **3.0.0-beta.2** rust-version **1.88** (verified 2026-07-25) — confidence HIGH
- crates.io / `cargo info rust-mcp-sdk` — **0.10.0**, rust-version **1.80** (community alternative) — confidence HIGH
- [modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk) — official SDK; stdio via `(stdin, stdout)` / `transport-io` — confidence HIGH
- [Cursor MCP docs](https://cursor.com/docs/mcp) — stdio `command`/`args`/`env` host config — confidence MEDIUM (web + official docs)
- Workspace: root `Cargo.toml`, `src-tauri/src/main.rs` (`windows_subsystem`), `src-tauri/src/state.rs` (`kb.sqlite`), `crates/agent/src/tools.rs` — confidence HIGH
- Prior STACK (v1.10 wiki) — see Appendix A

---

## Appendix A — v1.10 Wiki Compile Stack History

*(Preserved from research 2026-07-17; do not erase. Shipped; versions may have drifted.)*

### Supporting Libraries (v1.10)

| Library | Version (research-time) | Purpose | Notes (2026-07-25) |
|---------|-------------------------|---------|-------------------|
| `zip` | researched **2.4.2**; avoid pre | Obsidian vault zip | Workspace now pins **`zip` 7.2** (`deflate`) — keep current pin |
| `walkdir` | **2.5.0** | Walk `wiki/` tree | Optional; shallow tree may use `std::fs` |
| Hand-rolled slugify / YAML frontmatter | — | Paths + frontmatter | Still prefer over deprecated `serde_yaml` |
| `tempfile` | **3.x** | Export/unit tests | Unchanged |

### What NOT to Use (v1.10 — still valid)

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| LanceDB / graph UI / Louvain | Dual index / scope | sqlite-vec + Obsidian export |
| `serde_yaml` | Deprecated upstream | `format!` / small helper |
| Bidirectional Obsidian sync | Merge complexity | Export-only zip |

---
*Stack research for: Related-docs + read-only MCP (Jarvis v1.11)*
*Researched: 2026-07-25*
