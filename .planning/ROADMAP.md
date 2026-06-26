# Roadmap: Jarvis v1.9.x Structural Refactor

**Created:** 2026-06-17  
**Project:** Jarvis  
**Version target:** v1.9.0 → v1.9.6  
**Mode:** Vertical MVP slices (incremental, E2E-gated)

---

## Overview

| # | Phase | Goal | Requirements | Version |
|---|-------|------|--------------|---------|
| 1 | Scaffold + Shell Foundation | 3/3 | Complete   | 2026-06-17 |
| 2 | Chat Extraction + Config Types | 3/3 | Complete    | 2026-06-26 |
| 3 | Library/Tasks + Keychain | Extract index views; secrets to OS keychain | FE-03, SHELL-03, CFG-02, CFG-03 | v1.9.2 |
| 4 | Memory Model + View | Stable memory URIs; extract Memory view | FE-04, SHELL-04, MEM-01, MEM-02 | v1.9.3 |
| 5 | Agent Protocol | Structured tool calls; agent command module | AGT-01, AGT-02, AGT-03, SHELL-05 | v1.9.4 |
| 6 | Settings + Arch Review | Complete extraction; sync UX; sign-off | FE-05, QA-02, QA-04 | v1.9.5–1.9.6 |

**Total:** 6 phases | 22 requirements | 100% coverage ✓

---

### Phase 1: Scaffold + Shell Foundation

**Goal:** Create target directory structure and begin Tauri command module split with zero behavior change.

**Mode:** mvp

**Requirements:** FE-01, SHELL-01, QA-01, QA-03

**Success Criteria:**

1. `src/views/` directory exists with placeholder or first extracted layout component
2. `src-tauri/src/state.rs` holds `AppState`; `commands/mod.rs` registered
3. At least 2 command groups moved to `commands/*.rs` with identical IPC signatures
4. `npm run test:e2e:local` passes (smoke + navigation specs)
5. `cargo test --workspace` passes

**UI hint:** yes

**Key files:** `src/App.tsx`, `src-tauri/src/lib.rs`, `src-tauri/src/state.rs`, `src-tauri/src/commands/`

---

### Phase 2: Chat Extraction + Config Types

**Goal:** Extract Chat view and hook; introduce nested `AppConfig` structs with backward-compatible serde.

**Mode:** mvp

**Requirements:** FE-02, FE-06, SHELL-02, CFG-01

**Success Criteria:**

1. `ChatView.tsx` + `useChat.ts` own all chat/agent UI logic
2. `commands/chat.rs` contains all chat/session commands
3. `AppConfig` nested structs load/save existing flat `config.json`
4. `data-testid` values for chat (`chat-input`, `agent-mode-toggle`, etc.) unchanged
5. `qa.spec.ts`, `agent.spec.ts`, `full-ui.spec.ts` pass

**UI hint:** yes

**Key files:** `src/views/ChatView.tsx`, `src/hooks/useChat.ts`, `src/lib/tauri.ts`, `crates/config/src/types.rs`

---

### Phase 3: Library/Tasks + Keychain

**Goal:** Extract Library and Tasks views; migrate API secrets to OS keychain.

**Mode:** mvp

**Requirements:** FE-03, SHELL-03, CFG-02, CFG-03

**Success Criteria:**

1. `LibraryView.tsx` and `TasksView.tsx` extracted with hooks
2. `commands/index.rs` and `commands/lark.rs` complete
3. `cloud_api_key` read/written via `keyring`; not persisted in plaintext JSON
4. Legacy config with plaintext key migrates automatically on first load
5. `lark.spec.ts` and `full-ui.spec.ts` pass

**UI hint:** yes

**Key files:** `src/views/LibraryView.tsx`, `src/views/TasksView.tsx`, `crates/config/src/secrets.rs`, `src-tauri/src/commands/index.rs`

---

### Phase 4: Memory Model + View

**Goal:** Implement `memory://` URI scheme and extract Memory view.

**Mode:** mvp

**Requirements:** FE-04, SHELL-04, MEM-01, MEM-02

**Success Criteria:**

1. New memories created with `memory://{uuid}` URI
2. `resolve_memory_id` uses strict ID/URI only (title fallback deprecated with log)
3. Migration sets URI on existing memory sources
4. `MemoryView.tsx` + `useMemory.ts` extracted
5. `memory.spec.ts` passes

**UI hint:** yes

**Key files:** `crates/memory/`, `src/views/MemoryView.tsx`, `src-tauri/src/commands/memory.rs`

---

### Phase 5: Agent Protocol

**Goal:** Replace fragile XML tool parsing with structured JSON; complete agent command module.

**Mode:** mvp

**Requirements:** AGT-01, AGT-02, AGT-03, SHELL-05

**Success Criteria:**

1. `ToolCallParser` trait with JSON implementation for Mock + OpenAI-compatible
2. Parse errors returned in response metadata and shown in Chat UI
3. XML parser remains as fallback (logged when used)
4. `commands/agent.rs` and `commands/config.rs` complete
5. `agent.spec.ts` passes with structured tool format

**UI hint:** yes

**Key files:** `crates/agent/src/tools.rs`, `crates/llm/`, `src/views/ChatView.tsx`

---

### Phase 6: Settings + Architecture Review

**Goal:** Extract Settings view; surface sync errors; complete refactor sign-off.

**Mode:** mvp

**Requirements:** FE-05, QA-02, QA-04

**Success Criteria:**

1. `SettingsView.tsx` + `useJarvisConfig.ts` extracted; `App.tsx` under 300 lines
2. `lib.rs` under 200 lines (registration + startup only)
3. Scheduled sync errors persisted in `store.meta` and visible in Settings
4. Architecture review checklist passes (documented in phase VERIFICATION.md)
5. Full E2E suite green; no scope creep beyond refactor

**UI hint:** yes

**Key files:** `src/views/SettingsView.tsx`, `src/App.tsx`, `src-tauri/src/lib.rs`, `src-tauri/src/sync_scheduler.rs`

---

## Phase Dependencies

```
Phase 1 ──► Phase 2 ──► Phase 3 ──► Phase 4 ──► Phase 5 ──► Phase 6
(scaffold)   (chat)      (library)    (memory)    (agent)     (settings)
```

Each phase depends on the prior phase's E2E green gate.

---

## Research References

- `.planning/research/SUMMARY.md` — phase ordering rationale
- `.planning/research/ARCHITECTURE.md` — target directory layout (S0–S8 slices)
- `.planning/research/PITFALLS.md` — per-phase risk checklist
- `.planning/codebase/CONCERNS.md` — debt inventory

---
*Roadmap created: 2026-06-17*
