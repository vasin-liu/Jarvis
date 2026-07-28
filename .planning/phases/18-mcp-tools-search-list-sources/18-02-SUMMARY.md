# 18-02 Summary — agent thin wrappers

**Status:** complete
## Shipped
- `search_knowledge` / `list_sources` call `search_kb` / `list_indexed_sources`; truncation footer when capped
## Verify
- `cargo test -p agent` green; arms use shared helpers (no forked retrieve)
