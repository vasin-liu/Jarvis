# Phase 21: Readiness IPC — Context

**Gathered:** 2026-07-29  
**Status:** Ready for planning  
**Source:** Design + implementation plan express path (`/gsd-plan-phase 21 --chain` continuation)

## Phase Goal

The Tauri shell exposes embedder readiness to the frontend without coupling UI to FastEmbed internals (BOOT-02).

## Constraints

- Depends on Phase 20 (`EmbedderReadyState`, `ready_state`)
- Keep FastEmbed deferred cold-start path; add `Option<Arc<DeferredEmbedder>>` watch on `AppState`
- Only `store` opens SQLite
- No Settings UI this phase (Phase 22)
- No version bump / release smoke (Phase 23)
- Spec: `docs/superpowers/specs/2026-07-29-release-hardening-design.md`
- Plan draft: `docs/superpowers/plans/2026-07-29-release-hardening.md` Task 2

## Decisions

### D-01: AppState holds deferred watch
**Choice:** `deferred_embedder: Option<Arc<DeferredEmbedder>>` — `Some(same Arc)` on FastEmbed cold-start path; `None` otherwise  
**Why:** Trait object in `embedder` mutex cannot downcast; watch Arc shares Slot with fulfill/fail thread

### D-02: Pure helper `readiness_from_watch`
**Choice:** `readiness_from_watch(Option<&DeferredEmbedder>) -> EmbedderReadyState` — `None` → `Ready`  
**Why:** Unit-testable without constructing full `AppState`

### D-03: IPC `get_embedder_readiness`
**Choice:** Command returns `EmbedderReadinessView { state: String, message: Option<String> }` with `#[serde(rename_all = "camelCase")]`; states `"pending"|"ready"|"failed"`  
**Why:** Matches frontend contract for Phase 22; camelCase like newer IPC types

### D-04: reload_providers debt
**Choice:** Leave `reload_providers` / `save_config` sync `build_embedder` unchanged; comment that `deferred_embedder` is cold-start watch only  
**Why:** Spec marks unify as post-v1.12 debt; do not break Settings save

### D-05: Scope fence
**Choice:** No React/Settings/Vitest/version bump in this phase  
**Why:** Phases 22–23

## Claude's Discretion

- Exact placement of `EmbedderReadinessView` (config.rs preferred)
- Whether tests live in `state.rs` `#[cfg(test)]` only (preferred) vs additional tauri-app integration test

## Deferred Ideas

- Settings UI + poll — Phase 22
- Release smoke / 1.12.0 — Phase 23
- Deferred reload_providers — post-v1.12

## Success Criteria (from ROADMAP)

1. FastEmbed cold-start path keeps an `Option<Arc<DeferredEmbedder>>` watch on `AppState`
2. `get_embedder_readiness` returns `{ state, message }` for pending/ready/failed
3. Non-deferred providers report Ready without a deferred watch
4. Only `store` opens SQLite; `cargo check -p tauri-app` (+ readiness unit helpers) green
