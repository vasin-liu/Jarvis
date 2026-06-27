# Phase 5: Agent Protocol - Research

**Researched:** 2026-06-27  
**Domain:** Agent tool-call parsing, IPC shell split, Chat UI metadata  
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### JSON Tool Call Format (AGT-01)
- **D-01:** **Primary format:** strict **one bare JSON line** per tool call: `{"name":"...","arguments":{...}}`. Trim leading/trailing whitespace on the line before parse.
- **D-02:** **No markdown fence acceptance:** if model uses ` ```json ` instead of bare line → **Failed** + `tool_parse_warnings` (do not strip fence; do not treat as NotFound).
- **D-03:** **Multiple JSON objects** in one reply → **Failed** + warnings (consistent with single-line contract).
- **D-04:** **Schema (strict):** require `name` (string) + `arguments` (object). Extra keys ignored. Missing/invalid types → **Failed**.
- **D-05:** **`build_tools_prompt`:** document **JSON line only** — remove XML examples from primary instructions (Mock/E2E use JSON path).

#### Parser Architecture (AGT-01, AGT-02)
- **D-06:** `ToolCallParser` trait: `parse(text) -> ToolCallParseOutcome` = `Found(payload) | NotFound | Failed { error, raw_snippet }`.
- **D-07:** `JsonToolCallParser` implements D-01..D-04. `XmlToolCallParser` holds legacy `<tool_call>...</tool_call>` logic.
- **D-08:** `CompositeToolCallParser`: try JSON first, then XML fallback. `run.rs` uses composite (not `Option`-only silent failure).
- **D-09:** **Failed vs NotFound:** use **intent heuristic** — Failed when `<tool_call>`, ` ```json `, or a line starting with `{"name"` is present but parsing fails; pure natural language with no tool intent → **NotFound** (normal final answer).
- **D-10:** Invalid JSON **inside** valid `<tool_call>` tags → **Failed** + warnings (same as JSON path).

#### Parse Failure UX (AGT-02)
- **D-11:** On **Failed:** **stop agent tool loop** for that turn; treat reply as **final answer** (no auto-retry, no error-as-tool-result loop).
- **D-12:** Still **display model raw text** as `answer` alongside warnings (do not hide or replace with generic error string).
- **D-13:** Chat UI: inline **amber panel** below assistant message when `tool_parse_warnings.length > 0`, `data-testid="agent-tool-parse-warnings"`. Existing `agent-tool-calls` unchanged on successful parse.
- **D-14:** Extend `AskResponse` with `#[serde(default)] tool_parse_warnings: Vec<String>`; wire through `agent_response_to_ask`. **Both** `ask_agent_in_session` and `ask_agent_in_session_stream` return warnings in response metadata (no separate Tauri event this phase).

#### XML Fallback (one-release, aligned with Phase 4)
- **D-15:** Keep XML fallback **one release**; remove in a future release after deprecation window (same policy as Phase 4 memory fuzzy match).
- **D-16:** When XML fallback matches: **`eprintln!`** one line: `[jarvis] XML tool_call format deprecated; use JSON line`.
- **D-17:** **Tests:** unit/integration tests for `XmlToolCallParser` in `cargo test -p agent`; **E2E covers JSON path only** (no dedicated XML fallback E2E).

#### Mock & Prompt (AGT-03)
- **D-18:** `MockChatModel` agent path emits **JSON line** (no XML wrapper) for tool round.
- **D-19:** E2E continues Mock provider (`JARVIS_E2E=1`) — no live LLM in CI.

#### Orchestration Warnings (pipeline / router)
- **D-20:** **Merge all steps'** `tool_parse_warnings` into final `AskResponse.tool_parse_warnings`.
- **D-21:** Prefix each merged warning with **`[agent_name]:`** (use profile display name, not id).
- **D-22:** Show merged warnings **only on final assistant message** — not inside per-step orchestration cards.

#### Tauri Command Split (SHELL-04, SHELL-05)
- **D-23:** Create `commands/agent.rs`; move agent/profile/skill/hook/plugin IPC + helpers from `lib.rs` (list from synthesized plan 05-03).
- **D-24:** **`start_ask_e2e` stays in `lib.rs`** (or `commands/chat.rs`) — do not break E2E IPC names.
- **D-25:** Thin wrappers only; orchestration stays in `crates/agent`. `commands/config.rs` verified complete.

#### Quality Gates
- **D-26:** `cargo test -p agent` + `npm test` + `npx tsc --noEmit` required.
- **D-27:** `agent.spec.ts` passes with structured JSON Mock responses.
- **D-28:** Preserve agent testids: `agent-mode-toggle`, `agent-tool-calls`, `agent-tool-parse-warnings`, `orchestration-mode-*`.

### Claude's Discretion
- Exact line-splitting heuristic (single line vs first non-empty line in multi-line reply).
- `tool_parse_warnings` message wording and raw_snippet truncation length.
- OpenAI cloud json_object prompt tweak scope (minimal — parser must enforce bare line contract).
- Whether composite parser lives in `tools.rs` or new `parse.rs` module.

### Deferred Ideas (OUT OF SCOPE)
- Auto-retry agent loop feeding parse error back to model (user chose stop-on-failed).
- Per-step warning display inside orchestration step cards.
- Tauri event stream for warnings during `ask_agent_in_session_stream`.
- Native OpenAI function-calling / `tool_calls` API integration.
- Removing XML fallback in this phase (explicitly after one-release window).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| **AGT-01** | `ToolCallParser` trait with structured JSON path for Mock and OpenAI-compatible providers | New `tool_parse.rs` (or `parse.rs`) with `JsonToolCallParser`, `XmlToolCallParser`, `CompositeToolCallParser`; `run.rs` outcome-driven loop; `build_tools_prompt` JSON-only examples; existing `serde_json` for validation — no new crate |
| **AGT-02** | Tool parse failures surfaced to UI (not silent `None`); XML parser retained as fallback during migration | `ToolCallParseOutcome::Failed` → `AgentResponse.tool_parse_warnings` → `AskResponse.tool_parse_warnings` → ChatView amber panel `agent-tool-parse-warnings`; XML fallback + `eprintln!` per Phase 4 pattern |
| **AGT-03** | E2E agent specs pass with structured tool call format | `MockChatModel` emits bare JSON line when `system.contains("可用工具")`; unit/integration tests in `cargo test -p agent`; `agent.spec.ts` + `full-ui.spec.ts` agent sections green under `JARVIS_E2E=1` |
| **SHELL-05** | Config commands in `commands/config.rs`; E2E bootstrap path unchanged in `e2e.rs` | `commands/config.rs` already complete (Phase 3); create `commands/agent.rs` and re-export in `commands/mod.rs` + `generate_handler!`; keep `start_ask_e2e` in `lib.rs`; do not edit `e2e.rs` unless IPC contract breaks |
</phase_requirements>

## Summary

Phase 5 replaces the agent crate's fragile XML-wrapped JSON parsing with a **strict bare-line JSON protocol**, surfaces parse failures through response metadata and a new Chat UI panel, and completes the Tauri shell split by moving ~270 lines of agent IPC from `lib.rs` into `commands/agent.rs`. The work is entirely in-process refactor: **no new external dependencies**, no SQLite schema changes, and no Tauri command renames.

Current code returns `Option<ToolCallPayload>` from `parse_tool_call()` — malformed tool intent is indistinguishable from a normal final answer (AGT-02 violation). The agent loop in `run.rs` branches on `if let Some(payload) = parse_tool_call(&reply)`, so any parse failure silently ends the tool round as if the model finished answering. Mock and E2E still emit `<tool_call>...</tool_call>` (see `crates/llm/src/mock.rs` lines 50–53), and `build_tools_prompt` documents XML-only examples.

Locked decisions require the **strictest practical contract**: bare JSON line only; markdown fences and multi-object replies are **Failed**, not tolerated. On Failed, the loop stops, raw model text remains the answer, and warnings flow to the UI. XML stays one release with stderr deprecation, matching Phase 4 memory fuzzy-match policy.

**Primary recommendation:** Implement `ToolCallParser` + composite in `crates/agent`, extend `AgentResponse`/`AskResponse` with `tool_parse_warnings`, update Mock + prompt in the same wave as parser tests, then relocate agent IPC to `commands/agent.rs` without changing invoke names.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Tool call parsing & loop control | **API / Backend** (`crates/agent`) | — | Domain logic; testable without Tauri |
| Mock / prompt protocol shape | **API / Backend** (`crates/llm`, `crates/agent/tools.rs`) | — | Providers emit text; parser enforces contract |
| Parse warnings metadata | **API / Backend** (`crates/rag::AskResponse`, `AgentResponse`) | Tauri mapping in `commands/agent.rs` | Serde IPC payload; thin shell maps fields |
| Warnings + tool-call UI | **Browser / Client** (`ChatView.tsx`, `App.tsx` state) | — | Renders `tool_parse_warnings` / `tool_calls` from invoke response |
| Agent IPC commands | **API / Backend** (`src-tauri/commands/agent.rs`) | `lib.rs` registration only | Thin `State<AppState>` wrappers |
| E2E bootstrap | **API / Backend** (`lib.rs` `start_ask_e2e`) | `e2e.rs` config | Must stay stable per D-24 / SHELL-05 |
| Orchestration warning merge | **API / Backend** (`orchestrate.rs`, `router.rs`) | — | Accumulate per-step warnings before IPC mapping |

## Project Constraints (from CLAUDE.md)

- TDD required: failing tests first, then implementation; user-facing changes need E2E updates.
- E2E mandatory for user-facing features; deterministic mocks (`JARVIS_E2E=1`); no live LLM in CI.
- Surgical diffs; no scope beyond refactor; preserve IPC signatures and `data-testid` contracts.
- Only `crates/store` opens SQLite; agent logic stays in `crates/agent`.
- Quality gates: `cargo test --workspace`, `npm test`, `npm run test:e2e:local` after user-facing changes.
- Stack unchanged: Rust stable, Tauri 2, React 19, `serde`/`serde_json` for serialization.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `serde` / `serde_json` | workspace `1` | Parse/validate tool JSON; IPC payloads | Already used in `tools.rs`, `AskResponse`; no new parser crate |
| `thiserror` | workspace `1` | `AgentError` variants | Existing agent crate pattern |
| `async-trait` | workspace `0.1` | `ChatModel`, future parser trait ergonomics | Existing provider pattern |
| Vitest | `^3` (`vite.config.ts`) | Frontend unit tests | Project standard |
| WebdriverIO | `^9.20.0` | E2E agent journeys | `.cursor/rules/e2e-required.mdc` |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tempfile` | dev-dep in agent tests | Isolated DB for integration tests | Existing `run.rs` / `orchestrate.rs` tests |
| `MockChatModel` / `MockEmbedder` | in-repo | Deterministic agent loop | All agent unit tests + E2E |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-rolled JSON line parser | `schemars` / JSON Schema crate | Locked decision: minimal deps; `serde_json` + manual field checks sufficient |
| Native OpenAI `tool_calls` API | JSON-in-text protocol | **Out of scope** per CONTEXT deferred ideas |
| Tauri event for stream warnings | Metadata on final `AskResponse` | **Deferred** per D-14 |

**Installation:** None — no new packages for this phase.

**Version verification:** Workspace pins confirmed in root `Cargo.toml`; no registry lookup required.

## Package Legitimacy Audit

> Phase installs **no new external packages**. Refactor uses existing workspace dependencies only.

| Package | Registry | Verdict | Disposition |
|---------|----------|---------|-------------|
| *(none)* | — | — | N/A |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** none

## Architecture Patterns

### System Architecture Diagram

```
User question (Chat UI)
        │
        ▼
┌───────────────────────────────────────┐
│  Tauri IPC (unchanged names)          │
│  ask_agent_in_session[_stream]        │
│  commands/agent.rs (after move)       │
└───────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────┐
│  execute_agent_question               │
│  Single │ Pipeline │ Router           │
└───────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────┐
│  run_agent / run_orchestrated /       │
│  run_routed  (crates/agent)           │
└───────────────────────────────────────┘
        │
        ▼
   chat.complete(reply)
        │
        ▼
┌───────────────────────────────────────┐
│  CompositeToolCallParser              │
│  1. JsonToolCallParser (strict line)  │
│  2. XmlToolCallParser (fallback+log)  │
└───────────────────────────────────────┘
        │
   ┌────┴────┬──────────┐
   ▼         ▼          ▼
 Found    NotFound    Failed
   │         │          │
   ▼         ▼          ▼
 execute   final     stop loop;
 tool      answer    warnings +=
 loop                  raw answer
   │                    │
   └──────────┬─────────┘
              ▼
     AgentResponse
     (answer, tool_calls,
      tool_parse_warnings, …)
              │
              ▼
     agent_response_to_ask
              │
              ▼
     AskResponse → ChatView
     (agent-tool-calls /
      agent-tool-parse-warnings)
```

### Recommended Project Structure

```
crates/agent/src/
├── tool_parse.rs      # NEW: ToolCallParser trait + Json/Xml/Composite
├── tools.rs           # execute_tool, build_tools_prompt (JSON examples)
├── run.rs             # outcome-driven loop (Found/NotFound/Failed)
├── orchestrate.rs     # merge tool_parse_warnings with [agent_name]:
├── router.rs          # pass-through from run_agent
└── types.rs           # AgentResponse.tool_parse_warnings

crates/rag/src/types.rs    # AskResponse.tool_parse_warnings
crates/llm/src/mock.rs   # bare JSON line for agent tool round

src-tauri/src/
├── commands/agent.rs  # NEW: agent IPC + execute_agent_question helpers
├── commands/config.rs # VERIFY ONLY (already complete)
└── lib.rs             # generate_handler! + start_ask_e2e + init seeds

src/
├── App.tsx            # lastToolParseWarnings state
├── types/ipc.ts       # AskResponse.tool_parse_warnings
└── views/ChatView.tsx # agent-tool-parse-warnings panel
```

### Pattern 1: Outcome-driven parser (replaces `Option`)

**What:** Three-way parse result distinguishes success, no tool intent, and parse failure.  
**When to use:** Every `chat.complete` reply in `run.rs` tool loop.

**Example:**

```rust
// Pattern aligned with 05-CONTEXT.md D-06..D-11 [VERIFIED: codebase + CONTEXT]
pub enum ToolCallParseOutcome {
    Found(ToolCallPayload),
    NotFound,
    Failed { error: String, raw_snippet: String },
}

match composite.parse(&reply) {
    ToolCallParseOutcome::Found(payload) => { /* execute tool, continue loop */ }
    ToolCallParseOutcome::NotFound => { /* return AgentResponse with answer = reply */ }
    ToolCallParseOutcome::Failed { error, raw_snippet } => {
        warnings.push(format!("{error}: {raw_snippet}"));
        break; // final answer = reply (D-11, D-12)
    }
}
```

### Pattern 2: Thin Tauri command module (mirror `commands/memory.rs`)

**What:** `#[tauri::command]` fns take `State<AppState>`, delegate to crates, map errors to `String`.  
**When to use:** Relocating agent IPC from `lib.rs` (D-23).

**Example:**

```rust
// Mirror: src-tauri/src/commands/memory.rs [VERIFIED: codebase]
#[tauri::command]
pub async fn ask_agent_in_session(
    session_id: String,
    question: String,
    state: State<'_, AppState>,
) -> Result<AskResponse, String> {
    run_ask_agent_in_session(state.inner(), &session_id, &question).await
}
```

### Pattern 3: Phase 4 deprecation stderr

**What:** One-line `eprintln!` when legacy path matches; no user-visible error.  
**When to use:** XML fallback hit (D-16).

**Example:**

```rust
// Reference: crates/memory/src/learn.rs [VERIFIED: codebase]
eprintln!("[jarvis] XML tool_call format deprecated; use JSON line");
```

### Anti-Patterns to Avoid

- **Silent `None` on parse failure:** Recreates AGT-02 bug; must use `Failed`.
- **Stripping markdown fences:** Contradicts D-02; fences → `Failed`.
- **Auto-retry on Failed:** Explicitly deferred; stop loop per D-11.
- **Renaming Tauri commands during move:** Breaks frontend `invoke` without TS compile check.
- **Mixing parser change with unrelated `lib.rs` cleanup:** Increases debug surface (PITFALLS §13).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON parsing | Custom lexer | `serde_json::from_str` + manual `name`/`arguments` type checks | Edge cases (unicode, nesting) already handled |
| IPC DTO extension | Parallel warning channel | `AskResponse.tool_parse_warnings` with `#[serde(default)]` | Single response path for stream + non-stream |
| XML legacy support | New format | Extract existing `parse_tool_call` into `XmlToolCallParser` | Behavior parity for valid XML during one-release window |
| Full JSON Schema validation | `schemars` / external schema | Required-field check on `serde_json::Value` | Two fields only; locked minimal scope |

**Key insight:** The complexity is in **contract enforcement and UX**, not JSON parsing — keep `serde_json` and invest tests in Failed vs NotFound heuristics.

## Runtime State Inventory

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| **Stored data** | `chat_messages` stores `content` + `citations_json` only (`persist_assistant` in `commands/chat.rs`); **no** persisted `tool_calls` or `tool_parse_warnings` | **Code edit only** — warnings are ephemeral UI metadata for the active turn; reloading session will not show historical warning panels (acceptable per phase scope) |
| **Stored data** | `config.json` agent profiles, skills/hooks/plugins on disk under `app_data/skills|hooks|plugins` | **No migration** — protocol change does not alter stored config shape |
| **Live service config** | None — agent protocol is in-app only | None — verified by codebase audit |
| **OS-registered state** | None | None — verified |
| **Secrets/env vars** | E2E uses `JARVIS_E2E=1`, `JARVIS_E2E_FIXTURE`; unchanged per SHELL-05 | **Do not change** `e2e.rs` bootstrap unless IPC break |
| **Build artifacts** | `target/release/tauri-app.exe` for E2E | Rebuild via `npm run build:e2e` after Rust changes — standard CI path |

**Canonical post-refactor question:** After moving IPC to `commands/agent.rs`, runtime still uses same invoke names and Mock provider — no external system caches old XML format.

## Common Pitfalls

### Pitfall 1: Silent None regression

**What goes wrong:** Replacing `parse_tool_call` with a parser that still collapses errors to `NotFound` hides malformed tool calls.  
**Why it happens:** Convenience reuse of `ok()` / `Option` chaining.  
**How to avoid:** Unit tests for fence, multi-object, bad schema → `Failed`; assert warnings populated.  
**Warning signs:** Agent answers with gibberish JSON in message body; no amber panel.

### Pitfall 2: Mock/E2E drift

**What goes wrong:** Parser expects JSON but Mock still emits XML → CI never exercises primary path.  
**Why it happens:** Splitting parser and Mock across PRs/waves without coordination.  
**How to avoid:** D-18 in same wave as `JsonToolCallParser`; grep mock for `<tool_call>`.  
**Warning signs:** `agent_uses_search_tool` passes via XML fallback only; stderr shows deprecation every E2E run.

### Pitfall 3: E2E `start_ask_e2e` metadata loss

**What goes wrong:** E2E path in `App.tsx` resolves `jarvis-ask-done` with hardcoded `tool_calls: []` (lines 366–371) — tool panels never populate in primary E2E flow even on success.  
**Why it happens:** Event payload carries messages only, not full `AskResponse`.  
**How to avoid:** AGT-03 satisfied by agent **answer** assertions today; optional follow-up: extend payload or assert warnings via non-E2E invoke in integration tests. Do **not** break `start_ask_e2e` contract (D-24).  
**Warning signs:** New E2E asserting `agent-tool-calls` fails while manual dev works.

### Pitfall 4: IPC relocation handler drift

**What goes wrong:** `generate_handler!` omits a moved command → runtime "command not found".  
**Why it happens:** Manual list out of sync with `commands/mod.rs` re-exports.  
**How to avoid:** Move commands + update `mod.rs` + `lib.rs` handler in one task; `cargo build`.  
**Warning signs:** Settings agent list works but ask fails.

### Pitfall 5: Orchestration warning placement

**What goes wrong:** Warnings rendered inside `orchestration-steps` cards violate D-22.  
**Why it happens:** Reusing step UI for errors.  
**How to avoid:** Merge in `run_orchestrated` with `[agent_name]:` prefix; single panel on final message.  
**Warning signs:** Duplicate warnings per pipeline step.

### Pitfall 6: Strict JSON vs cloud model prose

**What goes wrong:** Cloud models wrap JSON in explanation text → strict whole-string parse fails.  
**Why it happens:** OpenAI-compatible chat returns natural language + JSON.  
**How to avoid:** Discretion: first **non-empty line** heuristic (D-09 intent + Claude's discretion) — still **reject** fences per D-02; document in parser tests.  
**Warning signs:** Real cloud agent mode always shows warnings; Mock/E2E fine.

## Code Examples

### Current XML parser (to extract)

```rust
// crates/agent/src/tools.rs — [VERIFIED: codebase]
pub fn parse_tool_call(text: &str) -> Option<ToolCallPayload> {
    let start = text.find("<tool_call>")?;
    let end = text.find("</tool_call>")?;
    let json = text[start + 11..end].trim();
    serde_json::from_str(json).ok()
}
```

### Target Mock agent tool response

```rust
// crates/llm/src/mock.rs — [VERIFIED: CONTEXT D-18]
return Ok(r#"{"name":"search_knowledge","arguments":{"query":"agent"}}"#.to_string());
```

### JsonToolCallParser strict line check

```rust
// Recommended implementation shape [VERIFIED: CONTEXT D-01..D-04]
let line = text.trim();
if line.contains("```") {
    return ToolCallParseOutcome::Failed {
        error: "markdown code fence not allowed".into(),
        raw_snippet: truncate(line),
    };
}
let value: serde_json::Value = match serde_json::from_str(line) {
    Ok(v) => v,
    Err(e) => return failed_with_intent(text, e.to_string()),
};
// Require name: string, arguments: object; reject multiple top-level objects in line
```

### AskResponse extension

```rust
// crates/rag/src/types.rs — [VERIFIED: CONTEXT D-14]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AskResponse {
    pub answer: String,
    pub citations: Vec<Citation>,
    #[serde(default)]
    pub tool_calls: Vec<ToolCallInfo>,
    #[serde(default)]
    pub orchestration_steps: Vec<OrchestrationStepInfo>,
    #[serde(default)]
    pub tool_parse_warnings: Vec<String>,
}
```

### Agent IPC inventory to move (`lib.rs` → `commands/agent.rs`)

| Symbol | Kind | Notes |
|--------|------|-------|
| `list_agent_profiles` | command | |
| `list_skills` | command | uses `load_skills_from_dir` |
| `list_hooks` | command | |
| `list_plugins` | command | |
| `set_active_agent` | command | |
| `remove_agent_profile` | command | |
| `upsert_agent_profile` | command | |
| `ask_agent_in_session` | command | |
| `ask_agent_in_session_stream` | command | Channel streaming |
| `run_ask_agent_in_session` | helper | shared with `start_ask_e2e` |
| `execute_agent_question` | helper | orchestration dispatch |
| `build_agent_context` | helper | |
| `agent_response_to_ask` | helper | add `tool_parse_warnings` mapping |
| `resolve_active_agent` | helper | |
| `ConfigChatResolver` / `ConfigEmbedResolver` | types | keep private to module |
| `seed_skills_dir` / `seed_hooks_dir` / `seed_plugins_dir` | init | **Optional stay in `lib.rs` init** (discretion) |

**Stay in `lib.rs`:** `start_ask_e2e`, `is_e2e_mode_cmd`, `init_state` seed calls (D-24).

**Verify only:** `commands/config.rs` — `get_config`, `set_config`, `get/set/clear_api_key`, `get_index_status` already present [VERIFIED: codebase].

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| XML-wrapped JSON in prompt + Mock | Bare JSON line strict | Phase 5 (v1.9.4) | Primary parse path; XML fallback one release |
| `Option` parse → silent failure | `ToolCallParseOutcome` + UI warnings | Phase 5 | AGT-02 compliance |
| Agent IPC in `lib.rs` | `commands/agent.rs` | Phase 5 | SHELL-04/05 completion |
| Memory title fuzzy match | UUID URI + deprecation log | Phase 4 (done) | Template for XML deprecation policy |

**Deprecated/outdated:**
- XML-only `build_tools_prompt` examples — remove from primary instructions (D-05).
- `parse_tool_call` as public API — superseded by `ToolCallParser` trait; keep thin re-export or deprecate internally.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | First non-empty line heuristic suffices for cloud models without accepting fences | Parser design | Cloud users see frequent warnings until prompt tuning |
| A2 | Ephemeral warnings (not persisted to SQLite) is acceptable UX | Runtime inventory | Users expect warnings after reload — would need schema change |
| A3 | `SHELL-04` marked complete in REQUIREMENTS.md is aspirational; `commands/agent.rs` does not exist yet | SHELL-05 | Planner must include file creation, not verify-only |
| A4 | E2E can pass AGT-03 with answer-length assertions only (no `agent-tool-calls` assert) | Validation | Tool UI regressions undetected in CI |

**Note:** A3 verified — `Glob **/commands/agent.rs` returned 0 files [VERIFIED: codebase].

## Open Questions

1. **Line selection in multi-line replies**
   - What we know: D-01 requires one bare JSON line; discretion allows first non-empty line vs whole-text trim.
   - What's unclear: Whether assistant preamble + JSON on line 2 is Found or Failed.
   - Recommendation: If any line starts with `{"name"` and exactly one such line validates → Found; multiple candidate lines → Failed (D-03).

2. **`raw_snippet` truncation length**
   - What we know: Discretion to planner; include in warning string shown in UI.
   - Recommendation: Cap at 120 chars to match existing tool result truncation style in ChatView (~400 for results).

3. **Seed helpers location**
   - What we know: D-23 lists IPC + helpers; seeds used only from `init_state`.
   - Recommendation: Keep seeds in `lib.rs` init to minimize diff; move only ask/list/profile commands.

## Environment Availability

Step 2.6: **SKIPPED** — no external dependencies identified. Phase uses existing Rust/Node toolchain only.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (workspace); Vitest 3; WebdriverIO 9 + Mocha |
| Config file | `vite.config.ts` (Vitest); `e2e/wdio.conf.ts` |
| Quick run command | `cargo test -p agent` |
| Full suite command | `cargo test --workspace && npm test && npm run test:e2e:local` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AGT-01 | Bare JSON line → Found | unit | `cargo test -p agent json_tool` | ❌ Wave 0 (`tool_parse.rs` tests) |
| AGT-01 | Fence / multi-object → Failed | unit | `cargo test -p agent json_tool` | ❌ Wave 0 |
| AGT-01 | XML fallback + deprecation | unit | `cargo test -p agent xml_tool` | ❌ Wave 0 |
| AGT-01 | Composite JSON before XML | unit | `cargo test -p agent composite` | ❌ Wave 0 |
| AGT-01 | `run_agent` uses JSON Mock | integration | `cargo test -p agent agent_uses_search_tool` | ✅ `run.rs` |
| AGT-02 | Failed stops loop; warnings on response | unit/integration | `cargo test -p agent parse_failed` | ❌ Wave 0 |
| AGT-02 | Pipeline merges `[name]:` warnings | integration | `cargo test -p agent orchestrate_warnings` | ❌ Wave 0 |
| AGT-02 | UI panel `agent-tool-parse-warnings` | E2E/manual | `npm run test:e2e:local -- --spec e2e/specs/agent.spec.ts` | ❌ Wave 0 (testid + optional assert) |
| AGT-03 | Mock emits JSON not XML | unit | `cargo test -p llm mock_agent` | ❌ Wave 0 (update mock test) |
| AGT-03 | Agent E2E journey green | E2E | `npm run test:e2e:local -- --spec e2e/specs/agent.spec.ts` | ✅ spec exists |
| AGT-03 | Full UI agent section | E2E | `npm run test:e2e:local -- --spec e2e/specs/full-ui.spec.ts` | ✅ spec exists |
| SHELL-05 | `commands/config.rs` complete | compile/review | `cargo build -p tauri-app` | ✅ file exists |
| SHELL-05 | Agent commands relocated | compile | `cargo build -p tauri-app` | ❌ Wave 0 (`commands/agent.rs`) |
| SHELL-05 | E2E bootstrap unchanged | E2E | `npm run test:e2e:local` (uses `start_ask_e2e`) | ✅ path exists |

### Sampling Rate

- **Per task commit:** `cargo test -p agent`
- **Per wave merge:** `cargo test -p agent && npm test && npx tsc --noEmit`
- **Phase gate:** `cargo test --workspace && npm test && npm run test:e2e:local` before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `crates/agent/src/tool_parse.rs` — parser unit tests (AGT-01, AGT-02)
- [ ] `AgentResponse.tool_parse_warnings` + orchestration merge tests (AGT-02)
- [ ] `crates/rag/src/types.rs` serde default test for `tool_parse_warnings` (AGT-02)
- [ ] `crates/llm/src/mock.rs` agent JSON emission + test update (AGT-03)
- [ ] `src/types/ipc.ts` + `ChatView.tsx` `agent-tool-parse-warnings` (AGT-02)
- [ ] `src-tauri/src/commands/agent.rs` + `commands/mod.rs` re-exports (SHELL-05)
- [ ] Optional: E2E assert for warnings panel with injected parse failure (AGT-02) — not required for AGT-03 minimum

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | N/A — local desktop app |
| V3 Session Management | no | Chat sessions local only |
| V4 Access Control | no | No new permission surface |
| V5 Input Validation | **yes** | Strict JSON schema for tool calls; Failed on malformed input; no eval of model text |
| V6 Cryptography | no | No crypto changes |

### Known Threat Patterns for Rust/Tauri agent stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed model output causing panic | Availability | `serde_json` Result handling; no `unwrap()` on parse; Failed outcome |
| Tool argument injection to shell | Elevation | Existing plugin permission checks unchanged (`plugin_has_permissions`) |
| XSS via warning panel | Tampering | React text rendering (no `dangerouslySetInnerHTML`); warnings are plain strings |
| IPC command rename breaking auth boundary | Spoofing | Freeze invoke names (D-23, SHELL-05) |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/05-agent-protocol/05-CONTEXT.md` — locked decisions D-01..D-28
- `crates/agent/src/tools.rs`, `run.rs`, `types.rs` — current parser and loop [VERIFIED: codebase]
- `crates/llm/src/mock.rs` — Mock agent XML emission [VERIFIED: codebase]
- `src-tauri/src/lib.rs` — agent IPC inventory [VERIFIED: codebase]
- `src-tauri/src/commands/config.rs` — SHELL-05 verify target [VERIFIED: codebase]
- `src/views/ChatView.tsx`, `src/App.tsx` — tool UI wiring [VERIFIED: codebase]
- `e2e/specs/agent.spec.ts` — E2E scope [VERIFIED: codebase]
- `.planning/research/PITFALLS.md` §13 — agent protocol migration risks

### Secondary (MEDIUM confidence)

- `.planning/research/ARCHITECTURE.md` S7 — agent protocol slice alignment
- `.planning/phases/04-memory-model-view/04-RESEARCH.md` — one-release deprecation pattern

### Tertiary (LOW confidence)

- None material — phase is codebase-driven refactor

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — no new deps; existing `serde_json` + test harness
- Architecture: **HIGH** — CONTEXT locked; codebase locations verified
- Pitfalls: **HIGH** — E2E metadata gap and Mock drift confirmed in source

**Research date:** 2026-06-27  
**Valid until:** 2026-07-27 (stable refactor domain)
