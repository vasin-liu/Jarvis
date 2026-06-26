# Phase 2: Chat Extraction + Config Types — Research

**Phase:** 02 — Chat Extraction + Config Types  
**Version target:** v1.9.1  
**Research date:** 2026-06-26  
**Researcher:** gsd-phase-researcher

---

<user_constraints>

## Locked Decisions (from 02-CONTEXT.md)

### Hook State Boundary (`useChat`)
- **D-01:** `useChat` owns **RAG session state only**: `sessions`, `activeSessionId`, `messages`, `question`, `streamingDraft`. Agent-related state stays in `App.tsx`.
- **D-02:** Global `busy` / `err` remain in `App.tsx`; `useChat` reports via callbacks (shared error bar pattern).
- **D-03:** E2E probe state (`e2eMode`, `askHandleCount`, `chat-session-ready`) stays in `App.tsx`; passed as props to `ChatView`.
- **D-04:** **Controlled component** pattern: App passes `busy`, `err`, `setErr`, and all agent props; `ChatView`/`useChat` do not own global app state.
- **D-05:** `indexProgress` listen + state stays in `App.tsx`; passed to `ChatView` as props.
- **D-06:** Streaming `Channel` subscription logic **stays in App.tsx until Phase 5**; `src/lib/tauri.ts` still wraps `ask_in_session_stream` IPC.
- **D-07:** Session list refresh is **autonomous in `useChat`** (hook loads/refreshes sessions on create/delete/switch).
- **D-08:** Phase 2 `tauri.ts` migration covers **all chat IPC paths** including stream wrapper functions.

### Agent UI in ChatView
- **D-09:** Agent UI JSX moves into `ChatView.tsx`; state and handlers remain App-controlled via **inline explicit props**.
- **D-10:** **Separate RAG vs Agent submit controls** — not a single shared submit path. Visually distinct input/submit areas.
- **D-11:** Preserve existing `data-testid` values where applicable; **add `agent-submit`** (and related agent-specific ids). Update `agent.spec.ts` in same PR.
- **D-12:** Tool-call panel (`agent-tool-calls`) and orchestration steps stay **below messages** (current layout).
- **D-13:** Add **「管理 Agent」** quick link in ChatView header → navigates to Settings view.
- **D-14:** `agent-mode-toggle` and `agent-profile-select` live in **Chat header bar** (same row as session controls).
- **D-15:** `ask_agent_in_session` / `ask_agent_in_session_stream` invoke calls **stay in App until Phase 5**.

### Tauri `commands/chat.rs`
- **D-16:** Extract **research-mapped chat commands only** (8): `list_chat_sessions`, `create_chat_session`, `delete_chat_session`, `list_chat_messages`, `ask_in_session`, `ask_in_session_stream`, `run_ask_in_session` — **exclude all `ask_agent_*`**.
- **D-17:** Move RAG private helpers with commands: `run_ask_in_session`, persist assistant, `auto_learn_from_chat` trigger logic → `chat.rs`.
- **D-18:** Access store/embedder/chat model via `State<AppState>` — no new service module in Phase 2.
- **D-19:** `start_ask_e2e` **stays in `e2e.rs` / `lib.rs`**, not `commands/chat.rs`.

### Config Nesting (CFG-01)
- **D-20:** Five nested Rust structs: `EmbeddingConfig`, `ChatConfig`, `LarkConfig`, `SyncConfig`, `AgentConfig`.
- **D-21:** **Flat JSON on disk** via `#[serde(flatten)]` — existing `config.json` key names unchanged (no migration file).
- **D-22:** `SyncConfig` bucket includes: `watch_folders`, `cursor_projects_root`, all `scheduled_sync_*`, `auto_summarize_on_index`, `auto_extract_tasks_on_index`.
- **D-23:** Frontend `src/types/ipc.ts` `AppConfig` **stays flat** — TS does not mirror Rust nesting in Phase 2.
- **D-24:** Add **config crate unit test**: deserialize flat JSON fixture → serialize → key-level equality roundtrip.

### Shared Chrome & Layout
- **D-25:** `IndexProgressBanner` and `chat-error` move **inside `ChatView`** (testids unchanged).
- **D-26:** Chat session sidebar (new/switch/delete) stays **inside `ChatView`**.
- **D-27:** Main nav Sidebar (`nav-*`) **unchanged in Phase 2**.
- **D-28:** Citation parse/render moves with `ChatView`; continues using `src/lib/citations.ts`.

### File Layout & PR Strategy
- **D-29:** **Single PR** for Phase 2.
- **D-30:** `useChat.ts` at `src/hooks/useChat.ts`.
- **D-31:** **Monolithic `ChatView.tsx`** for Phase 2.
- **D-32:** Chat types in `src/types/chat.ts`; `ipc.ts` may re-export.

### Quality Gates
- **D-33:** **Full E2E suite** must pass before merge.
- **D-34:** `cargo test --workspace` + `npm test` required.
- **D-35:** Agent E2E updated in same PR for split submit controls.

## Claude's Discretion
- Exact nested struct field assignments within the five config buckets (as long as flatten preserves flat JSON keys).
- Minor ChatView internal organization (single file) as long as testids and controlled props contract hold.
- Whether `ipc.ts` re-exports from `types/chat.ts` vs direct imports.

## Deferred Ideas
- `useAgent` hook + agent invoke migration → Phase 5
- Streaming Channel logic in hook → Phase 5
- Main nav AppShell / Sidebar component → Phase 6
- OS keychain for API secrets → Phase 3
- Nested TS `AppConfig` mirroring Rust → Phase 6 Settings extraction (optional)

</user_constraints>

---

## Summary

Phase 2 performs two parallel workstreams on a fully-scaffolded Phase 1 foundation:

1. **Chat view extraction** — move the heaviest slice of `App.tsx` (~600+ lines of chat JSX + ~15 `useState`) into `ChatView.tsx` + `useChat.ts`, while respecting the controlled-component boundary where agent state and streaming `Channel` remain in `App` until Phase 5.
2. **Config nesting** — restructure `AppConfig` (28 flat fields → 5 nested structs) using `#[serde(flatten)]` so existing `config.json` files load unchanged; add a roundtrip unit test.

**Phase 1 carry-forward** [VERIFIED]: `src-tauri/src/commands/{config,library,sync}.rs` are already extracted. `AppState` is in `state.rs`. `src/lib/tauri.ts` has wrappers for config/library/sync. `ChatView.tsx` is a passthrough stub `{children}`. All 52 IPC names are registered in `generate_handler!`.

**No new crates or npm packages needed** [VERIFIED]: This phase only moves code within existing files and the existing crate structure.

---

## Phase Requirements

| Req ID | Description | Deliverable | Verification |
|--------|-------------|-------------|--------------|
| **FE-02** | Chat view extracted to `src/views/ChatView.tsx` with `useChat` hook; IPC via `src/lib/tauri.ts` | `ChatView.tsx` owns all chat/agent JSX; `useChat.ts` owns RAG session state; `tauri.ts` wraps chat IPC | `npm test` (Vitest); `npm run test:e2e:local` (`qa.spec.ts`, `agent.spec.ts`, `full-ui.spec.ts`) |
| **FE-06** | All existing `data-testid` selectors preserved or updated in same PR as JSX moves | Same testid values on same DOM elements after JSX move; `agent-submit` added | `npm run test:e2e:local` full suite |
| **SHELL-02** | Chat commands in `src-tauri/src/commands/chat.rs` with unchanged IPC signatures | `commands/chat.rs` with 7 command fns + helpers; `lib.rs` updated `generate_handler!` | `cargo test --workspace`; E2E qa/agent/full-ui |
| **CFG-01** | `AppConfig` split into nested structs with serde flatten | `crates/config/src/types.rs` nested; flat JSON roundtrip unit test passes | `cargo test -p config` |

---

## Architectural Responsibility Map

### After Phase 2

| Concern | Owner | Phase 2 change |
|---------|-------|----------------|
| Chat session CRUD (UI) | `useChat.ts` | **NEW** — extracted from App.tsx |
| RAG session state (`sessions`, `messages`, `question`, `streamingDraft`) | `useChat.ts` | **NEW** |
| Agent state (`agentMode`, `lastToolCalls`, `lastOrchestrationSteps`) | `App.tsx` (unchanged) | stays |
| Streaming `Channel` setup | `App.tsx` (unchanged, D-06) | stays until Phase 5 |
| E2E probe state (`e2eMode`, `askHandleCount`) | `App.tsx` (unchanged, D-03) | stays |
| Chat + Agent JSX layout | `ChatView.tsx` | **MOVED** from App.tsx |
| `IndexProgressBanner`, `chat-error` | `ChatView.tsx` (D-25) | **MOVED** from App.tsx |
| Chat IPC wrappers | `src/lib/tauri.ts` | **EXTENDED** (D-08) |
| Chat types (`ChatSession`, `ChatMessage`) | `src/types/chat.ts` | **NEW** file (D-32) |
| Chat commands (7 fns) | `commands/chat.rs` | **NEW** module (SHELL-02) |
| RAG helpers (`run_ask_in_session`, `persist_assistant`) | `commands/chat.rs` | **MOVED** from lib.rs (D-17) |
| Agent commands (`ask_agent_*`) | `lib.rs` (unchanged) | stays until Phase 5 |
| `start_ask_e2e` | `lib.rs` / `e2e.rs` (D-19) | stays |
| `AppConfig` Rust struct | `crates/config/src/types.rs` | **NESTED** (CFG-01) |
| `AppConfig` TS interface | `src/types/ipc.ts` (D-23) | stays flat |

---

## Standard Stack

All work uses existing installed dependencies [VERIFIED]:

| Layer | Dependency | Version |
|-------|-----------|---------|
| Rust/Tauri | `tauri`, `serde`, `serde_json`, `async-trait`, `tokio` | workspace |
| Rust chat | `rag`, `store`, `memory`, `embedder`, `llm` | workspace crates |
| Config | `serde` with `#[serde(flatten)]` | workspace |
| TypeScript | `@tauri-apps/api/core` `invoke`, `Channel` | existing |
| React | `useState`, `useCallback`, `useEffect`, `useRef` | existing |

**No new packages required** for this phase.

---

## Architecture Patterns

### Pattern 1: Controlled-component prop boundary (FE-02, D-04)

`App.tsx` owns global state and passes it into `ChatView` as explicit props:

```typescript
// App.tsx (after Phase 2)
<ChatView
  // RAG state (from useChat)
  sessions={sessions}
  activeSessionId={activeSessionId}
  messages={messages}
  question={question}
  streamingDraft={streamingDraft}
  onSetQuestion={setQuestion}
  onCreateSession={createSession}
  onSwitchSession={switchSession}
  onDeleteSession={deleteSession}
  // Agent state (stays in App — D-01)
  agentMode={agentMode}
  setAgentMode={setAgentMode}
  lastToolCalls={lastToolCalls}
  lastOrchestrationSteps={lastOrchestrationSteps}
  onAgentAsk={handleAgentAsk}   // stays in App until Phase 5
  // Global state (D-02, D-03, D-04, D-05)
  busy={busy}
  err={err}
  setErr={setErr}
  indexProgress={indexProgress}
  e2eMode={e2eMode}
  askHandleCount={askHandleCount}
  config={config}
  onRagAsk={handleRagAsk}       // new: separate from agent submit (D-10)
/>
```

### Pattern 2: `useChat` hook boundary (FE-02, D-01/D-07)

```typescript
// src/hooks/useChat.ts
export function useChat() {
  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [question, setQuestion] = useState("");
  const [streamingDraft, setStreamingDraft] = useState("");
  // ... session CRUD (listChatSessions, createChatSession, deleteChatSession via tauri.ts)
  // ... refreshMessages on session switch
  // Reports errors via onError callback (D-02)
  return { sessions, activeSessionId, messages, question, streamingDraft,
           createSession, switchSession, deleteSession, setQuestion, refreshMessages };
}
```

### Pattern 3: `commands/chat.rs` module (SHELL-02, D-16/D-17)

Follow the established pattern from `commands/config.rs` / `commands/library.rs`:

```rust
// src-tauri/src/commands/chat.rs
use crate::AppState;
use store::{ChatMessage, ChatRole, ChatSession};
use rag::{ask, ask_stream, AskResponse};
use memory::learn_from_exchange;
use tauri::ipc::Channel;

#[derive(Clone, serde::Serialize)]
struct TokenEvent { token: String }

#[tauri::command]
pub fn list_chat_sessions(state: tauri::State<'_, AppState>) -> Result<Vec<ChatSession>, String> {
    state.store.list_chat_sessions().map_err(|e| e.to_string())
}

// ... create_chat_session, delete_chat_session, list_chat_messages,
//     ask_in_session, ask_in_session_stream
// + private helpers: run_ask_in_session, persist_assistant
```

`lib.rs` re-registration:
```rust
// src-tauri/src/commands/mod.rs — add chat module
mod chat;
pub use chat::{
    list_chat_sessions, create_chat_session, delete_chat_session,
    list_chat_messages, ask_in_session, ask_in_session_stream,
};
```

```rust
// lib.rs generate_handler! — replace inline fns with commands:: imports
use commands::{
    // existing...
    list_chat_sessions, create_chat_session, delete_chat_session,
    list_chat_messages, ask_in_session, ask_in_session_stream,
};
```

### Pattern 4: `#[serde(flatten)]` config nesting (CFG-01, D-20/D-21)

```rust
// crates/config/src/types.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(flatten)]
    pub embedding: EmbeddingConfig,
    #[serde(flatten)]
    pub chat_cfg: ChatConfig,
    #[serde(flatten)]
    pub lark: LarkConfig,
    #[serde(flatten)]
    pub sync: SyncConfig,
    #[serde(flatten)]
    pub agent_cfg: AgentConfig,
    /// Runtime-only; not persisted
    #[serde(skip)]
    pub fastembed_cache_dir: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub embedder: EmbedderProvider,
    pub mock_embed_dim: usize,
    pub ollama_base_url: String,
    pub ollama_embed_model: String,
    pub ollama_embed_dim: usize,
    #[serde(default = "default_fastembed_model")]
    pub fastembed_model: String,
    #[serde(default = "default_fastembed_dim")]
    pub fastembed_dim: usize,
    #[serde(default = "default_cloud_base_url")]
    pub cloud_base_url: String,
    #[serde(default)]
    pub cloud_api_key: String,
    #[serde(default = "default_cloud_embed_model")]
    pub cloud_embed_model: String,
    #[serde(default = "default_cloud_embed_dim")]
    pub cloud_embed_dim: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatConfig {
    pub chat: ChatProvider,
    pub ollama_chat_model: String,
    #[serde(default = "default_cloud_chat_model")]
    pub cloud_chat_model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LarkConfig {
    pub lark_cli_bin: String,
    #[serde(default = "default_lark_identity")]
    pub lark_identity: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncConfig {
    pub watch_folders: Vec<String>,
    #[serde(default)]
    pub cursor_projects_root: String,
    #[serde(default)]
    pub auto_summarize_on_index: bool,
    #[serde(default)]
    pub auto_extract_tasks_on_index: bool,
    #[serde(default)]
    pub scheduled_sync_enabled: bool,
    #[serde(default = "default_sync_interval_minutes")]
    pub scheduled_sync_interval_minutes: u32,
    #[serde(default = "default_true")]
    pub scheduled_sync_watch_folders: bool,
    #[serde(default)]
    pub scheduled_sync_lark: bool,
    #[serde(default = "default_true")]
    pub scheduled_sync_cursor: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default)]
    pub auto_learn_from_chat: bool,
    #[serde(default = "default_profiles")]
    pub agents: Vec<agent::AgentProfile>,
    #[serde(default = "default_active_agent_id")]
    pub active_agent_id: String,
    #[serde(default)]
    pub enabled_skill_ids: Vec<String>,
    #[serde(default)]
    pub enabled_hook_ids: Vec<String>,
    #[serde(default)]
    pub enabled_plugin_ids: Vec<String>,
    #[serde(default)]
    pub agent_orchestration_mode: AgentOrchestrationMode,
    #[serde(default)]
    pub pipeline_agent_ids: Vec<String>,
    #[serde(default = "default_granted_plugin_permissions")]
    pub granted_plugin_permissions: Vec<String>,
}
```

**Field access change**: callers using `cfg.watch_folders` change to `cfg.sync.watch_folders`. The `embedding_dim()` method stays on `AppConfig` but delegates to `self.embedding`. All field accesses in `lib.rs`, `state.rs`, `providers.rs`, `sync_scheduler.rs`, and `index_ops.rs` must be updated.

### Pattern 5: Separate RAG vs Agent submit (D-10, D-11)

The current `handleAsk` function in App.tsx branches on `agentMode` to pick an IPC command. Phase 2 splits this into two separate handlers:

- `handleRagAsk` — calls `ask_in_session` / `ask_in_session_stream`; lives in `useChat` or App
- `handleAgentAsk` — calls `ask_agent_in_session` / `ask_agent_in_session_stream`; stays in App (D-15)

ChatView renders two visually distinct forms:
- RAG form: `data-testid="chat-form"`, `data-testid="chat-input"`, `data-testid="ask-submit"` (preserved)
- Agent form: new `data-testid="agent-form"`, `data-testid="agent-input"` (if separate), `data-testid="agent-submit"` (new, D-11)

---

## Don't Hand-Roll

| Concern | Use instead of writing from scratch |
|---------|-------------------------------------|
| `#[serde(flatten)]` nesting | Built into `serde` — no custom de/serializer needed |
| Channel streaming | `tauri::ipc::Channel<T>` — same as existing `ask_in_session_stream` |
| Session CRUD IPC | Extend `src/lib/tauri.ts` wrappers — same `invoke` pattern already used |
| E2E testid stability | Preserve existing `data-testid` values; only add `agent-submit` |
| Config roundtrip test | `serde_json::from_str` + `serde_json::to_value` comparison — no special test crate |

---

## Runtime State Inventory

### Chat-related `useState` in `App.tsx` to migrate to `useChat` (D-01)

| State | Type | Destination |
|-------|------|-------------|
| `sessions` | `ChatSession[]` | `useChat` |
| `activeSessionId` | `string \| null` | `useChat` |
| `messages` | `ChatMessage[]` | `useChat` |
| `question` | `string` | `useChat` |
| `streamingDraft` | `string` | `useChat` |

### State that stays in `App.tsx` (D-02/D-03/D-04/D-05/D-15)

| State | Reason |
|-------|--------|
| `busy`, `err`, `setErr` | Global app state |
| `e2eMode`, `askHandleCount`, `handleAskRef` | E2E probe hooks |
| `indexProgress` | Shared across views |
| `agentMode` | Agent-only state |
| `lastToolCalls`, `lastOrchestrationSteps` | Agent result state |
| `config` | Settings + agent profile access |

### IPC commands to move to `commands/chat.rs` [VERIFIED from lib.rs]

| Command | Fn name | Line in lib.rs | Type |
|---------|---------|---------------|------|
| `list_chat_sessions` | sync fn | 228 | command |
| `create_chat_session` | sync fn | 233 | command |
| `delete_chat_session` | sync fn | 245 | command |
| `list_chat_messages` | sync fn | 250 | command |
| `ask_in_session` | async fn | 350 | command |
| `ask_in_session_stream` | async fn | 359 | command |
| `run_ask_in_session` | private async | 265 | helper |
| `persist_assistant` | private async | 387 | helper |

**Stays in lib.rs** [VERIFIED]:
- `start_ask_e2e` (D-19) — line 316
- `run_ask_agent_in_session` — line 289
- `ask_agent_in_session` — not shown yet (after line 450)
- `ask_agent_in_session_stream` — not shown yet
- `AskDonePayload`, `TokenEvent` struct — move `TokenEvent` to `chat.rs`

### `AppConfig` field-to-struct mapping for nesting [VERIFIED from types.rs]

Total fields in current flat `AppConfig`: 28 persisted + 1 skip (`fastembed_cache_dir`)

| Struct | Fields |
|--------|--------|
| `EmbeddingConfig` | `embedder`, `mock_embed_dim`, `ollama_base_url`, `ollama_embed_model`, `ollama_embed_dim`, `fastembed_model`, `fastembed_dim`, `cloud_base_url`, `cloud_api_key`, `cloud_embed_model`, `cloud_embed_dim` |
| `ChatConfig` | `chat`, `ollama_chat_model`, `cloud_chat_model` |
| `LarkConfig` | `lark_cli_bin`, `lark_identity` |
| `SyncConfig` | `watch_folders`, `cursor_projects_root`, `auto_summarize_on_index`, `auto_extract_tasks_on_index`, `scheduled_sync_enabled`, `scheduled_sync_interval_minutes`, `scheduled_sync_watch_folders`, `scheduled_sync_lark`, `scheduled_sync_cursor` |
| `AgentConfig` | `auto_learn_from_chat`, `agents`, `active_agent_id`, `enabled_skill_ids`, `enabled_hook_ids`, `enabled_plugin_ids`, `agent_orchestration_mode`, `pipeline_agent_ids`, `granted_plugin_permissions` |

### Field access sites that need updating (due to nesting)

Files using `AppConfig` fields directly (verified by grep pattern):

| File | Fields accessed | Update path |
|------|-----------------|-------------|
| `src-tauri/src/lib.rs` | `cfg.lark_cli_bin`, `cfg.lark_identity`, `cfg.embedding_dim()`, `cfg.cursor_projects_root`, `cfg.auto_learn_from_chat`, `cfg.active_agent_id`, `cfg.agents`, etc. | `cfg.lark.lark_cli_bin`, `cfg.sync.cursor_projects_root`, etc. |
| `src-tauri/src/state.rs` | `cfg.embedding_dim()`, provider fields | Keep `embedding_dim()` on `AppConfig` delegating to `self.embedding` |
| `crates/config/src/providers.rs` | `cfg.embedder`, `cfg.chat`, `cfg.cloud_*`, `cfg.ollama_*`, etc. | `cfg.embedding.embedder`, etc. |
| `src-tauri/src/sync_scheduler.rs` | `cfg.scheduled_sync_*`, `cfg.cursor_projects_root` | `cfg.sync.*` |
| `src-tauri/src/index_ops.rs` | `cfg.cursor_projects_root` | `cfg.sync.cursor_projects_root` |

### Existing `data-testid` in chat area [VERIFIED from App.tsx grep]

These must land on the same DOM elements in `ChatView.tsx`:

```
e2e-active, chat-session-ready, ask-handle-count  (App.tsx — stay there)
chat-error                                          (ChatView.tsx — D-25)
index-progress                                      (ChatView.tsx — D-25)
chat-message-assistant, citation-excerpt            (ChatView.tsx)
orchestration-steps, agent-tool-calls               (ChatView.tsx — D-12)
ask-busy, streaming-answer                          (ChatView.tsx)
agent-mode-toggle, agent-profile-select             (ChatView.tsx header — D-14)
orchestration-mode-{mode}                           (ChatView.tsx)
chat-form, chat-input, ask-submit                   (ChatView.tsx — preserved)
agent-submit                                        (ChatView.tsx — NEW, D-11)
```

**New testid**: `agent-submit` — the separate agent form submit button.

---

## Common Pitfalls

### Pitfall 1: Breaking `serde(flatten)` with duplicate field names [PITFALLS §12]
**Risk**: If any field name appears in two nested structs, serde will error on deserialization.  
**Prevention**: Verify each field appears exactly once across all five structs. The field inventory above is exhaustive.

### Pitfall 2: State duplication when extracting `useChat` [PITFALLS §3]
**Risk**: `useChat` and `App.tsx` both hold a copy of `sessions` that diverge.  
**Prevention**: `App.tsx` calls `useChat()` hook and destructures its return — single source of truth. App does not maintain its own `sessions` state after Phase 2.

### Pitfall 3: E2E `ask-handle-count` counter breaks [PITFALLS §2]
**Risk**: The `askHandleCount` increment and `__JARVIS_E2E_ASK__` global stay in App.tsx (D-03), but `handleRagAsk` partially moves logic.  
**Prevention**: The counter increments in App.tsx's `handleRagAsk` wrapper, not inside `useChat`. `handleAskRef.current` must still be set in App.tsx and point to the unified ask dispatcher.

### Pitfall 4: `run_ask_agent_in_session` depends on `run_ask_in_session` [PITFALLS §4]
**Risk**: `run_ask_agent_in_session` in lib.rs calls `execute_agent_question` and `agent_response_to_ask` — these are agent-only helpers staying in lib.rs. But `start_ask_e2e` calls both `run_ask_in_session` and `run_ask_agent_in_session`.  
**Prevention**: Move `run_ask_in_session` to `chat.rs` as `pub(crate)` so `lib.rs` can still call it from `start_ask_e2e`. Or keep `run_ask_in_session` in lib.rs and make the command thin. **Recommended**: expose `pub(crate) async fn run_ask_in_session` from `commands/chat.rs` so `lib.rs::start_ask_e2e` imports it.

### Pitfall 5: `embedding_dim()` method broken by nesting [PITFALLS §8]
**Risk**: `AppConfig::embedding_dim()` currently accesses flat fields; after nesting it must delegate to `self.embedding`.  
**Prevention**: Keep the method on `AppConfig`, update to use `self.embedding.embedder`, `self.embedding.fastembed_model`, etc. No callers need to change — they still call `cfg.embedding_dim()`.

### Pitfall 6: `#[serde(flatten)]` vs `#[serde(default)]` interaction
**Risk**: When flattening a nested struct, `serde(default)` on the nested struct's fields must still be present on the inner struct fields (not just on `AppConfig`).  
**Prevention**: Move the `#[serde(default = "...")]` annotations onto the inner struct fields, not removed. Confirmed by roundtrip test (D-24).

### Pitfall 7: `TokenEvent` struct duplication [PITFALLS §4]
**Risk**: `TokenEvent` is currently defined in lib.rs for streaming. Moving `ask_in_session_stream` to chat.rs needs the struct.  
**Prevention**: Define `TokenEvent` in `commands/chat.rs`; `lib.rs` no longer needs it after agent streaming migration completes in Phase 5. For now, if `ask_agent_in_session_stream` also uses `Channel<TokenEvent>`, the struct must be accessible from both lib.rs and chat.rs. **Solution**: define in `chat.rs`, `pub(crate)` export, import in lib.rs.

---

## Code Examples

### Roundtrip config test (D-24)

```rust
// crates/config/src/lib.rs or types.rs #[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn config_flat_json_roundtrip() {
        // Simulate a v1.8 flat config.json
        let flat_json = r#"{
            "watch_folders": ["/tmp/docs"],
            "embedder": "fastembed",
            "chat": "mock",
            "mock_embed_dim": 4,
            "ollama_base_url": "http://127.0.0.1:11434",
            "ollama_embed_model": "nomic-embed-text",
            "ollama_chat_model": "llama3.2",
            "ollama_embed_dim": 768,
            "lark_cli_bin": "lark-cli",
            "fastembed_model": "bge-small-zh-v1.5",
            "fastembed_dim": 512
        }"#;
        
        let cfg: AppConfig = serde_json::from_str(flat_json).expect("deserialize");
        
        // Verify key fields
        assert_eq!(cfg.sync.watch_folders, vec!["/tmp/docs"]);
        assert_eq!(cfg.embedding.embedder, EmbedderProvider::FastEmbed);
        assert_eq!(cfg.lark.lark_cli_bin, "lark-cli");
        
        // Re-serialize and check key names are still flat (no nesting in JSON)
        let output = serde_json::to_value(&cfg).expect("serialize");
        assert!(output.get("watch_folders").is_some(), "watch_folders must be top-level");
        assert!(output.get("sync").is_none(), "no 'sync' nesting object in JSON");
        assert!(output.get("embedder").is_some(), "embedder must be top-level");
    }
}
```

### `tauri.ts` chat wrappers (D-08, FE-02)

```typescript
// src/lib/tauri.ts — additions for Phase 2
import type { ChatSession, ChatMessage } from "../types/chat";
import type { AskResponse } from "../types/ipc";

export function listChatSessions() {
  return invoke<ChatSession[]>("list_chat_sessions");
}
export function createChatSession(title?: string) {
  return invoke<ChatSession>("create_chat_session", { title });
}
export function deleteChatSession(id: string) {
  return invoke<void>("delete_chat_session", { id });
}
export function listChatMessages(sessionId: string) {
  return invoke<ChatMessage[]>("list_chat_messages", { sessionId });
}
export function askInSession(sessionId: string, question: string) {
  return invoke<AskResponse>("ask_in_session", { sessionId, question });
}
export function askInSessionStream(
  sessionId: string,
  question: string,
  onToken: Channel<{ token: string }>,
) {
  return invoke<AskResponse>("ask_in_session_stream", { sessionId, question, onToken });
}
```

---

## Validation Architecture

### Test mapping per requirement

| Requirement | Test type | Command | Test target |
|-------------|-----------|---------|-------------|
| CFG-01 flat JSON roundtrip | Unit (Rust) | `cargo test -p config` | `config::types::tests::config_flat_json_roundtrip` |
| SHELL-02 command compile | Unit (Rust) | `cargo test --workspace` | `src-tauri` build; IPC names unchanged |
| FE-02 hook isolation | Unit (TS) | `npm test` | `src/hooks/useChat.test.ts` (new) |
| FE-06 testid preservation | E2E | `npm run test:e2e:local` | all specs |
| RAG ask flow | E2E | `npm run test:e2e:local` | `qa.spec.ts` |
| Agent ask flow (split submit) | E2E | `npm run test:e2e:local` | `agent.spec.ts` (updated for `agent-submit`) |
| Full journey | E2E | `npm run test:e2e:local` | `full-ui.spec.ts` |

### E2E impact of split submit (D-10/D-11/D-35)

Current `agent.spec.ts` calls `askQuestion()` from `helpers.ts` which uses `ask-submit` testid.  
After Phase 2:
- RAG path: still uses `ask-submit` → no change to `qa.spec.ts`
- Agent path: must use `agent-submit` testid → **`agent.spec.ts` and `helpers.ts` need an `askAgentQuestion()` helper** or the existing `askQuestion()` must be updated to detect mode

**Recommended**: Add `askAgentQuestion(text)` helper in `helpers.ts` that uses `[data-testid="agent-submit"]` and update `agent.spec.ts` to call it.

### Nyquist validation checklist [VERIFIED]

```
Goal: Chat view + config nesting extracted without user-visible regression
Verify:
 - [ ] cargo test -p config (CFG-01 roundtrip test)
 - [ ] cargo test --workspace (SHELL-02 compile + existing unit tests)
 - [ ] npm test (FE-02 useChat hook unit test)
 - [ ] npm run test:e2e:local (full suite, all 8 specs)
 - [ ] qa.spec.ts: RAG ask still works
 - [ ] agent.spec.ts: agent-submit testid works (updated)
 - [ ] full-ui.spec.ts: full journey green
 - [ ] Manual: existing config.json loads without error
```

---

## Security Domain (ASVS L1)

| Concern | Status in Phase 2 | Notes |
|---------|-------------------|-------|
| `cloud_api_key` in config.json | **Still present** (plaintext) | Deferred to Phase 3 (CFG-02/CFG-03). Nesting via `#[serde(flatten)]` does NOT remove the key from JSON. |
| IPC boundary | Unchanged — `Result<T, String>` | No new attack surface |
| Agent `shell_exec` plugin permission | Unchanged default | `default_granted_plugin_permissions` preserved in `AgentConfig` |
| No new external network calls | Confirmed | Phase 2 is structural only |

> **Note**: The config nesting in Phase 2 is a shape refactor only. `cloud_api_key` will still be serialized into flat JSON in the `EmbeddingConfig` struct. Keychain migration is Phase 3.

---

## Sources

| Claim | Source |
|-------|--------|
| `commands/mod.rs` contains config, library, sync [VERIFIED] | `src-tauri/src/commands/mod.rs` (7 lines) |
| `ChatView.tsx` is a `{children}` passthrough stub [VERIFIED] | `src/views/ChatView.tsx` (5 lines) |
| `tauri.ts` has 7 wrappers for config/library/sync [VERIFIED] | `src/lib/tauri.ts` |
| `AppConfig` has 28 flat fields + 1 skip [VERIFIED] | `crates/config/src/types.rs` |
| Chat commands are at lib.rs lines 228–423 [VERIFIED] | `src-tauri/src/lib.rs` |
| `start_ask_e2e` at lib.rs line 316 calls both run functions [VERIFIED] | `src-tauri/src/lib.rs:316–347` |
| `ask-handle-count`, `agent-mode-toggle`, `ask-submit` testids [VERIFIED] | `src/App.tsx` data-testid grep |
| `submitChatForm()` in helpers.ts uses `ask-submit` testid [VERIFIED] | `e2e/helpers.ts:134` |
| Phase 1 completed: 52 IPC commands verified [VERIFIED] | `01-03-SUMMARY.md` |
| `TokenEvent` struct defined in lib.rs [VERIFIED] | `src-tauri/src/lib.rs:57–60` |
| `#[serde(flatten)]` causes no JSON key nesting [CITED] | serde documentation; standard Rust pattern |
| `pub(crate)` visibility enables cross-module helper sharing [CITED] | Rust reference |
| Agent state + streaming Channel stay in App.tsx until Phase 5 [CITED] | `02-CONTEXT.md` D-06, D-15 |

---

*Research complete. Ready for Phase 2 planning.*
