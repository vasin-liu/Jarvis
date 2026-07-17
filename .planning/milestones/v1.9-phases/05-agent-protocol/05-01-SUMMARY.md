---
phase: 05-agent-protocol
plan: "01"
subsystem: agent
tags: [rust, tool-parse, json, xml-fallback]
provides:
  - ToolCallParser trait with Json/Xml/Composite implementations
  - Outcome-driven agent loop with tool_parse_warnings
key-files:
  created: [crates/agent/src/tool_parse.rs]
  modified: [crates/agent/src/run.rs, crates/agent/src/types.rs, crates/agent/src/tools.rs, crates/agent/src/lib.rs]
requirements-completed: [AGT-01, AGT-02]
completed: 2026-06-29
---

# Plan 05-01 Summary

**Structured JSON-first tool parsing with Failed/NotFound outcomes and agent loop stop-on-parse-failure.**

## Accomplishments

- Added `tool_parse.rs` with `JsonToolCallParser`, `XmlToolCallParser`, `CompositeToolCallParser`
- Extended `AgentResponse` with `tool_parse_warnings`
- Updated `run.rs` to branch on `ToolCallParseOutcome` (Found / NotFound / Failed)
- Unit tests: bare JSON, XML fallback, fence rejection, multi-object Failed, `parse_failed_stops_loop`

## Self-Check: PASSED

- `cargo test -p agent` green (23 tests)
