---
phase: 05-agent-protocol
plan: "02"
subsystem: agent
tags: [mock, ipc, orchestration]
provides:
  - JSON line Mock agent tool response
  - AskResponse.tool_parse_warnings IPC field
  - Orchestration warning merge with [agent_name]: prefix
key-files:
  modified: [crates/llm/src/mock.rs, crates/agent/src/tools.rs, crates/rag/src/types.rs, crates/agent/src/orchestrate.rs, src-tauri/src/lib.rs]
requirements-completed: [AGT-01, AGT-02, AGT-03]
completed: 2026-06-29
---

# Plan 05-02 Summary

**Mock and prompt aligned to bare JSON protocol; parse warnings flow through IPC and orchestration merge.**

## Accomplishments

- `MockChatModel` emits bare JSON line for agent tool round
- `build_tools_prompt` documents JSON-only examples (no XML in instructions)
- `AskResponse.tool_parse_warnings` with serde default + `agent_response_to_ask` wiring
- `run_orchestrated` merges prefixed warnings across pipeline steps
- Tests: `mock_agent_tool_emits_json_line`, `orchestrate_warnings_merge`, rag serde round-trip

## Self-Check: PASSED

- `cargo test -p agent -p llm -p rag` green
