# Phase 20: DeferredEmbedder readiness API — Context

**Gathered:** 2026-07-29  
**Status:** Ready for planning  
**Source:** Design + implementation plan express path (brainstorming approved; `/gsd-plan-phase 20 --chain`)

## Phase Goal

Callers can non-blockingly observe deferred FastEmbed init as Pending, Ready, or Failed, and failed/timed-out waits surface clear errors (BOOT-01, BOOT-04).

## Constraints

- Keep existing `DeferredEmbedder` + fulfill/fail/`embed` wait loop — harden, do not rewrite
- Do not change RAG / `RetrieverConfig::default`
- No auto-fallback embedder provider
- TDD: failing tests before production code
- Only `crates/store` opens SQLite (N/A for this crate-only phase)
- Spec: `docs/superpowers/specs/2026-07-29-release-hardening-design.md`
- Plan draft: `docs/superpowers/plans/2026-07-29-release-hardening.md` Task 1

## Decisions

### D-01: Expose `EmbedderReadyState` enum
**Choice:** `Pending | Ready | Failed { message: String }` in `crates/embedder`  
**Why:** Matches product Pending/Ready/Failed UI contract for later phases

### D-02: Non-blocking `ready_state(&self)`
**Choice:** Snapshot of internal `Slot` without waiting  
**Why:** BOOT-01 requires observable state without blocking the UI thread

### D-03: Injectable wait timeout
**Choice:** `with_wait_timeout(id, dim, Duration)`; `new()` delegates to `Duration::from_secs(300)`  
**Why:** Production timeout unchanged; tests can assert timeout without 300s waits (BOOT-04)

### D-04: Timeout / fail errors stay `EmbedError::Init`
**Choice:** Keep existing error strings pattern (`timed out waiting…`, `deferred embedder failed: {msg}`)  
**Why:** Downstream already maps Init; BOOT-04 needs clear errors, not new error types

### D-05: Phase 20 scope fence
**Choice:** No AppState, no IPC, no Settings UI in this phase  
**Why:** Those are Phases 21–22; Phase 20 is crate API + unit tests only

## Claude's Discretion

- Exact test names/structure within `deferred.rs` `#[cfg(test)]`
- Whether `EmbedderReadyState` derives `PartialEq`/`Eq`/`Clone`/`Debug` (recommended yes)

## Deferred Ideas

- AppState `deferred_embedder` watch — Phase 21
- `get_embedder_readiness` IPC — Phase 21
- Settings UI / polling — Phase 22
- Release bump / smoke — Phase 23
- Unify `reload_providers` with deferred path — post-v1.12 debt

## Success Criteria (from ROADMAP)

1. `ready_state()` returns Pending before fulfill/fail, Ready after fulfill, Failed with message after fail
2. `with_wait_timeout` allows short waits in tests; timeout while Pending returns a clear Init error
3. Production `new()` still uses the existing ~300s wait default
4. `cargo test -p embedder` green (including prior fulfill/fail tests)
