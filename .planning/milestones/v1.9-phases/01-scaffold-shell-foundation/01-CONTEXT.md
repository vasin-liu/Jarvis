# Phase 1: Scaffold + Shell Foundation - Context

**Gathered:** 2026-06-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Establish the v1.9.x refactor file structure with **zero user-visible behavior change**. Phase 1 creates extraction points on both layers without moving business logic out of monoliths yet.

**In scope:**
- `src/views/` stub components (thin wrappers; App.tsx retains logic)
- `src/types/ipc.ts` + `src/types/view.ts` (types extracted from App.tsx)
- `src/lib/tauri.ts` partial wrappers for Phase 1 command groups only
- `src-tauri/src/state.rs` — `AppState` struct + impl methods
- `src-tauri/src/commands/mod.rs` + at least 2 command modules (`config`, `library`; optional 3rd `sync`)
- E2E gate: `smoke.spec.ts` + `navigation.spec.ts` green
- Full Rust + Vitest gates per QA-03

**Out of scope (later phases):**
- Extracting view JSX or hooks from App.tsx (Phase 2+)
- Chat/agent/lark/memory command modules beyond optional tiny `sync`
- `index_ops.rs` refactoring
- Config nesting, keychain, agent protocol, memory URIs
- New user-facing features or `data-testid` changes

</domain>

<decisions>
## Implementation Decisions

### Tauri Command Split
- **D-01:** Extract `commands/config.rs` + `commands/library.rs` as the first two modules (low coupling, read-heavy).
- **D-02:** Optional 3rd module: `commands/sync.rs` (`get_sync_status`, `run_scheduled_sync_cmd`) — smallest module wins; skip if it adds risk.
- **D-03:** `commands/mod.rs` re-exports command fns; `lib.rs` keeps `generate_handler![...]` listing names from `commands::`.
- **D-04:** Private helpers move with their command module when extracted; helpers for not-yet-extracted modules stay in `lib.rs`.

### AppState (`state.rs`)
- **D-05:** Move `AppState` struct definition + impl methods (e.g., `reload_providers`, watcher/scheduler lifecycle) to `state.rs`.
- **D-06:** `init_state()` / `setup()` orchestration stays in `lib.rs` for Phase 1.
- **D-07:** `lark_opts`, `seed_skills_dir` / `seed_hooks_dir` / `seed_plugins_dir`, and agent glue (`build_agent_context`, `execute_agent_question`, `agent_response_to_ask`) **stay in `lib.rs`** until their target phases (Lark Phase 3, agent Phase 5).

### Frontend Scaffold
- **D-08:** Create stub views: one file per view under `src/views/` (`ChatView.tsx`, `LibraryView.tsx`, etc.) re-exporting fragments still defined in `App.tsx`.
- **D-09:** Extract `View` type to `src/types/view.ts`; routing logic stays in `App.tsx`.
- **D-10:** No shared component extraction (Sidebar, banners) in Phase 1 — defer to later phases.
- **D-11:** All existing `data-testid` values unchanged; selectors stay on same DOM elements in App.tsx.

### IPC Typing Layer
- **D-12:** Create `src/types/ipc.ts` by moving TS interfaces from App.tsx (`ChatSession`, `Source`, `TaskItem`, etc.).
- **D-13:** Create `src/lib/tauri.ts` with wrappers **only** for Phase 1 extracted commands (config, library, sync).
- **D-14:** App.tsx continues direct `invoke()` calls in Phase 1 — migrate to `tauri.ts` starting Phase 2 Chat extraction.
- **D-15:** Error handling: passthrough `Result<T, String>` — no new frontend error taxonomy in Phase 1.

### index_ops.rs
- **D-16:** Leave `index_ops.rs` completely untouched in Phase 1; refactor when `commands/index.rs` is extracted (roadmap Phase 3 / research S2f).

### Quality Gates
- **D-17:** E2E merge gate: `smoke.spec.ts` + `navigation.spec.ts` only (roadmap minimum).
- **D-18:** `cargo test --workspace` + `npm test` required on Phase 1 merge (QA-03).
- **D-19:** Regression bar = zero behavior change — no IPC renames, no testid moves, no new features; line-count reduction optional.

### Stub Structure & PR Slicing (defaults — user skipped detailed discussion)
- **D-20:** One stub file per view; App.tsx imports each directly (no barrel `views/index.ts` unless planner finds benefit).
- **D-21:** Single Phase 1 PR preferred (shell + FE scaffold together); if split, shell lands first and each PR passes full gate.

### Claude's Discretion
- Exact file naming inside `commands/` (e.g., `library.rs` vs `sources.rs`) as long as grouping matches research map.
- Whether optional 3rd `sync.rs` is included if diff stays minimal.
- Stub implementation detail (named re-export vs thin wrapper component) as long as App.tsx owns all logic and E2E passes.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope & requirements
- `.planning/ROADMAP.md` — Phase 1 goal, success criteria, key files
- `.planning/REQUIREMENTS.md` — FE-01, SHELL-01, QA-01, QA-03 traceability
- `.planning/PROJECT.md` — Core value, constraints, refactor framing

### Target architecture
- `.planning/research/ARCHITECTURE.md` — S0 scaffold spec, command grouping map, target directory layout
- `.planning/research/SUMMARY.md` — Phase ordering rationale
- `.planning/research/PITFALLS.md` — Per-phase risk checklist

### Codebase context
- `.planning/codebase/ARCHITECTURE.md` — Layer model, invariants, data flows
- `.planning/codebase/STRUCTURE.md` — Directory layout, naming conventions, where to add code
- `.planning/codebase/CONCERNS.md` — Monolith debt inventory (App.tsx, lib.rs)
- `.planning/codebase/TESTING.md` — Test pyramid, E2E spec map, CI gates

### Engineering policy
- `AGENTS.md` — Stack, milestones, file boundaries, TDD/E2E requirements
- `.cursor/rules/e2e-required.mdc` — E2E mandatory for user-facing changes
- `.cursor/rules/tdd-goal-driven.mdc` — Red-green-refactor workflow

### Implementation targets (current monoliths)
- `src/App.tsx` — Frontend monolith to scaffold around (~2,800 lines)
- `src-tauri/src/lib.rs` — Tauri shell monolith (~52 commands)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/lib/citations.ts`, `src/lib/sourceDisplay.ts` — already extracted; pattern for future `lib/` modules
- `src-tauri/src/index_ops.rs`, `insights_ops.rs`, `sync_scheduler.rs`, `e2e.rs` — already split orchestration modules; command split follows same pattern
- `e2e/helpers.ts` — DOM helpers; Phase 1 must not break navigation testids

### Established Patterns
- Tauri commands return `Result<T, String>` — preserve in Phase 1
- `snake_case` IPC names — no renames (frontend `invoke("list_sources")` etc.)
- Thin shell, fat crates — commands delegate to crates/index_ops; Phase 1 only moves command fns, not domain logic
- E2E deterministic mocks via `JARVIS_E2E=1` + `src-tauri/src/e2e.rs`

### Integration Points
- `lib.rs::run()` — `setup()` initializes state, registers commands via `generate_handler!`
- `App.tsx` — view switch on `View` type; nav uses `data-testid="nav-*"` selectors
- Command groups map (research): config (3), library (3), sync (2) = 8 commands for Phase 1 extraction

</code_context>

<specifics>
## Specific Ideas

- Follow research S0 ordering: structure first, behavior unchanged, line counts may stay flat
- User explicitly chose config+library as first command pair (lowest coupling)
- Smallest-module criterion for optional 3rd extraction → sync.rs

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope. Items explicitly deferred to later phases:
- Component extraction (Sidebar, IndexProgressBanner) → Phase 2+ / Phase 6
- Full `lib/tauri.ts` migration → Phase 2 Chat extraction
- `index_ops.rs` + `commands/index.rs` → Phase 3
- Agent/lark/memory command modules → Phases 3–5
- Full E2E suite as Phase 1 gate → later phases (Phase 1 uses smoke+navigation minimum)

</deferred>

---

*Phase: 1-Scaffold + Shell Foundation*
*Context gathered: 2026-06-17*
