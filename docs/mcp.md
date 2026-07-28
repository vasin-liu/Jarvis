# Jarvis MCP (`jarvis-mcp`)

Read-only MCP bridge for the Jarvis knowledge base. External hosts (Cursor, Claude Desktop) spawn a **console** binary over **stdio** — not the Tauri GUI process.

**Status:** Phase 18 — live hybrid `search` and Indexed `list_sources` via shared `kb_readonly` helpers (same path as in-app agent tools).

## Build

```bash
cargo build -p mcp --release
```

| Platform | Binary |
| --- | --- |
| Windows | `target/release/jarvis-mcp.exe` |
| Unix | `target/release/jarvis-mcp` |

## Prerequisite

Run the **Jarvis desktop app once** so `%APPDATA%\com.jarvis.app\kb.sqlite` (or your data dir) exists. `jarvis-mcp` **never** creates the database. Embedding provider settings come from sibling `config.json` (`config::build_embedder`) — startup **fails closed** if the embedder cannot be built.

## Path precedence

1. `--db <path-to-kb.sqlite>` — explicit file
2. `JARVIS_DATA_DIR` — directory; opens `{dir}/kb.sqlite`
3. Default app data dir — Windows `%APPDATA%\com.jarvis.app\kb.sqlite`; macOS `~/Library/Application Support/com.jarvis.app\kb.sqlite`; Linux `$XDG_DATA_HOME/com.jarvis.app/kb.sqlite` or `~/.local/share/com.jarvis.app/kb.sqlite`

## Cursor (`mcp.json`)

```json
{
  "mcpServers": {
    "jarvis": {
      "command": "D:/path/to/target/release/jarvis-mcp.exe",
      "args": [],
      "env": {
        "JARVIS_DATA_DIR": "%APPDATA%/com.jarvis.app"
      }
    }
  }
}
```

Or pin the DB file:

```json
{
  "mcpServers": {
    "jarvis": {
      "command": "D:/path/to/target/release/jarvis-mcp.exe",
      "args": ["--db", "D:/path/to/kb.sqlite"]
    }
  }
}
```

## Claude Desktop

Add the same stdio server under Claude Desktop’s MCP config (`claude_desktop_config.json`), using `command` / `args` / `env` as above. Use an absolute path to `jarvis-mcp` (`.exe` on Windows).

## Tools (allowlist)

Exactly two read-only tools — no write/delete/ingest/memory tools.

### `search`

Hybrid retrieval: vector + FTS5 + RRF via `retriever::search_kb` / `RetrieverConfig::default()` (`final_k` = **8**).

| Arg | Notes |
| --- | --- |
| `query` | Required string; empty/whitespace → tool error `empty_query` |
| `limit` | Optional; clamped to ≤ `final_k` (8) |

Success JSON (text content):

```json
{
  "results": [
    {
      "chunk_id": 1,
      "source_id": "...",
      "title": "...",
      "loc": "...",
      "excerpt": "…"
    }
  ]
}
```

Excerpts are truncated to **200** Unicode characters. Absolute filesystem paths / `uri` are **not** included.

### `list_sources`

Inventories sources with `IndexStatus::Indexed` only (same filter as agent `list_sources`).

| Field | Notes |
| --- | --- |
| `sources[]` | `{ id, title, kind }` — no `uri` / absolute paths |
| `truncated` | `true` when more than **200** Indexed sources |
| `total_indexed` | Full Indexed count before cap |

Hard cap: **200** sources per response.
