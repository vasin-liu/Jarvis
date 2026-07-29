# Phase 17: MCP transport + read-only scaffold - Research

**Researched:** 2026-07-28
**Domain:** stdio MCP sidecar binary (`jarvis-mcp`), structural read-only tool allowlist, KB path resolution, SQLite WAL for GUI+MCP coexistence
**Confidence:** HIGH (CONTEXT D-01..D-09 + codebase paths); MEDIUM (`rmcp` 2.2.0 macro wiring — verified via docs.rs + upstream README)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Packaging & binary
- **D-01:** New workspace member **`crates/mcp`** with binary name **`jarvis-mcp`** (console subsystem — must NOT use `windows_subsystem = "windows"`). Separate from the Tauri GUI app.
- **D-02:** Pin workspace dependency **`rmcp = "2.2.0"`** with features `server`, `transport-io`, `macros`. Do **not** adopt `rmcp` 3.x beta / do not bump MSRV solely for MCP.

#### Read-only tool surface (MCP-04)
- **D-03:** `tools/list` MUST expose **exactly** the names `search` and `list_sources` — no other tool names in v1.11.
- **D-04:** Phase 17 registers those two tools with **stub handlers** (clear structured empty/error payload indicating implementation lands in Phase 18). Do **not** leave the tool list empty.

#### Data-dir / DB resolution (MCP-03)
- **D-05:** Path precedence: **`--db <sqlite-file>`** > **`JARVIS_DATA_DIR`** (directory; open `kb.sqlite` inside) > default directory matching Tauri identifier `com.jarvis.app` (document Windows `%APPDATA%` path in docs).
- **D-06:** If the resolved DB file is **missing**, the binary **exits with a clear error** — do **not** create an empty KB / run schema init as a silent side effect of MCP startup.

#### Documentation
- **D-07:** Add **`docs/mcp.md`** with Cursor + Claude Desktop stdio config snippets (`command` / `args` / `env.JARVIS_DATA_DIR`), plus a **short README pointer**. No in-app Settings toggle (MCP-F02 deferred).

#### Store / concurrency
- **D-08:** Enable **SQLite WAL** in `Store::open` (and in-memory open if applicable) so GUI + MCP can coexist safely. Keep `store` as the **only** module that opens SQLite connections.

#### Testing bar
- **D-09:** Verify with **`cargo test -p mcp`** (and store tests if WAL changes): tool allowlist exact set, data-dir/`--db` resolution unit tests, stub handler smoke. **No WebDriver E2E for MCP** in this phase.

### Claude's Discretion
- Exact `rmcp` 2.2.0 ServerHandler / `#[tool]` / router wiring patterns
- Stub response JSON shape (as long as tools are listed and non-mutating)
- CLI flag parsing library (`clap` vs manual) — prefer minimal deps
- Whether default AppData path helper lives in `crates/mcp` or a tiny shared util (must not open SQLite outside `store`)

### Deferred Ideas (OUT OF SCOPE)
- Real hybrid `search` / `list_sources` semantics — **Phase 18** (`kb_readonly` + MCP-01/MCP-02)
- MCP happy-path cargo harness + citation-trust E2E — **Phase 19**
- `get_chunk` / `read`, Settings toggle, HTTP/SSE — **MCP-F01..F03**
- Empty-DB auto-create on MCP start — **rejected (D-06)**
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MCP-03 | Project ships a stdio MCP binary (`jarvis-mcp`) plus Cursor/Claude Desktop config documentation | `crates/mcp` + `jarvis-mcp` bin; `rmcp` stdio transport; `docs/mcp.md` + README pointer |
| MCP-04 | MCP tool surface is structurally read-only (allowlist: `search` + `list_sources` only) | Fixed two-tool registry via `#[tool_router]`; unit test `tools/list` name set; no agent `execute_tool` reuse |
</phase_requirements>

## Summary

Phase 17 delivers **transport + structural read-only contract**, not retrieval semantics. External hosts (Cursor, Claude Desktop) spawn a **separate console binary** that speaks MCP over stdio, advertises exactly two tools, and resolves the user's existing `kb.sqlite` without mutating the index. Tool bodies are **stubs** until Phase 18 wires `kb_readonly`.

The GUI release binary intentionally hides its console:

```1:6:src-tauri/src/main.rs
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri_app_lib::run()
}
```

MCP **must not** inherit this attribute. A dedicated `jarvis-mcp` binary is the only viable stdio host on Windows (Pitfall: in-process GUI stdio).

**Critical pre-open guard:** `Store::open` calls `Connection::open` + `init_schema`, which **creates** a missing file and initializes tables. D-06 requires a **preflight** `Path::exists()` check in the MCP binary **before** calling `Store::open`. Do not add a silent “create KB” code path.

**Primary recommendation:** Add `crates/mcp` with `[[bin]] name = "jarvis-mcp"`, pin `rmcp` 2.2.0 (`server` + `transport-io` + `macros`), implement `JarvisMcp` with `#[tool_router(server_handler)]` and exactly two `#[tool]` methods returning stub JSON, resolve DB via `paths.rs` precedence, open Store with dim from sibling `config.json`, enable WAL in `Store::open`, document host config in `docs/mcp.md`.

## Approach (aligned with CONTEXT)

### 1. Crate layout

```
crates/mcp/
  Cargo.toml          # [[bin]] jarvis-mcp; deps: rmcp, store, config, tokio, serde_json, thiserror
  src/
    lib.rs            # JarvisMcp server struct + tool_router (testable)
    main.rs           # CLI parse → resolve paths → open Store → serve(stdio())
    paths.rs          # resolve_data_dir / resolve_db_path / default_app_data_dir
  tests/
    allowlist.rs      # integration: tool names == {search, list_sources}
    paths.rs          # --db / JARVIS_DATA_DIR / default precedence
```

- **`lib.rs` testable:** unit tests can call tool router / path helpers without spawning stdio.
- **`main.rs` thin:** parse args, fail fast on missing DB, `#[tokio::main]`, `service.serve(rmcp::transport::stdio()).await?` then `waiting()`.
- **No `kb_readonly` in Phase 17:** defer to Phase 18; stubs must not import `retriever` / `embedder` unless needed for dim-only config (prefer `config` crate only at startup).

### 2. `rmcp` 2.2.0 wiring (verified)

Pin in workspace `Cargo.toml`:

```toml
rmcp = { version = "2.2.0", default-features = false, features = ["server", "transport-io", "macros"] }
```

Tools-only server pattern from official SDK (docs.rs 2.2.0 + upstream README):

```rust
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router, ServiceExt, transport::stdio};

#[derive(Clone)]
struct JarvisMcp { /* hold Arc<Store> if stub needs DB touch; optional Phase 17 */ }

#[tool_router(server_handler)]
impl JarvisMcp {
    #[tool(description = "Hybrid search over indexed knowledge (stub until Phase 18)")]
    fn search(&self, Parameters(args): Parameters<SearchParams>) -> String { /* stub JSON */ }

    #[tool(description = "List indexed sources (stub until Phase 18)")]
    fn list_sources(&self) -> String { /* stub JSON */ }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // resolve db, preflight exists, Store::open, build JarvisMcp
    let service = JarvisMcp { /* ... */ }.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
```

- Use **`#[tool_router(server_handler)]`** — no separate manual `ServerHandler` impl unless custom server metadata is needed later.
- Define arg structs with `schemars::JsonSchema` + `serde::Deserialize` for `search` (`query: String`, optional `limit`).
- Stub payload shape (discretion): JSON object e.g. `{"status":"not_implemented","phase":18,"message":"..."}` as tool result text; non-mutating, parseable by hosts.
- Optional: set tool annotation `readOnlyHint: true` if macro supports it — **advisory only**; structural allowlist (exactly two tools) is the real enforcement (Pitfall 1).

**Version gate:** `cargo info rmcp@2.2.0` — latest on crates.io is **3.0.0-beta.3** (MSRV conflict). Stay on **2.2.0** per D-02.

### 3. KB path resolution (D-05, D-06)

Precedence implemented in pure `paths.rs` (no SQLite):

| Priority | Source | Resolved `kb.sqlite` | Resolved config dir |
|----------|--------|----------------------|---------------------|
| 1 | `--db /path/to/kb.sqlite` | exact path | `db.parent()` |
| 2 | `JARVIS_DATA_DIR=/path/to/dir` | `{dir}/kb.sqlite` | `{dir}` |
| 3 | Default | `{app_data}/kb.sqlite` | `{app_data}` |

**Default app data directory** (match Tauri `identifier: com.jarvis.app`):

| OS | Default directory |
|----|-------------------|
| Windows | `%APPDATA%\com.jarvis.app` |
| Linux | `$XDG_DATA_HOME/com.jarvis.app` or `~/.local/share/com.jarvis.app` |
| macOS | `~/Library/Application Support/com.jarvis.app` |

Implementation: small helper in `crates/mcp/src/paths.rs` using `std::env::var("APPDATA")` / `HOME` / `XDG_DATA_HOME` — **do not** depend on Tauri runtime. Mirror path documented in `.planning/codebase/INTEGRATIONS.md` and `state.rs` layout:

```38:39:src-tauri/src/state.rs
    let config_path = app_data.join("config.json");
    let db_path = app_data.join("kb.sqlite");
```

**Startup sequence:**

1. Parse `--db` from `std::env::args()` (no `clap` in workspace today — hand-roll minimal parser).
2. `resolve_db_path()` → `PathBuf`.
3. If `!db_path.is_file()` → `eprintln!(...)` + **`std::process::exit(1)`** (D-06).
4. Load `{config_dir}/config.json` via `config::load_config` (not keychain migration path — MCP is read-only sidecar).
5. `dim = config.embedding_dim()` (fallback `AppConfig::default().embedding_dim()` if config file absent but DB exists — edge case only).
6. `Store::open(&db_path, dim)?` — sole SQLite open site.

### 4. WAL in `Store::open` (D-08)

Add after `Connection::open` in both `open` and `open_in_memory`:

```rust
conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode=WAL;")?;
```

Optionally add `PRAGMA busy_timeout=5000;` if integration tests show `SQLITE_BUSY` under GUI+MCP — start with WAL only to minimize diff.

Current open path (no WAL yet):

```29:37:crates/store/src/store.rs
    pub fn open(path: impl AsRef<Path>, dim: usize) -> Result<Self> {
        register_sqlite_vec();
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        init_schema(&conn, dim)?;
        Ok(Self {
            conn: Mutex::new(conn),
            dim: Mutex::new(dim),
        })
    }
```

Add store unit test: after `Store::open`, query `PRAGMA journal_mode` → `wal` (in-memory may report `memory` — document or skip in-memory assertion).

### 5. Documentation (D-07)

**`docs/mcp.md`** sections:

1. What `jarvis-mcp` is (read-only KB bridge; Phase 17 stubs / Phase 18 real search).
2. Build: `cargo build -p mcp --release` → `target/release/jarvis-mcp.exe`.
3. Path env vars (`JARVIS_DATA_DIR`, `--db`) and default `%APPDATA%\com.jarvis.app`.
4. Cursor `mcp.json` snippet (from STACK.md).
5. Claude Desktop config equivalent.
6. Prerequisite: user must have run Jarvis GUI at least once so `kb.sqlite` exists (D-06).

**README.md:** one-line link under Tests or new “MCP” bullet → `docs/mcp.md`.

### 6. Explicit non-goals (Phase 17)

- No `retriever`, `agent`, `indexer`, or mutate `Store` methods from MCP handlers.
- No `kb_readonly` extraction (Phase 18).
- No WebDriver / `npm run test:e2e:local` MCP coverage (Phase 19 cargo harness).
- No Settings UI toggle (MCP-F02).
- No HTTP/SSE transport features on `rmcp`.

## Pitfalls

| # | Pitfall | Mitigation (Phase 17) |
|---|---------|------------------------|
| 1 | **`windows_subsystem = "windows"` on MCP** — stdio broken for Cursor spawn | Separate `jarvis-mcp` binary; **never** `#![windows_subsystem]` on `crates/mcp/src/main.rs` |
| 2 | **`rmcp` 3.x / MSRV 1.88** — workspace MSRV is **1.85** | Pin **exact** `2.2.0`; re-check at implement time; reject 3.0.0-beta.x |
| 3 | **Silent empty DB creation** — `Store::open` + `init_schema` creates file | Preflight `is_file()` in MCP **before** `Store::open`; clear stderr + exit 1 |
| 4 | **Second SQLite owner** — `rusqlite` in `mcp` manifest | MCP depends on `store` only; grep guard: no direct `rusqlite` in `crates/mcp` |
| 5 | **Hint-only read-only** — registering agent tool loop | Exactly two `#[tool]` fns; test name set; no `agent::execute_tool` |
| 6 | **GUI+MCP lock contention** | WAL in `Store::open`; keep stub handlers short (no embed under lock in Phase 17) |
| 7 | **Embedding MCP in Tauri exe** | Do not add MCP server to `src-tauri` main; sidecar only |
| 8 | **Stub phase pulling heavy deps** | Phase 17 deps: `rmcp`, `store`, `config`, `tokio`, `serde`/`serde_json`, `thiserror` — defer `embedder`/`retriever` to Phase 18 |

## File Create / Modify List

### Create

| File | Purpose |
|------|---------|
| `crates/mcp/Cargo.toml` | Package + `[[bin]] name = "jarvis-mcp"` |
| `crates/mcp/src/main.rs` | Console entry: args, preflight, tokio stdio serve |
| `crates/mcp/src/lib.rs` | `JarvisMcp` + `#[tool_router(server_handler)]` stub tools |
| `crates/mcp/src/paths.rs` | `--db` / `JARVIS_DATA_DIR` / default AppData resolution |
| `crates/mcp/tests/allowlist.rs` | Integration: tool list == `{search, list_sources}` |
| `crates/mcp/tests/paths.rs` | Unit/integration: path precedence + missing DB error |
| `docs/mcp.md` | Host config documentation (MCP-03) |

### Modify

| File | Change |
|------|--------|
| `Cargo.toml` | Add `"crates/mcp"` member; `[workspace.dependencies] rmcp = { version = "2.2.0", ... }` |
| `crates/store/src/store.rs` | `PRAGMA journal_mode=WAL` in `open` + `open_in_memory` |
| `crates/store/src/store.rs` or `crates/store/tests/` | WAL pragma regression test |
| `README.md` | Short pointer to `docs/mcp.md` |

### Do NOT modify (Phase 17)

| File | Reason |
|------|--------|
| `src-tauri/src/main.rs` | GUI subsystem unchanged |
| `crates/agent/src/tools.rs` | `kb_readonly` extraction is Phase 18 |
| `src/App.tsx` / Settings | MCP-F02 deferred |
| `e2e/specs/*` | No WebDriver MCP this phase (D-09) |

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| MCP protocol / stdio | `crates/mcp` (`rmcp`) | — | Protocol stays out of Tauri |
| Tool allowlist | `crates/mcp` (`#[tool_router]`) | Tests | Structural MCP-04 enforcement |
| KB path resolution | `crates/mcp/src/paths.rs` | `docs/mcp.md` | Pure Rust; no Tauri `app_data_dir()` |
| SQLite open + WAL | `crates/store` | `crates/mcp` caller | Single DB owner invariant |
| Config / embed dim | `crates/config` | MCP startup | Same `embedding_dim()` as GUI |
| Real search/list | — (Phase 18) | `kb_readonly` + `retriever` | Out of Phase 17 boundary |
| Host documentation | `docs/mcp.md` | `README.md` | MCP-03 |

## Standard Stack

| Library | Version | Purpose | Notes |
|---------|---------|---------|-------|
| **`rmcp`** | **2.2.0** | MCP server + stdio + macros | `default-features = false`; features `server`, `transport-io`, `macros` |
| `store` | workspace path | Sole SQLite access | `Store::open` only |
| `config` | workspace path | `load_config` + `embedding_dim()` | No keychain side effects in MCP |
| `tokio` | workspace 1.x | `#[tokio::main]`, async stdio | Already workspace dep |
| `serde` / `serde_json` | workspace | Stub JSON payloads | |
| `thiserror` | workspace 1.x | MCP crate errors | |
| `schemars` | transitive via `rmcp` | Tool input schemas | Do not pin unless direct use requires |

**Not added:** `clap` (no existing workspace usage — manual `--db` parse), `rusqlite` in mcp, `rmcp` HTTP transport features.

## Build & Verify Commands

```bash
# Build sidecar
cargo build -p mcp --release

# Phase gate
cargo test -p mcp
cargo test -p store          # WAL regression

# Workspace sanity (no MCP E2E)
cargo test -p store -p config
```

Release artifact path for docs: `target/release/jarvis-mcp.exe` (Windows) / `target/release/jarvis-mcp` (Unix).

## Validation Architecture

> `workflow.nyquist_validation: true` in `.planning/config.json` [VERIFIED]

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test`; `tempfile` for path tests; optional in-process tool router inspection |
| Config file | `crates/mcp/Cargo.toml`; store inline `#[cfg(test)]` |
| Quick run command | `cargo test -p mcp` |
| Full suite command | `cargo test -p mcp && cargo test -p store` |
| E2E this phase | **Not required** (D-09; MCP protocol verified at cargo layer; WebDriver deferred to Phase 19) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MCP-04 | `tools/list` names == `{search, list_sources}` only | integration | `cargo test -p mcp allowlist` | ❌ Wave 0 |
| MCP-04 | No third tool name / no mutate aliases | integration | same | ❌ Wave 0 |
| MCP-04 | Stub `search` / `list_sources` callable without panic | integration | `cargo test -p mcp stub_smoke` | ❌ Wave 0 |
| MCP-03 | `--db` overrides env | unit | `cargo test -p mcp paths` | ❌ Wave 0 |
| MCP-03 | `JARVIS_DATA_DIR` → `{dir}/kb.sqlite` | unit | same | ❌ Wave 0 |
| MCP-03 | Missing DB → error (no file created) | unit | same | ❌ Wave 0 |
| D-08 | WAL enabled on file-backed Store | unit | `cargo test -p store wal` | ❌ Wave 0 |
| MCP-01/02 | Real hybrid search | — | Phase 18 | ⏭️ deferred |
| TRUST-02 | MCP happy-path harness | — | Phase 19 | ⏭️ deferred |

### Sampling Rate

- **Per task commit:** `cargo test -p mcp` (focused module if split)
- **Per wave merge:** `cargo test -p mcp && cargo test -p store`
- **Phase gate:** allowlist + paths + WAL green; `docs/mcp.md` present; README link added; **no** `npm run test:e2e:local` requirement for Phase 17 alone

### Wave 0 Gaps

- [ ] `crates/mcp/` scaffold + workspace member + `rmcp` pin
- [ ] `paths.rs` tests with `tempfile` (existing file vs missing)
- [ ] Allowlist test enumerating registered tool names
- [ ] Stub handler smoke (returns JSON, no Store mutation)
- [ ] WAL test in `store`
- [ ] `docs/mcp.md` + README pointer
- [ ] Manual smoke (optional): Cursor `mcp.json` pointing at debug binary with `JARVIS_DATA_DIR` — not CI-gated

## Phase 18 Handoff Notes

- Replace stub bodies with `kb_readonly::search` / `kb_readonly::list_sources` shared with `crates/agent`.
- MCP will then depend on `retriever`, `embedder`, and likely hold `Arc<Store>` + embedder in `JarvisMcp` state.
- Keep the same two tool names and allowlist test — Phase 18 should only change handler internals and golden output tests.

## RESEARCH COMPLETE

**Phase:** 17-mcp-transport-read-only-scaffold
**Confidence:** HIGH
**Ready for:** `/gsd-plan-phase 17`
