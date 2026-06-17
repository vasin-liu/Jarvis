# Phase 1: Scaffold + Shell Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-17
**Phase:** 1-Scaffold + Shell Foundation
**Areas discussed:** Command groups, Frontend scaffold, IPC typing, AppState helpers, Test gate, Optional 3rd module, index_ops boundaries

---

## Command Groups

| Option | Description | Selected |
|--------|-------------|----------|
| config + library | get/set_config, get_index_status + list_sources/remove_source/source_count | ✓ |
| tasks + memory | Isolated CRUD domains | |
| config + tasks | Config simple; tasks small | |
| You decide | Safest pair for zero-regression | |

| Option | Description | Selected |
|--------|-------------|----------|
| Exactly 2 modules | Roadmap minimum | |
| 2–3 if third is tiny | e.g., sync.rs with 2 commands | ✓ |
| You decide | As many as safe | |

| Option | Description | Selected |
|--------|-------------|----------|
| commands/mod.rs re-exports | lib.rs keeps generate_handler! | ✓ |
| register_commands() helper | Single registration function | |
| You decide | Match repo patterns | |

| Option | Description | Selected |
|--------|-------------|----------|
| Move helpers with module | Private helpers follow their commands | ✓ |
| Leave all helpers in lib.rs | Until module extracted | |
| You decide | Minimize lib.rs without premature moves | |

**User's choice:** config + library (+ optional tiny 3rd); mod re-exports; helpers move with extracted modules.

---

## Frontend Scaffold

| Option | Description | Selected |
|--------|-------------|----------|
| Folders only | Empty dirs; App.tsx unchanged | |
| Stub views re-exporting App.tsx | Research S0 pattern | ✓ |
| Extract layout components | Sidebar + banners now | |
| You decide | Max structure, min risk | |

| Option | Description | Selected |
|--------|-------------|----------|
| Keep router inline | No router change | |
| Extract View type to types/view.ts | Routing logic stays in App.tsx | ✓ |
| You decide | | |

| Option | Description | Selected |
|--------|-------------|----------|
| No component extraction | Defer Sidebar/banners | ✓ |
| Layout chrome only | Partial extract | |
| Layout + stub views | Combined | |

| Option | Description | Selected |
|--------|-------------|----------|
| Zero testid changes | All stay in App.tsx | ✓ |
| Move testids with JSX | Only if elements move | |

**User's choice:** Stub views; View type extracted; no components; testids unchanged.

---

## IPC Typing Layer

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 1 partial tauri.ts | Wrappers for extracted commands only | ✓ |
| Phase 1 full tauri.ts | All 52 commands upfront | |
| Defer to Phase 2 | No tauri.ts in Phase 1 | |

| Option | Description | Selected |
|--------|-------------|----------|
| Extract types from App.tsx | Move interfaces to types/ipc.ts | ✓ |
| Empty types file | Populate later | |

| Option | Description | Selected |
|--------|-------------|----------|
| App.tsx keeps direct invoke | tauri.ts for new modules only | ✓ |
| Migrate config calls in Phase 1 | Partial App.tsx cleanup | |

| Option | Description | Selected |
|--------|-------------|----------|
| String passthrough errors | No new error taxonomy | ✓ |
| Typed JarvisError enum | New frontend errors | |

**User's choice:** Partial tauri.ts; extract types; App.tsx still invokes directly; string errors.

---

## AppState Helpers

| Option | Description | Selected |
|--------|-------------|----------|
| Struct + impl only | reload_providers, watcher lifecycle | ✓ |
| Struct + init_state | setup helpers in state.rs | |

| Helper | Decision |
|--------|----------|
| lark_opts | Stay in lib.rs until lark.rs (Phase 3) |
| seed_skills/hooks/plugins | Stay in lib.rs (startup in setup) |
| build_agent_context / execute_agent_question | Stay in lib.rs until Phase 5 |

**User's choice:** Minimal state.rs — struct + impl only; all glue stays in lib.rs for now.

---

## Test Gate

| Option | Description | Selected |
|--------|-------------|----------|
| smoke + navigation only | Roadmap minimum | ✓ |
| Full E2E suite | All specs every PR | |
| smoke + navigation + qa | Add Q&A spec | |

| Option | Description | Selected |
|--------|-------------|----------|
| cargo test --workspace | Full Rust suite (QA-03) | ✓ |
| npm test (Vitest) | Frontend unit tests (QA-03) | ✓ |
| Zero behavior change bar | No renames/features/testid moves | ✓ |

**User's choice:** Minimal E2E (smoke+nav); full Rust+Vitest; zero behavior change.

---

## Optional 3rd Module

| Option | Description | Selected |
|--------|-------------|----------|
| sync.rs | 2 commands, tiny | ✓ |
| tasks.rs | 3 commands, isolated CRUD | |
| Skip 3rd | config + library only | |

**Criterion:** Smallest module wins.

---

## index_ops Boundaries

| Option | Description | Selected |
|--------|-------------|----------|
| Leave untouched | No Phase 1 changes | ✓ |
| Refactor at commands/index.rs | Phase 3 / S2f | ✓ |

**User's choice:** index_ops.rs unchanged in Phase 1.

---

## Skipped Areas (defaults applied in CONTEXT.md)

- **Stub structure:** One file per view; direct imports (no barrel) — research default
- **PR slicing:** Single PR preferred — research default

---

## Claude's Discretion

- Exact `commands/` file naming
- Whether sync.rs included as 3rd module
- Stub wrapper implementation detail

## Deferred Ideas

None captured during discussion.
