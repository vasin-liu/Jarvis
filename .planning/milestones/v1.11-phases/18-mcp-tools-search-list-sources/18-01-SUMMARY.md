# 18-01 Summary — kb_readonly helpers

**Status:** complete
## Shipped
- `crates/retriever/src/kb_readonly.rs`: `search_kb`, `list_indexed_sources`, caps 8/200/200-char excerpts
## Verify
- `cargo test -p retriever kb_readonly` — 8 passed; full retriever suite green
