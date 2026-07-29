# Phase 17: MCP transport + read-only scaffold - Pattern Map

**Mapped:** 2026-07-28
**Files analyzed:** 11 (7 create, 4 modify)
**Analogs found:** 10 / 11

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/mcp/Cargo.toml` | config | — | `crates/lark/Cargo.toml` + root `Cargo.toml` | exact |
| `crates/mcp/src/main.rs` | controller | request-response (stdio) | `src-tauri/src/main.rs` (inverse) + `src-tauri/src/state.rs` startup | role-match |
| `crates/mcp/src/lib.rs` | service / provider | request-response | `crates/agent/src/tools.rs` (`list_sources` stub target) | role-match |
| `crates/mcp/src/paths.rs` | utility | file-I/O | `src-tauri/src/state.rs` + `crates/agent/tests/real_embed_hang.rs` | exact |
| `crates/mcp/tests/allowlist.rs` | test | transform (assert set) | `crates/agent/src/tools.rs` `#[cfg(test)]` | role-match |
| `crates/mcp/tests/paths.rs` | test | file-I/O | `crates/config/tests/migration.rs` | exact |
| `docs/mcp.md` | config / docs | — | `README.md` + `.planning/research/STACK.md` | role-match |
| `Cargo.toml` (root) | config | — | existing workspace members block | exact |
| `crates/store/src/store.rs` | store | file-I/O | self (extend `open` / `open_in_memory`) | exact |
| `crates/store/tests/` or inline `#[cfg(test)]` | test | file-I/O | `crates/store/tests/persistence.rs` + inline tests in `store.rs` | exact |
| `README.md` | config / docs | — | existing `README.md` Docs section | exact |

## Pattern Assignments

### `crates/mcp/Cargo.toml` (config)

**Analog:** `crates/lark/Cargo.toml` (thin workspace crate) + root `Cargo.toml` (members + `[workspace.dependencies]`)

**Workspace member pattern** (root `Cargo.toml` lines 1–19):

```toml
[workspace]
members = [
    "src-tauri",
    "crates/store",
    # ...
    "crates/agent",
]
resolver = "2"
```

**Thin crate manifest pattern** (`crates/lark/Cargo.toml`):

```toml
[package]
name = "lark"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
ingest = { path = "../ingest" }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = "3"
```

**New for Phase 17:** add `[[bin]] name = "jarvis-mcp"` (first workspace binary besides Tauri); deps `rmcp = { workspace = true }`, `store`, `config`, `tokio`, `serde`/`serde_json`, `thiserror` — **no** `rusqlite`. Pin `rmcp` in root `[workspace.dependencies]` per `.planning/research/STACK.md` lines 45–55.

---

### `crates/mcp/src/main.rs` (controller, request-response)

**Analog:** `src-tauri/src/state.rs` (startup: resolve paths → load config → `Store::open`) — **not** `src-tauri/src/main.rs` (GUI hides console)

**Anti-pattern — GUI binary must NOT be copied** (`src-tauri/src/main.rs` lines 1–6):

```rust
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri_app_lib::run()
}
```

**Startup sequence to mirror** (`src-tauri/src/state.rs` lines 38–58):

```rust
    let config_path = app_data.join("config.json");
    let db_path = app_data.join("kb.sqlite");
    // ...
    let mut config = load_config_with_migration(&config_path).map_err(|e| e.to_string())?;
    config.fastembed_cache_dir = Some(app_data.join("fastembed_cache"));
    let config_dim = config.embedding_dim();

    let store = Arc::new(Store::open(&db_path, config_dim).map_err(|e| e.to_string())?);
```

**MCP-specific deviations:**
- Use `config::load_config` (not `load_config_with_migration`) — no keychain side effects (RESEARCH §3).
- **Preflight** `db_path.is_file()` before `Store::open` — D-06; `Store::open` + `init_schema` creates missing files.
- On missing DB: `eprintln!(...)` + `std::process::exit(1)`.
- `#[tokio::main]` + `rmcp` stdio serve (RESEARCH §2) — no Tauri `App` / `Manager`.
- **No** `#![windows_subsystem = "windows"]`.

---

### `crates/mcp/src/lib.rs` (service / provider, request-response)

**Analog:** `crates/agent/src/tools.rs` — future Phase 18 will align MCP `search` / `list_sources` with agent KB tools; Phase 17 stubs only

**Tool name + JSON result pattern** (`crates/agent/src/tools.rs` lines 90–97):

```rust
        "list_sources" => {
            let sources = store.list_sources()?;
            let lines: Vec<String> = sources
                .into_iter()
                .filter(|s| s.status == IndexStatus::Indexed)
                .map(|s| format!("- {} ({})", s.title, s.kind.as_str()))
                .collect();
            Ok((lines.join("\n"), vec![]))
        }
```

**Module layout pattern** (`crates/lark/src/lib.rs`):

```rust
mod error;
mod health;
// ...

pub use error::{LarkError, Result};
pub use health::{check_auth, detect_cli_bin, LarkAuthStatus};
```

**Phase 17 lib.rs shape:** `mod error; mod paths;` + `JarvisMcp` struct with `#[tool_router(server_handler)]` and exactly two `#[tool]` methods (`search`, `list_sources`). Stub returns structured JSON string e.g. `{"status":"not_implemented","phase":18,...}` via `serde_json::json!` (same crate pattern as `agent/src/tools.rs` line 6). **Do not** call `execute_tool` or import `retriever`/`embedder` in Phase 17.

**Crate root re-exports** (`crates/store/src/lib.rs`):

```rust
pub mod error;
pub mod store;
pub mod types;
// ...

pub use error::{Result, StoreError};
pub use store::Store;
```

---

### `crates/mcp/src/paths.rs` (utility, file-I/O)

**Analog:** `src-tauri/src/state.rs` (layout) + `crates/agent/tests/real_embed_hang.rs` (APPDATA path without Tauri)

**Canonical data layout** (`src-tauri/src/state.rs` lines 38–39):

```rust
    let config_path = app_data.join("config.json");
    let db_path = app_data.join("kb.sqlite");
```

**Default AppData helper** (`crates/agent/tests/real_embed_hang.rs` lines 22–27):

```rust
fn fastembed_cache_dir() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    let dir = PathBuf::from(appdata)
        .join("com.jarvis.app")
        .join("fastembed_cache");
    dir.exists().then_some(dir)
}
```

**Env-var precedence pattern** (`crates/config/src/file.rs` lines 9–11, 13–19):

```rust
pub fn is_e2e_mode() -> bool {
    std::env::var("JARVIS_E2E").is_ok()
}

pub fn load_config(path: impl AsRef<Path>) -> Result<AppConfig> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    // ...
}
```

**Phase 17 precedence:** `--db` > `JARVIS_DATA_DIR/kb.sqlite` > `{default_app_data}/kb.sqlite`. Pure functions, no SQLite. Document `%APPDATA%\com.jarvis.app` per `.planning/codebase/INTEGRATIONS.md`.

---

### `crates/mcp/tests/allowlist.rs` (test, transform)

**Analog:** `crates/agent/src/tools.rs` `#[cfg(test)]` + tool name assertions

**Tool name assertion pattern** (`crates/agent/src/tools.rs` lines 264–268):

```rust
    #[test]
    fn parses_tool_call_block() {
        let text = r#"prefix <tool_call>{"name":"list_tasks","arguments":{}}</tool_call>"#;
        let call = parse_tool_call(text).unwrap();
        assert_eq!(call.name, "list_tasks");
    }
```

**Integration test file layout** (`crates/store/tests/persistence.rs` header):

```rust
use store::{IndexStatus, NewChunk, Source, SourceKind, Store};

#[test]
fn data_survives_reopen_and_is_searchable() {
    let dir = tempfile::tempdir().unwrap();
    // ...
}
```

**Phase 17:** assert registered tool names == exactly `{search, list_sources}`; stub smoke — call handlers, expect JSON stub, no panic. No live stdio / no WebDriver (D-09).

---

### `crates/mcp/tests/paths.rs` (test, file-I/O)

**Analog:** `crates/config/tests/migration.rs`

**Tempdir + env var save/restore** (`crates/config/tests/migration.rs` lines 6–36):

```rust
#[test]
fn load_config_with_migration_e2e_skips_keychain_and_leaves_runtime_empty() {
    let prev = std::env::var("JARVIS_E2E").ok();
    std::env::set_var("JARVIS_E2E", "1");

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.json");
    fs::write(&path, r#"{ ... }"#).unwrap();

    let cfg = load_config_with_migration(&path).unwrap();
    // assertions ...

    match prev {
        Some(v) => std::env::set_var("JARVIS_E2E", v),
        None => std::env::remove_var("JARVIS_E2E"),
    }
}
```

**Missing file guard pattern** (`src-tauri/src/e2e.rs` lines 33–36):

```rust
    let fixture_path = Path::new(&fixture);
    if !fixture_path.is_file() {
        return Err(format!("e2e fixture missing: {fixture}"));
    }
```

**Phase 17 cases:** `--db` overrides env; `JARVIS_DATA_DIR` → `{dir}/kb.sqlite`; missing DB path returns error and does **not** create file (verify with `tempfile`, no `Store::open` on missing path in preflight tests).

---

### `docs/mcp.md` (config / docs)

**Analog:** `README.md` structure + `.planning/research/STACK.md` host config snippet

**README docs pointer pattern** (`README.md` lines 7–8, 29–35):

```markdown
**Docs:** `docs/superpowers/specs/2026-06-12-personal-ai-knowledge-hub-design.md`  
**Agent guide:** `AGENTS.md`

## Tests

```bash
cargo test --workspace   # Rust unit + integration
```
```

**Host config snippet source** (`.planning/research/STACK.md` lines 60–69):

```json
{
  "mcpServers": {
    "jarvis": {
      "command": "path/to/jarvis-mcp.exe",
      "args": [],
      "env": { "JARVIS_DATA_DIR": "%APPDATA%/com.jarvis.app" }
    }
  }
}
```

**Sections to include:** what `jarvis-mcp` is (Phase 17 stubs); build command `cargo build -p mcp --release`; path precedence; Cursor + Claude Desktop stdio config; prerequisite — GUI must have created `kb.sqlite` (D-06).

---

### `Cargo.toml` (root, modify)

**Analog:** self — add member + workspace dep

**Member list** (lines 2–18): append `"crates/mcp"`.

**Workspace dependency block** (lines 27–42): append:

```toml
rmcp = { version = "2.2.0", default-features = false, features = ["server", "transport-io", "macros"] }
```

Follow existing `{ workspace = true }` convention in child manifests. Do **not** bump `rust-version = "1.85"`.

---

### `crates/store/src/store.rs` (store, modify — WAL)

**Analog:** self — extend existing `open` / `open_in_memory`

**Current open path** (lines 29–37):

```rust
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

**Target change:** merge WAL into pragma batch:

```rust
conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode=WAL;")?;
```

Apply to both `open` and `open_in_memory` (lines 40–48). Keep `store` as sole SQLite owner — MCP calls `Store::open` only.

---

### `crates/store/tests/` or inline WAL test (test, modify)

**Analog:** `crates/store/tests/persistence.rs` (file-backed reopen) + inline `#[cfg(test)]` in `store.rs`

**File-backed integration pattern** (`crates/store/tests/persistence.rs` lines 17–24):

```rust
#[test]
fn data_survives_reopen_and_is_searchable() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("kb.sqlite");

    {
        let store = Store::open(&path, 4).unwrap();
        // ...
    }
```

**Inline unit test module** (`crates/store/src/store.rs` lines 641–672):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    // ...

    #[test]
    fn upsert_get_list_delete_source() {
        let store = Store::open_in_memory(4).unwrap();
        // ...
    }
}
```

**WAL assertion:** after `Store::open(&path, dim)`, query `PRAGMA journal_mode` → `wal` for file-backed DB; skip or expect `memory` for `open_in_memory`.

---

### `README.md` (modify)

**Analog:** self — one-line pointer under Docs or Tests

Add bullet/link: MCP setup → `docs/mcp.md`. Match existing style (lines 7–8): backtick paths, no new top-level section unless warranted.

---

## Shared Patterns

### Single SQLite owner (`store` only)

**Source:** `crates/store/src/store.rs` + `src-tauri/src/state.rs`

**Apply to:** `crates/mcp/src/main.rs`, `crates/mcp/Cargo.toml` (no `rusqlite` dep)

```rust
let store = Arc::new(Store::open(&db_path, config_dim).map_err(|e| e.to_string())?);
```

MCP opens the same `kb.sqlite` the GUI uses; dim from `config.embedding_dim()`.

---

### Config load without keychain migration

**Source:** `crates/config/src/file.rs`

**Apply to:** `crates/mcp/src/main.rs`

```rust
pub fn load_config(path: impl AsRef<Path>) -> Result<AppConfig> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}
```

Use `load_config`, not `load_config_with_migration`, for read-only sidecar.

---

### Embedding dimension resolution

**Source:** `crates/config/src/types.rs` lines 295–305

**Apply to:** `crates/mcp/src/main.rs` before `Store::open`

```rust
impl AppConfig {
    pub fn embedding_dim(&self) -> usize {
        match self.embedding.embedder {
            EmbedderProvider::Mock => self.embedding.mock_embed_dim,
            EmbedderProvider::Ollama => self.embedding.ollama_embed_dim,
            EmbedderProvider::FastEmbed => {
                embedder::fastembed_model_dim(&self.embedding.fastembed_model)
                    .unwrap_or(self.embedding.fastembed_dim)
            }
            EmbedderProvider::Cloud => self.embedding.cloud_embed_dim,
        }
    }
}
```

---

### Error types (`thiserror` per crate)

**Source:** `crates/lark/src/error.rs`

**Apply to:** `crates/mcp/src/error.rs` (new)

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LarkError {
    #[error("command failed: {0}")]
    Command(String),
    // ...
}

pub type Result<T> = std::result::Result<T, LarkError>;
```

Map to `String` or `Box<dyn Error>` at `main.rs` boundary if needed.

---

### Integration tests with `tempfile`

**Source:** `crates/store/Cargo.toml`, `crates/config/tests/migration.rs`

**Apply to:** `crates/mcp/tests/paths.rs`, allowlist tests needing fixtures

```toml
[dev-dependencies]
tempfile = "3"
```

---

### Read-only tool surface (structural allowlist)

**Source:** `crates/agent/src/tools.rs` — **reference only**; Phase 17 must NOT register agent's full tool set

**Apply to:** `crates/mcp/src/lib.rs`

Agent exposes many tools (`search_knowledge`, `list_sources`, mutate tools). MCP Phase 17 exposes **exactly two** names: `search`, `list_sources` (MCP-04). Phase 18 replaces stubs with shared `kb_readonly`; do not wire `execute_tool` in Phase 17.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `crates/mcp/src/lib.rs` (`rmcp` `#[tool_router]` wiring) | service | request-response | **No in-repo MCP/rmcp usage** — first MCP integration. Planner must follow RESEARCH §2 (`rmcp` 2.2.0 docs) for macro/API details; behavioral analog is `agent/src/tools.rs` stub targets only. |

---

## Metadata

**Analog search scope:** `Cargo.toml`, `crates/*/Cargo.toml`, `crates/store/`, `crates/config/`, `crates/agent/`, `crates/lark/`, `src-tauri/src/`, `README.md`, `.planning/research/STACK.md`
**Files scanned:** ~25
**Pattern extraction date:** 2026-07-28

## PATTERN MAPPING COMPLETE

**Phase:** 17 - mcp-transport-read-only-scaffold
**Files classified:** 11
**Analogs found:** 10 / 11

### Coverage
- Files with exact analog: 6
- Files with role-match analog: 4
- Files with no analog: 1 (`rmcp` macro wiring — external SDK only)

### Key Patterns Identified
- Workspace crates use `version.workspace = true` + path deps; first non-Tauri `[[bin]]` in `crates/mcp`
- MCP `main.rs` mirrors `state.rs` startup (paths → config → `Store::open`) but **without** Tauri and **with** missing-DB preflight
- GUI `main.rs` `windows_subsystem` is an anti-pattern for MCP — separate console binary required
- `Store::open` is the only SQLite entry; WAL pragma added in same `execute_batch` as `foreign_keys`
- Path tests follow `config/tests/migration.rs` env save/restore + `tempfile`
- Docs follow README pointer style; host JSON from STACK.md research

### File Created
`.planning/phases/17-mcp-transport-read-only-scaffold/17-PATTERNS.md`

### Ready for Planning
Pattern mapping complete. Planner can now reference analog patterns in PLAN.md files.
