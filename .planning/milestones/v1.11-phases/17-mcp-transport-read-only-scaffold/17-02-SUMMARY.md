# 17-02 Summary — WAL on Store::open

**Status:** complete
**Plan:** 17-02
**Phase:** 17-mcp-transport-read-only-scaffold

## What shipped
- `PRAGMA journal_mode=WAL` in `Store::open` and `Store::open_in_memory` batch (with foreign_keys)
- Regression tests: `store_open_enables_wal`, `open_in_memory_initializes_after_pragma_batch`

## Verify
- `cargo test -p store wal` — pass
- `cargo test -p store` — pass (26 unit + 2 integration)

## Decisions
- D-08: file-backed Store reports journal_mode wal for GUI + MCP coexistence
