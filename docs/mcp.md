# Jarvis MCP (`jarvis-mcp`)

Read-only MCP bridge for the Jarvis knowledge base. External hosts (Cursor, Claude Desktop) spawn a **console** binary over **stdio** — not the Tauri GUI process.

**Phase 17 status:** transport + tool allowlist only. Tools `search` and `list_sources` return stub `not_implemented` JSON until **Phase 18** wires hybrid retrieval.

## Build

```bash
cargo build -p mcp --release
```

| Platform | Binary |
| --- | --- |
| Windows | `target/release/jarvis-mcp.exe` |
| Unix | `target/release/jarvis-mcp` |

## Prerequisite

Run the **Jarvis desktop app once** so `%APPDATA%\com.jarvis.app\kb.sqlite` (or your data dir) exists. `jarvis-mcp` **never** creates the database.

## Path precedence

1. `--db <path-to-kb.sqlite>` — explicit file
2. `JARVIS_DATA_DIR` — directory; opens `{dir}/kb.sqlite`
3. Default app data dir — Windows `%APPDATA%\com.jarvis.app\kb.sqlite`; macOS `~/Library/Application Support/com.jarvis.app/kb.sqlite`; Linux `$XDG_DATA_HOME/com.jarvis.app/kb.sqlite` or `~/.local/share/com.jarvis.app/kb.sqlite`

Sibling `config.json` (same directory as the DB) supplies embedding dimensions when present.

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

| Tool | Phase 17 | Phase 18 |
| --- | --- | --- |
| `search` | stub JSON | hybrid KB search |
| `list_sources` | stub JSON | list indexed sources |

No other MCP tools are registered. Write/mutate agent tools are intentionally absent.
