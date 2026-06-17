# Project Research Summary

**Project:** Jarvis v1.9.x Structural Refactor  
**Domain:** Brownfield Tauri 2 desktop app — incremental architecture decomposition  
**Researched:** 2026-06-17  
**Confidence:** HIGH

## Executive Summary

Jarvis v1.8.0 is feature-complete with sound crate-level architecture but two critical monoliths (`src/App.tsx` ~2,800 lines, `src-tauri/src/lib.rs` ~1,300 lines) and three fragile cross-cutting areas (plaintext secrets, XML agent tool parsing, fuzzy memory identity). The v1.9.x refactor preserves all user-facing behavior while decomposing structure in **vertical MVP slices** — each phase touches frontend + shell + (when needed) domain code, with E2E green after every merge.

The recommended approach is **balanced phasing**: scaffold frontend views and Tauri command modules early, interleave keychain hardening with Settings extraction, then memory URI scheme and structured agent protocol. Big-bang extraction is the primary risk; `data-testid` contracts and IPC signature stability are the guardrails.

## Key Findings

### Recommended Stack

Preserve Tauri 2 + Rust workspace + React 19. Add only `keyring` for OS credential storage. No new frontend state library or router — extract views/hooks within existing patterns.

**Core technologies:**
- **keyring** — API secrets out of `config.json` — HIGH confidence
- **React view/hook split** — `src/views/*`, `src/hooks/*`, `src/lib/tauri.ts` — HIGH confidence
- **Tauri `commands/*` modules** — thin shell decomposition — HIGH confidence
- **Structured LLM tool calls** — extend `ChatModel` + `ToolCallParser` — MEDIUM confidence

### Expected Features

**Must have (table stakes):**
- All v1.8 user journeys unchanged (Chat, Library, Tasks, Memory, Settings)
- Hybrid RAG with citations, indexing pipeline, Lark/Cursor sync
- E2E green (`JARVIS_E2E=1`) after every phase
- Config/DB migration without data loss

**Should have (protect differentiators):**
- Agent orchestration (pipeline/router) during refactor
- Streaming chat + tool-call UI
- Per-agent provider overrides

**Defer (post-v1.9):**
- v2.0 version bump
- Dedicated memories table (optional if `memory://` URIs suffice)
- Full plugin sandbox

### Architecture Approach

Decompose presentation (`src/views` + hooks) and IPC shell (`commands/*` + `state.rs`) while preserving domain crate boundaries and single-DB-owner rule. Vertical slices S0–S8 from architecture research map to 6 GSD phases.

**Major components:**
1. **Frontend shell** — `App.tsx` becomes layout + nav + view router only
2. **Tauri command modules** — mirror crate boundaries (`chat`, `index`, `lark`, `memory`, `agent`, `config`)
3. **Config/secrets layer** — nested `AppConfig` + keyring migration
4. **Agent + memory hardening** — structured tools + `memory://` URIs

### Critical Pitfalls

1. **Big-bang extraction** — one vertical slice per merge; E2E before merge
2. **Breaking `data-testid`** — treat as public API; update specs with JSX moves
3. **Incorrect state lifting** — one writer per persisted state; hooks not duplicate `useState`
4. **IPC signature drift** — command names/args unchanged unless explicit migration
5. **Plaintext key migration failure** — read legacy key once, write keyring, strip JSON atomically

## Implications for Roadmap

### Phase 1: Scaffold + Shell Foundation
**Rationale:** Establish target directories and first command modules without behavior change  
**Delivers:** `src/views/` scaffold, `commands/mod.rs` + 2–3 command modules, layout extraction  
**Addresses:** FE-01, SHELL-01, QA-01  
**Avoids:** Big-bang extraction

### Phase 2: Chat View + Config Types
**Rationale:** Highest-traffic UI path; start nested config types (no keychain yet)  
**Delivers:** `ChatView`, `useChat`, `commands/chat.rs` complete  
**Addresses:** FE-02, SHELL-02, CFG-01  
**Avoids:** State lifting bugs

### Phase 3: Library/Tasks + Keychain
**Rationale:** User-visible settings path; secrets migration lands with Settings touch  
**Delivers:** Library/Tasks views, keyring integration, `commands/index.rs` + `lark.rs`  
**Addresses:** FE-03, CFG-02, SHELL-03  

### Phase 4: Memory Model
**Rationale:** URI scheme before agent protocol to stabilize identity  
**Delivers:** `memory://` URIs, Memory view extraction, migration for existing rows  
**Addresses:** MEM-01, MEM-02, FE-04  

### Phase 5: Agent Protocol
**Rationale:** Isolated domain change; Mock/E2E updated first  
**Delivers:** `ToolCallParser`, structured JSON path, parse errors in UI  
**Addresses:** AGT-01, AGT-02  

### Phase 6: Settings + Sync UX + Arch Review
**Rationale:** Complete extraction; surface sync errors; final review gate  
**Delivers:** `SettingsView`, sync error meta, architecture sign-off  
**Addresses:** FE-05, CFG-03, QA-02  

### Phase Ordering Rationale

- Scaffold before feature extraction reduces merge conflict surface
- Keychain ships when Settings is touched (user expectation for secret storage UI)
- Memory before agent avoids tool identity bugs during protocol change
- Settings last because it depends on nested config + keychain

### Research Flags

- **Phase 5:** Provider-specific structured output quirks — plan per-provider tests
- **Phase 3:** Windows keyring behavior — integration test on CI runner if possible

Phases with standard patterns (lower research need):
- **Phase 1–2:** Well-documented React/Tauri module patterns

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Brownfield; one new dep (`keyring`) |
| Features | HIGH | Codebase map + CONCERNS.md comprehensive |
| Architecture | HIGH | Target layout defined in ARCHITECTURE.md research |
| Pitfalls | HIGH | E2E harness provides strong safety net |

**Overall confidence:** HIGH

### Gaps to Address

- **keyring CI testing:** May need mocked secret backend in E2E — handle in Phase 3 planning
- **Provider JSON mode variance:** Test OpenAI + Ollama paths separately in Phase 5

## Sources

### Primary (HIGH confidence)
- `.planning/codebase/*` — full codebase map (2026-06-17)
- `.planning/PROJECT.md` — user decisions from questioning
- `AGENTS.md` — milestone history and constraints

### Secondary (MEDIUM confidence)
- `keyring` crate docs — cross-platform credential storage pattern

---
*Research completed: 2026-06-17*  
*Ready for roadmap: yes*
