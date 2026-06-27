# Phase 5: Agent Protocol — Research

**Phase:** 5-Agent Protocol  
**Version target:** v1.9.4  
**Requirements:** AGT-01, AGT-02, AGT-03, SHELL-05  
**Research date:** 2026-06-27

---

## User Constraints

> From `05-CONTEXT.md` (synthesized from ROADMAP + REQUIREMENTS). Decisions D-01 through D-19 locked.

### Tool Protocol (AGT-01, AGT-02)
- **D-01..D-06:** `ToolCallParser` trait; JSON primary; XML fallback with deprecation log; parse failures → warnings metadata, not silent None.

### UI (AGT-02)
- **D-07..D-09:** `AskResponse.tool_parse_warnings`; ChatView panel `agent-tool-parse-warnings`; preserve `agent-tool-calls`.

### Mock/E2E (AGT-01, AGT-03)
- **D-10..D-12:** Mock emits JSON line; prompt updated; E2E deterministic.

### Shell (SHELL-04, SHELL-05)
- **D-13..D-16:** `commands/agent.rs`; E2E bootstrap IPC unchanged; config.rs verify only.

---

## Current State [VERIFIED]

### XML-only parser

```23:28:crates/agent/src/tools.rs
pub fn parse_tool_call(text: &str) -> Option<ToolCallPayload> {
    let start = text.find("<tool_call>")?;
    let end = text.find("</tool_call>")?;
    let json = text[start + 11..end].trim();
    serde_json::from_str(json).ok()
}
```

**Problem:** Returns `None` on any failure — indistinguishable from "no tool call" (AGT-02 violation). Malformed XML/JSON is silently treated as final answer.

### Agent loop

```73:75:crates/agent/src/run.rs
    for _ in 0..MAX_TOOL_ROUNDS {
        let reply = chat.complete(&messages).await?;
        if let Some(payload) = parse_tool_call(&reply) {
```

### Mock agent tool response

```50:53:crates/llm/src/mock.rs
        return Ok(
            r#"<tool_call>{"name":"search_knowledge","arguments":{"query":"agent"}}</tool_call>"#
                .to_string(),
        );
```

### Tools prompt

`build_tools_prompt` shows only XML examples (lines 201–212 in tools.rs).

### Agent IPC still in lib.rs [VERIFIED]

Not yet in `commands/agent.rs` (file does not exist):
- `list_agent_profiles`, `list_skills`, `list_hooks`, `list_plugins`
- `upsert_agent_profile`, `remove_agent_profile`, `set_active_agent`
- `ask_agent_in_session`, `ask_agent_in_session_stream`
- Helpers: `execute_agent_question`, `build_agent_context`, `agent_response_to_ask`, `run_ask_agent_in_session`
- Resolvers: `ConfigChatResolver`, `ConfigEmbedResolver`

`commands/config.rs` **exists** (Phase 3) with get/set config, API key, index status.

### UI

- `ChatView` renders `agent-tool-calls` when `lastToolCalls.length > 0` (line 246)
- `App.tsx` sets `lastToolCalls` from `resp.tool_calls` after agent ask
- No parse-warning surface today

### AskResponse shape

```28:36:crates/rag/src/types.rs
pub struct AskResponse {
    pub answer: String,
    pub citations: Vec<Citation>,
    pub tool_calls: Vec<ToolCallInfo>,
    pub orchestration_steps: Vec<OrchestrationStepInfo>,
}
```

Needs `tool_parse_warnings: Vec<String>` with `#[serde(default)]`.

---

## Target Architecture

### ToolCallParser module (`crates/agent/src/tool_parse.rs`)

```rust
pub enum ToolCallParseOutcome {
    Found(ToolCallPayload),
    NotFound,
    Failed { message: String },
}

pub trait ToolCallParser: Send + Sync {
    fn id(&self) -> &str;
    fn parse(&self, text: &str) -> ToolCallParseOutcome;
}

// JsonToolCallParser — primary
// XmlToolCallParser — legacy (from current parse_tool_call)
// CompositeToolCallParser — JSON then XML; log on XML hit
```

**JSON detection heuristics:**
1. Trim; if whole string parses as `{"name":...,"arguments":...}` → Found
2. First line that looks like JSON object with required keys
3. Optional ```json code fence

**Failed vs NotFound:**
- If text contains `<tool_call>` or `{` with `"name"` but parse fails → Failed
- Otherwise → NotFound

### AgentResponse extension

```rust
pub struct AgentResponse {
    // existing fields...
    #[serde(default)]
    pub tool_parse_warnings: Vec<String>,
}
```

Populate in `run.rs` when outcome is Failed; accumulate across rounds if needed.

### Prompt migration

Update `build_tools_prompt` header:
```
需要工具时仅回复一行 JSON（推荐）：
{"name":"search_knowledge","arguments":{"query":"关键词"}}
旧格式（已弃用）：<tool_call>...</tool_call>
```

---

## Pitfalls

### P1: Silent None regression [HIGH]
Replacing `Option` without Failed path recreates bug. Must test malformed JSON returns warning.

### P2: Mock/E2E drift [HIGH]
Mock must switch to JSON in same PR as JsonParser; otherwise agent loop never exercises primary path in CI.

### P3: Breaking orchestration [MEDIUM]
`run_orchestrated` / `run_routed` call `run_agent` internally — parser change is centralized in `run.rs` (good).

### P4: IPC relocation drift [HIGH]
`generate_handler!` must list identical command names after move to `commands/agent.rs`.

### P5: AskResponse serde compat [LOW]
`#[serde(default)]` on new field keeps old persisted messages deserializable.

### P6: OpenAI cloud variance [MEDIUM]
Cloud may wrap JSON in prose; JsonParser should extract first valid object, not require whole-string JSON.

---

## Validation Architecture

### Deliverable 1: ToolCallParser + run.rs

| Layer | Test | Location |
|-------|------|----------|
| Unit | JSON bare line parses | `crates/agent/src/tool_parse.rs` |
| Unit | XML fallback + deprecation | same |
| Unit | Malformed → Failed not NotFound | same |
| Integration | `run_agent` with Mock JSON | `crates/agent/src/run.rs` tests |
| Integration | Existing `agent_uses_search_tool` green | run.rs |

### Deliverable 2: Metadata + UI

| Layer | Test | Location |
|-------|------|----------|
| Unit | AskResponse deserializes without warnings field | rag types test |
| FE | ChatView renders warnings testid | optional Vitest or E2E |
| E2E | agent.spec.ts green | `e2e/specs/agent.spec.ts` |

### Deliverable 3: commands/agent.rs

| Layer | Test | Location |
|-------|------|----------|
| Compile | cargo build src-tauri | CI |
| E2E | start_ask_e2e unchanged | existing specs |

---

## Plan Wave Structure (MVP)

| Wave | Plan | Focus |
|------|------|-------|
| 1 | 05-01 | ToolCallParser trait + run.rs + AgentResponse warnings |
| 2 | 05-02 | Mock JSON + AskResponse + ChatView warnings UI |
| 3 | 05-03 | commands/agent.rs relocation |
| 4 | 05-04 | E2E gate + SHELL-05 verification |

---

*Research complete — ready for planning.*
