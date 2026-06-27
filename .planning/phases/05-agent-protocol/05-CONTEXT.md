# Phase 5: Agent Protocol - Context

**Gathered:** 2026-06-27 (discuss-phase session)
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace fragile **XML-wrapped JSON** tool parsing with a **structured JSON-first** protocol (strict bare-line JSON), surface parse failures in Chat UI, and **relocate agent IPC** to `commands/agent.rs` — zero regression on agent mode, orchestration, or E2E journeys.

**In scope:**
- `ToolCallParser` trait in `crates/agent` with JSON primary + XML fallback (one-release deprecation)
- Parse errors in `AgentResponse` / `AskResponse` metadata → Chat UI (AGT-02)
- Update `MockChatModel` + `build_tools_prompt` for JSON line protocol (AGT-01, AGT-03)
- Create `src-tauri/src/commands/agent.rs` — move agent/profile/skill/hook/plugin IPC from `lib.rs` (SHELL-04/05)
- Verify `commands/config.rs` complete; **do not change** `e2e.rs` bootstrap unless required (SHELL-05)
- `agent.spec.ts` + `full-ui.spec.ts` agent sections green

**Out of scope (later phases):**
- Native OpenAI `tool_calls` API (function-calling) — JSON-in-text only this phase
- Settings view extraction (Phase 6)
- Removing XML fallback entirely (after one-release window — same policy as Phase 4 memory fuzzy match)
- Agent orchestration refactor (pipeline/router logic stays in `crates/agent`)
- Auto-retry agent loop on parse failure
- Per-step orchestration warning panels in step cards

</domain>

<decisions>
## Implementation Decisions

### JSON Tool Call Format (AGT-01)
- **D-01:** **Primary format:** strict **one bare JSON line** per tool call: `{"name":"...","arguments":{...}}`. Trim leading/trailing whitespace on the line before parse.
- **D-02:** **No markdown fence acceptance:** if model uses ` ```json ` instead of bare line → **Failed** + `tool_parse_warnings` (do not strip fence; do not treat as NotFound).
- **D-03:** **Multiple JSON objects** in one reply → **Failed** + warnings (consistent with single-line contract).
- **D-04:** **Schema (strict):** require `name` (string) + `arguments` (object). Extra keys ignored. Missing/invalid types → **Failed**.
- **D-05:** **`build_tools_prompt`:** document **JSON line only** — remove XML examples from primary instructions (Mock/E2E use JSON path).

### Parser Architecture (AGT-01, AGT-02)
- **D-06:** `ToolCallParser` trait: `parse(text) -> ToolCallParseOutcome` = `Found(payload) | NotFound | Failed { error, raw_snippet }`.
- **D-07:** `JsonToolCallParser` implements D-01..D-04. `XmlToolCallParser` holds legacy `<tool_call>...</tool_call>` logic.
- **D-08:** `CompositeToolCallParser`: try JSON first, then XML fallback. `run.rs` uses composite (not `Option`-only silent failure).
- **D-09:** **Failed vs NotFound:** use **intent heuristic** — Failed when `<tool_call>`, ` ```json `, or a line starting with `{"name"` is present but parsing fails; pure natural language with no tool intent → **NotFound** (normal final answer).
- **D-10:** Invalid JSON **inside** valid `<tool_call>` tags → **Failed** + warnings (same as JSON path).

### Parse Failure UX (AGT-02)
- **D-11:** On **Failed:** **stop agent tool loop** for that turn; treat reply as **final answer** (no auto-retry, no error-as-tool-result loop).
- **D-12:** Still **display model raw text** as `answer` alongside warnings (do not hide or replace with generic error string).
- **D-13:** Chat UI: inline **amber panel** below assistant message when `tool_parse_warnings.length > 0`, `data-testid="agent-tool-parse-warnings"`. Existing `agent-tool-calls` unchanged on successful parse.
- **D-14:** Extend `AskResponse` with `#[serde(default)] tool_parse_warnings: Vec<String>`; wire through `agent_response_to_ask`. **Both** `ask_agent_in_session` and `ask_agent_in_session_stream` return warnings in response metadata (no separate Tauri event this phase).

### XML Fallback (one-release, aligned with Phase 4)
- **D-15:** Keep XML fallback **one release**; remove in a future release after deprecation window (same policy as Phase 4 memory fuzzy match).
- **D-16:** When XML fallback matches: **`eprintln!`** one line: `[jarvis] XML tool_call format deprecated; use JSON line`.
- **D-17:** **Tests:** unit/integration tests for `XmlToolCallParser` in `cargo test -p agent`; **E2E covers JSON path only** (no dedicated XML fallback E2E).

### Mock & Prompt (AGT-03)
- **D-18:** `MockChatModel` agent path emits **JSON line** (no XML wrapper) for tool round.
- **D-19:** E2E continues Mock provider (`JARVIS_E2E=1`) — no live LLM in CI.

### Orchestration Warnings (pipeline / router)
- **D-20:** **Merge all steps'** `tool_parse_warnings` into final `AskResponse.tool_parse_warnings`.
- **D-21:** Prefix each merged warning with **`[agent_name]:`** (use profile display name, not id).
- **D-22:** Show merged warnings **only on final assistant message** — not inside per-step orchestration cards.

### Tauri Command Split (SHELL-04, SHELL-05)
- **D-23:** Create `commands/agent.rs`; move agent/profile/skill/hook/plugin IPC + helpers from `lib.rs` (list from synthesized plan 05-03).
- **D-24:** **`start_ask_e2e` stays in `lib.rs`** (or `commands/chat.rs`) — do not break E2E IPC names.
- **D-25:** Thin wrappers only; orchestration stays in `crates/agent`. `commands/config.rs` verified complete.

### Quality Gates
- **D-26:** `cargo test -p agent` + `npm test` + `npx tsc --noEmit` required.
- **D-27:** `agent.spec.ts` passes with structured JSON Mock responses.
- **D-28:** Preserve agent testids: `agent-mode-toggle`, `agent-tool-calls`, `agent-tool-parse-warnings`, `orchestration-mode-*`.

### Claude's Discretion
- Exact line-splitting heuristic (single line vs first non-empty line in multi-line reply).
- `tool_parse_warnings` message wording and raw_snippet truncation length.
- OpenAI cloud json_object prompt tweak scope (minimal — parser must enforce bare line contract).
- Whether composite parser lives in `tools.rs` or new `parse.rs` module.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope & requirements
- `.planning/ROADMAP.md` — Phase 5 goal, success criteria
- `.planning/REQUIREMENTS.md` — AGT-01..03, SHELL-05
- `.planning/phases/04-memory-model-view/04-CONTEXT.md` — one-release deprecation + eprintln pattern (D-15..D-16)

### Research & pitfalls
- `.planning/phases/05-agent-protocol/05-RESEARCH.md`
- `.planning/research/PITFALLS.md` — agent protocol migration
- `.planning/research/ARCHITECTURE.md` — S7 agent protocol slice

### Code (current implementation)
- `crates/agent/src/tools.rs` — XML parse + `build_tools_prompt`
- `crates/agent/src/run.rs` — agent loop
- `crates/llm/src/mock.rs` — Mock agent tool response
- `crates/rag/src/types.rs` — `AskResponse` (add warnings field)
- `src-tauri/src/lib.rs` — agent IPC to relocate
- `src/views/ChatView.tsx` — tool call + warnings UI
- `src/types/ipc.ts` — frontend `AskResponse`
- `e2e/specs/agent.spec.ts` — agent E2E

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `parse_tool_call()` in `tools.rs` — extract to `XmlToolCallParser` unchanged in behavior for valid XML.
- `AgentResponse.tool_calls` + `ChatView` `agent-tool-calls` panel — extend adjacent for warnings panel.
- `commands/config.rs` + `commands/memory.rs` — pattern for `commands/agent.rs` thin relocation.
- Phase 4 `eprintln!` deprecation — same one-release policy for XML tool_call.

### Established Patterns
- Trait + composite parser fits agent crate injection style (`Embedder`, `ChatModel`).
- IPC errors as `String` at Tauri boundary; metadata on `AskResponse` for UI.
- E2E deterministic Mock; `data-testid` kebab-case selectors.

### Integration Points
- `run.rs` loop: replace `Option` parse with outcome-driven branch (Found → tool exec; Failed → warnings + break; NotFound → final answer).
- `orchestrate.rs` / `router.rs`: accumulate warnings from each step into merged response.
- `agent_response_to_ask` in `lib.rs` (moves to `commands/agent.rs`): map `AgentResponse.tool_parse_warnings`.

</code_context>

<specifics>
## Specific Ideas

- User wants **strictest practical JSON contract** — bare line only; fences and multi-object replies are errors, not tolerated formats.
- Parse failures should be **visible but non-blocking** — show what the model said + amber warning; no silent swallow, no automatic retry loop.
- XML compatibility mirrors **Phase 4 memory deprecation**: one release + stderr log, not indefinite dual protocol.

</specifics>

<deferred>
## Deferred Ideas

- Auto-retry agent loop feeding parse error back to model (user chose stop-on-failed).
- Per-step warning display inside orchestration step cards.
- Tauri event stream for warnings during `ask_agent_in_session_stream`.
- Native OpenAI function-calling / `tool_calls` API integration.
- Removing XML fallback in this phase (explicitly after one-release window).

</deferred>

---

*Phase: 5-Agent Protocol*
*Context gathered: 2026-06-27*
