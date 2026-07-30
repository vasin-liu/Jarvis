# Phase 21 Plan Check

**Checked:** 2026-07-29  
**Checker:** gsd-plan-checker  
**Artifacts reviewed:** `21-CONTEXT.md`, `21-RESEARCH.md`, `21-01-PLAN.md`, `21-02-PLAN.md`, `21-VALIDATION.md`, `REQUIREMENTS.md` (BOOT-02), `ROADMAP.md` Phase 21  
**Codebase baseline:** `src-tauri/src/state.rs`, `src-tauri/src/commands/config.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`, `crates/embedder/src/lib.rs`

---

## status: passed

Plans are executable, aligned with locked decisions D-01..D-05, and satisfy Nyquist RED→GREEN wave structure. One intra-plan compile-order note (F-01) should be applied during execution; not a blocker for `/gsd-execute-phase 21`.

---

## Checklist Results

| # | Criterion | Result | Notes |
|---|-----------|--------|-------|
| 1 | D-01..D-05 cited/covered | ✅ Pass | See decision matrix below |
| 2 | BOOT-02 with TDD RED→GREEN | ✅ Pass | Wave 1 RED (21-01); Wave 2 GREEN (21-02); IPC + helper mapped |
| 3 | No UI scope creep (D-05) | ✅ Pass | `files_modified` limited to tauri shell; no `src/`, e2e, Vitest |
| 4 | Concrete code (no TBD) | ✅ Pass | Full snippets for helper, init_state, IPC, handler registration |
| 5 | `must_haves` / verify commands | ✅ Pass | YAML frontmatter + per-task `<automated>` on all 8 tasks |
| 6 | Wave dependency 21-02 after 21-01 | ✅ Pass | `wave: 1/2`, `depends_on: [21-01]` / `[20-02]` |
| 7 | Nyquist automated verifies | ✅ Pass | `21-VALIDATION.md` maps every task; `nyquist_compliant: true` |
| 8 | Verify commands valid | ✅ Pass | Package `tauri-app` confirmed in `src-tauri/Cargo.toml`; filter `readiness_` matches `mod readiness_tests` paths |
| 9 | ROADMAP Phase 21 success criteria | ✅ Pass | All four SC mapped to tasks + validation rows |

---

## Decision Coverage Matrix

| Decision | 21-01 | 21-02 | 21-VALIDATION |
|----------|-------|-------|---------------|
| **D-01** AppState deferred watch | `deferred_pending` cites watch semantics | Task 2 init_state `Some(deferred.clone())` before cast | 21-01-02, 21-02-02, 21-02-05 |
| **D-02** Pure `readiness_from_watch` | Three RED tests (None→Ready, Pending, Failed) | Task 1 helper + `embedder_readiness`; non-blocking path | 21-01-01/02/03, 21-02-01 |
| **D-03** IPC `get_embedder_readiness` | `deferred_failed` prep for message mapping | Tasks 3–4 view + command + handler | 21-01-03, 21-02-03/04 |
| **D-04** reload_providers debt | Wave 1 fence (no init changes) | Task 1 field comment; Task 2 leaves `reload_providers` untouched | 21-02-02 + manual review table |
| **D-05** Scope fence | Single file; no IPC/AppState prod code | Repeated fences; artifact table lists exclusions | Wave 1/2 requirement lists |

---

## Requirement Coverage

### BOOT-02 — Embedder readiness over IPC without extra SQLite

| Behavior | RED (21-01) | GREEN (21-02) | Verify |
|----------|-------------|---------------|--------|
| None watch → Ready (Mock/E2E/non-deferred) | `no_deferred_is_ready` | `deferred_embedder: None` in else branch | `cargo test -p tauri-app no_deferred_is_ready` |
| Deferred Pending / Failed observable | `deferred_pending`, `deferred_failed` | `readiness_from_watch` delegates to `ready_state()` | `cargo test -p tauri-app readiness_` |
| IPC exposes `{ state, message }` camelCase | — | `EmbedderReadinessView` + `get_embedder_readiness` | `cargo check -p tauri-app` |
| No SQLite outside `store` | Wave 1 fence | Tasks 2–3 acceptance criteria | Code review; no new `Store::open` |
| Handler registered for frontend poll | — | Task 4 `mod.rs` + `lib.rs` | `cargo check -p tauri-app` |

Runtime invoke JSON deferred to Phase 22 consumer (documented manual-only in `21-VALIDATION.md`) — appropriate for shell-only phase per D-05.

### ROADMAP Phase 21 Success Criteria

| SC | Covered by |
|----|------------|
| 1. FastEmbed cold-start keeps `Option<Arc<DeferredEmbedder>>` on AppState | 21-02 Task 2 + code review |
| 2. `get_embedder_readiness` returns `{ state, message }` pending/ready/failed | 21-02 Task 3 `readiness_view` |
| 3. Non-deferred providers report Ready without watch | D-02 + `no_deferred_is_ready` + else branch `None` |
| 4. store-only SQLite; shell compiles + unit helpers green | 21-02 Task 5; `cargo test -p tauri-app readiness_ && cargo check -p tauri-app` |

---

## TDD Wave Structure

```
Wave 1 (21-01-PLAN.md)              Wave 2 (21-02-PLAN.md)
─────────────────────────           ─────────────────────────
Task 1: RED tests (3)         ───►   Task 1: readiness_from_watch + AppState methods
Task 2: confirm RED compile          Task 2: init_state deferred_embedder wiring
                                     Task 3: EmbedderReadinessView + IPC command
                                     Task 4: mod.rs + lib.rs registration
                                     Task 5: regression gate
depends_on: [20-02]                  depends_on: [21-01]
```

Wave 0 dependency: Phase 20 `EmbedderReadyState` re-export at `crates/embedder/src/lib.rs:8` — sampled via `cargo test -p embedder` per `21-VALIDATION.md`.

Dependency direction correct. No circular edges.

---

## Codebase Alignment (concrete references)

| Plan reference | Baseline verified |
|----------------|-------------------|
| FastEmbed branch `state.rs:68–90` | Present; plan tuple refactor matches current `deferred` + spawn pattern |
| `AppState` struct `state.rs:115–129` | No `deferred_embedder` yet — plan adds after `scheduler` |
| `reload_providers` `state.rs:151–155` | Sync `build_embedder`; D-04 leave-unchanged is correct |
| `get_index_status` pattern `config.rs:35–43` | camelCase struct + thin command — IPC mirrors this |
| `commands/mod.rs` config re-export | Extend line 15–17 with `get_embedder_readiness` |
| `lib.rs` `generate_handler!` | `get_index_status` at line 70 — adjacent registration valid |
| Phase 20 imports | `embedder::EmbedderReadyState` exported from crate root |

Line numbers may drift slightly; file paths and patterns are accurate.

---

## Findings

| ID | Severity | Finding | Action |
|----|----------|---------|--------|
| F-01 | warning | **21-02 Task 1** adds `AppState.deferred_embedder` field but forbids `init_state` update until Task 2; Rust requires all struct fields in `Ok(AppState { ... })` — Task 1 `<automated>` verify cannot succeed until constructor is updated | During execution: add `deferred_embedder: None` to `Ok(AppState { ... })` at end of Task 1 (compile stub), then Task 2 replaces with conditional wiring; **or** move struct field + constructor to Task 2 and limit Task 1 to `readiness_from_watch` only |
| F-02 | info | `.planning/ROADMAP.md` Phase 21 still lists **Plans: TBD** while 21-01/21-02 exist | Update ROADMAP at phase kickoff — not a plan blocker |
| F-03 | info | IPC `readiness_view` string mapping (`pending`/`ready`/`failed`) has compile-only verify, no dedicated unit test | Acceptable — trivial match on Phase 20 enum; optional `#[test]` on `readiness_view` if desired post-GREEN |
| F-04 | info | `Ready` via `Some(deferred)` after `fulfill()` not covered in shell tests | Phase 20 covers `ready_state()` transition; helper delegates — acceptable gap |
| F-05 | info | Task 21-02-05 `<automated>` runs tests only; acceptance criteria also require `cargo check` | Task 4 combined verify covers both; Task 5 manual run should include check per validation table 21-02-05 |

No **error**-severity findings.

---

## Nyquist Compliance

- 8/8 tasks have `<automated>` verify commands
- No 3 consecutive tasks without automated verify
- Wave 1 RED (compile fail on missing symbol) before Wave 2 GREEN enforced via `depends_on: [21-01]`
- Estimated feedback latency < 30s (`21-VALIDATION.md`)
- E2E gap justified: IPC not user-facing until Phase 22 (D-05); BOOT-03 covers UI + Vitest

---

## Verdict

**PASSED** — Plans are ready for `/gsd-execute-phase 21`. Apply F-01 during Task 1/2 (one-line `init_state` stub or reorder field addition) so Task 1 verify can run green; otherwise all locked decisions, BOOT-02, ROADMAP criteria, and Nyquist gates are covered with concrete, executable steps.
