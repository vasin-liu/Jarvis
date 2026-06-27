# Phase 5: Agent Protocol - Context

**Gathered:** 2026-06-27 (synthesized from ROADMAP + REQUIREMENTS + research — no discuss-phase session)
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace fragile **XML-wrapped JSON** tool parsing with a **structured JSON-first** protocol, surface parse failures in Chat UI, and **relocate agent IPC** to `commands/agent.rs` — zero regression on agent mode, orchestration, or E2E journeys.

**In scope:**
- `ToolCallParser` trait in `crates/agent` with JSON primary + XML fallback (logged)
- Parse errors in `AgentResponse` / `AskResponse` metadata → Chat UI (AGT-02)
- Update `MockChatModel` + `build_tools_prompt` for JSON line protocol (AGT-01, AGT-03)
- Create `src-tauri/src/commands/agent.rs` — move agent/profile/skill/hook/plugin IPC from `lib.rs` (SHELL-04/05)
- Verify `commands/config.rs` complete; **do not change** `e2e.rs` bootstrap unless required (SHELL-05)
- `agent.spec.ts` + `full-ui.spec.ts` agent sections green

**Out of scope (later phases):**
- Native OpenAI `tool_calls` API (function-calling) — JSON-in-text / json_object mode only this phase
- Settings view extraction (Phase 6)
- Removing XML fallback entirely (post-migration release)
- Agent orchestration refactor (pipeline/router logic stays in `crates/agent`)

</domain>

<decisions>
## Implementation Decisions

### Tool Call Protocol (AGT-01, AGT-02)
- **D-01:** Add `ToolCallParser` trait: `parse(text) -> ToolCallParseOutcome` where outcome is `Found(payload) | NotFound | Failed { error, raw_snippet }`.
- **D-02:** **Primary parser:** `JsonToolCallParser` — accepts (a) bare JSON object line `{"name":"...","arguments":{...}}`, (b) optional markdown ```json fence, (c) JSON object anywhere in text (first valid match). Strict schema: `name` string + `arguments` object.
- **D-03:** **Fallback parser:** existing XML `<tool_call>...</tool_call>` logic moved to `XmlToolCallParser`; when JSON misses and XML matches, emit **`eprintln!` deprecation** (`[jarvis] XML tool_call format deprecated; use JSON`).
- **D-04:** **Parser chain:** `CompositeToolCallParser` tries JSON then XML; `run.rs` uses composite instead of `parse_tool_call` returning `Option`.
- **D-05:** On `Failed` or unrecoverable malformed tool attempt (JSON/XML detected but invalid): append to `AgentResponse.tool_parse_warnings: Vec<String>`; do **not** silently treat reply as final answer when tool intent was detected.
- **D-06:** `NotFound` (no tool call) → normal final-answer path unchanged.

### Response Metadata & UI (AGT-02)
- **D-07:** Extend `AskResponse` with `#[serde(default)] tool_parse_warnings: Vec<String>` in `crates/rag/src/types.rs`; wire through `agent_response_to_ask`.
- **D-08:** Chat UI: when `tool_parse_warnings.length > 0`, show inline amber panel with `data-testid="agent-tool-parse-warnings"` (alongside existing `agent-tool-calls`).
- **D-09:** Existing `data-testid="agent-tool-calls"` unchanged; tool calls still populate on successful parse.

### Mock & Prompt (AGT-01, AGT-03)
- **D-10:** `MockChatModel` agent path emits **JSON line** (no XML wrapper) for first tool round.
- **D-11:** `build_tools_prompt` documents JSON format first; keep one XML example marked deprecated for fallback testing.
- **D-12:** E2E continues using Mock provider (`JARVIS_E2E=1`) — no live LLM in CI.

### Tauri Command Split (SHELL-04, SHELL-05)
- **D-13:** Create `commands/agent.rs` and move from `lib.rs`: `list_agent_profiles`, `list_skills`, `list_hooks`, `list_plugins`, `upsert_agent_profile`, `remove_agent_profile`, `set_active_agent`, `ask_agent_in_session`, `ask_agent_in_session_stream`, plus helpers `build_agent_context`, `execute_agent_question`, `agent_response_to_ask`, `run_ask_agent_in_session`, resolvers if agent-only.
- **D-14:** **`start_ask_e2e` stays in `lib.rs`** or moves to `commands/chat.rs` (E2E bootstrap) — do **not** break `e2e.rs` / `start_ask_e2e` IPC name.
- **D-15:** Thin wrappers only; orchestration stays in `crates/agent`.
- **D-16:** `commands/config.rs` already exists — verify complete; no functional change unless compile requires import fix.

### Quality Gates
- **D-17:** `cargo test -p agent` + `npm test` + `npx tsc --noEmit` required.
- **D-18:** `agent.spec.ts` passes with structured JSON Mock responses.
- **D-19:** Preserve all agent-related testids: `agent-mode-toggle`, `agent-tool-calls`, `orchestration-mode-*`.

### Claude's Discretion
- Exact JSON extraction heuristic (line vs block vs regex).
- Whether `tool_parse_warnings` vs `tool_parse_errors` field naming (prefer `warnings` for non-fatal).
- OpenAI cloud json_object prompt tweak scope (minimal — parser must handle JSON text in reply).

</decisions>

<canonical_refs>
## Canonical References

- `.planning/ROADMAP.md` — Phase 5 goal + success criteria
- `.planning/REQUIREMENTS.md` — AGT-01..03, SHELL-05
- `.planning/research/PITFALLS.md` § agent protocol migration
- `.planning/research/ARCHITECTURE.md` — S7 agent protocol slice
- `crates/agent/src/tools.rs` — current XML parse + prompt
- `crates/agent/src/run.rs` — agent loop
- `crates/llm/src/mock.rs` — Mock agent tool response
- `e2e/specs/agent.spec.ts` — agent E2E
- `src/views/ChatView.tsx` — tool call UI

</canonical_refs>

---

*Phase: 5-Agent Protocol*
*Context synthesized: 2026-06-27*
