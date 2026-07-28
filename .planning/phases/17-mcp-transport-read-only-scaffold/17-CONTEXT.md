# Phase 17: MCP transport + read-only scaffold - Context

**Gathered:** 2026-07-28
**Status:** Ready for planning
**Mode:** --auto (recommended defaults from v1.11 research + ROADMAP locks)

<domain>
## Phase Boundary

Ship a **separate console stdio MCP binary** (`jarvis-mcp`) whose `tools/list` surface is **structurally read-only**: exactly `{search, list_sources}` — no write/delete/ingest/memory-mutate tools. Resolve the KB via documented data-dir / DB path. Pin **`rmcp` 2.2.0**. Only `crates/store` opens SQLite.

**This phase does NOT implement hybrid search semantics** (Phase 18 owns `kb_readonly` + real `search` / `list_sources` handlers). Phase 17 may register stub handlers under the locked tool names so allowlist tests pass.

**Out of scope:** MCP-F01 get_chunk/read, MCP-F02 Settings toggle, MCP-F03 HTTP/SSE, related-docs UI, citation-trust E2E (Phase 19).

</domain>

<decisions>
## Implementation Decisions

### Packaging & binary
- **D-01:** New workspace member **`crates/mcp`** with binary name **`jarvis-mcp`** (console subsystem — must NOT use `windows_subsystem = "windows"`). Separate from the Tauri GUI app so Cursor/Claude Desktop can spawn stdio.
- **D-02:** Pin workspace dependency **`rmcp = "2.2.0"`** with features `server`, `transport-io`, `macros` (or equivalent documented for 2.2.0). Do **not** adopt `rmcp` 3.x beta / do not bump MSRV solely for MCP.

### Read-only tool surface (MCP-04)
- **D-03:** `tools/list` MUST expose **exactly** the names `search` and `list_sources` — no other tool names in v1.11.
- **D-04:** Phase 17 registers those two tools with **stub handlers** (clear structured empty/error payload indicating implementation lands in Phase 18). Do **not** leave the tool list empty — allowlist tests need fixed names. Full hybrid semantics wait for Phase 18.

### Data-dir / DB resolution (MCP-03)
- **D-05:** Path precedence: **`--db <sqlite-file>`** > **`JARVIS_DATA_DIR`** (directory; open `kb.sqlite` inside) > default directory matching the Tauri app data location for identifier `com.jarvis.app` (document the Windows `%APPDATA%` path in docs).
- **D-06:** If the resolved DB file is **missing**, the binary **exits with a clear error** — do **not** create an empty KB / run schema init as a silent side effect of MCP startup.

### Documentation
- **D-07:** Add **`docs/mcp.md`** with Cursor + Claude Desktop stdio config snippets (`command` / `args` / `env.JARVIS_DATA_DIR`), plus a **short README pointer** to that doc. No in-app Settings toggle this phase (MCP-F02 deferred).

### Store / concurrency
- **D-08:** Enable **SQLite WAL** in `Store::open` (and in-memory open if applicable) so GUI + MCP can coexist safely. Keep `store` as the **only** module that opens SQLite connections.

### Testing bar
- **D-09:** Verify with **`cargo test -p mcp`** (and store tests if WAL changes): tool allowlist exact set, data-dir/`--db` resolution unit tests, stub handler smoke. **No WebDriver E2E for MCP** in this phase (Phase 19 / cargo harness).

### Claude's Discretion
- Exact `rmcp` 2.2.0 ServerHandler / `#[tool]` / router wiring patterns
- Stub response JSON shape (as long as tools are listed and non-mutating)
- CLI flag parsing library (`clap` vs manual) — prefer minimal deps
- Whether default AppData path helper lives in `crates/mcp` or a tiny shared util (must not open SQLite outside `store`)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase / milestone
- `.planning/ROADMAP.md` — Phase 17 goal, success criteria, MCP constraints
- `.planning/REQUIREMENTS.md` — MCP-03, MCP-04; Out of Scope (no write tools; pin rmcp 2.2.0)
- `.planning/PROJECT.md` — core value; stack constraints
- `.planning/research/SUMMARY.md` — recommended MCP approach + Phase 17 deliverables
- `.planning/research/STACK.md` — `rmcp` 2.2.0 pin, Cargo snippets, Cursor mcp.json example
- `.planning/research/ARCHITECTURE.md` — `crates/mcp` placement, WAL, store ownership
- `.planning/research/PITFALLS.md` — GUI stdio / packaging landmines (if present)

### Code
- `src-tauri/src/main.rs` — `windows_subsystem = "windows"` (why MCP must be a separate console bin)
- `src-tauri/src/state.rs` — app data dir → `kb.sqlite` layout (`config.json`, skills, …)
- `crates/store/src/store.rs` — `Store::open` (sole SQLite owner; add WAL here)
- `Cargo.toml` — workspace members / MSRV 1.85

### External
- [Cursor MCP docs](https://cursor.com/docs/mcp) — stdio host config shape
- crates.io `rmcp@2.2.0` — stay on 2.x until intentional MSRV ≥ 1.88

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Store::open(path, dim)` — MCP opens the same `kb.sqlite` the GUI uses; dim must match embedder config (stub phase may open read path with configured/default dim — researcher confirms)
- App data layout in `state.rs`: `app_data_dir()/kb.sqlite` under Tauri identifier `com.jarvis.app`
- Workspace crate pattern: `crates/*` path deps + `{ workspace = true }`

### Established Patterns
- Thin binaries / fat crates; no SQLite outside `store`
- Trait injection + mocks for tests; MCP tests stay offline
- GUI release hides console via `windows_subsystem` — MCP binary must not inherit that

### Integration Points
- New workspace member + bin target `jarvis-mcp`
- Future Phase 18: stub handlers → `kb_readonly` shared with `crates/agent` tools
- Docs consume install path of `jarvis-mcp.exe` for host config

</code_context>

<specifics>
## Specific Ideas

[--auto] Selected all gray areas: Packaging & binary, Read-only tool surface, Data-dir / DB resolution, Documentation, Store / concurrency (WAL), Testing bar.

[auto] Packaging — Q: "Where does the MCP binary live?" → Selected: "crates/mcp + jarvis-mcp console bin" (recommended; research Option A)
[auto] Tool stubs — Q: "What does tools/list expose in Phase 17?" → Selected: "Exactly search + list_sources with stub handlers" (recommended for MCP-04)
[auto] Path resolution — Q: "How to find kb.sqlite?" → Selected: "--db > JARVIS_DATA_DIR > default AppData; fail if missing" (recommended)
[auto] Docs — Q: "Where to document host config?" → Selected: "docs/mcp.md + README pointer" (recommended)
[auto] WAL — Q: "GUI+MCP concurrency?" → Selected: "Enable WAL in Store::open" (recommended)
[auto] Settings toggle — Q: "In-app enable?" → Selected: "No — docs only; MCP-F02 deferred" (recommended)
[auto] Tests — Q: "Verification layer?" → Selected: "cargo test -p mcp; no WebDriver MCP E2E this phase" (recommended)

</specifics>

<deferred>
## Deferred Ideas

- **Phase 18:** Real `search` / `list_sources` via shared `kb_readonly`; agent tool alignment (MCP-01, MCP-02)
- **Phase 19:** MCP happy-path cargo harness + related-docs E2E regression / citation trust
- **MCP-F01:** `get_chunk` / `read` document tools
- **MCP-F02:** Settings toggle to enable/disable MCP server
- **MCP-F03:** HTTP/SSE local transport
- Empty-DB auto-create on MCP start (rejected for D-06)

</deferred>

---
*Phase: 17-mcp-transport-read-only-scaffold*
*Context gathered: 2026-07-28 via /gsd-discuss-phase 17 --auto*
