# Phase 20: DeferredEmbedder Readiness API — Research

**Researched:** 2026-07-29  
**Domain:** Deferred FastEmbed init observability (Rust library API, `embedder` crate only)  
**Confidence:** HIGH  
**Requirements:** BOOT-01, BOOT-04 only

## User Constraints

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

### Claude's Discretion

- Exact test names/structure within `deferred.rs` `#[cfg(test)]`
- Whether `EmbedderReadyState` derives `PartialEq`/`Eq`/`Clone`/`Debug` (recommended yes)

### Deferred Ideas

- AppState `deferred_embedder` watch — Phase 21
- `get_embedder_readiness` IPC — Phase 21
- Settings UI / polling — Phase 22
- Release bump / smoke — Phase 23
- Unify `reload_providers` with deferred path — post-v1.12 debt

## Project Constraints (from `.cursor/rules/`)

### TDD & Goal-Driven (`.cursor/rules/tdd-goal-driven.mdc`)

- State goal + acceptance checks before coding.
- Red → Green → Refactor: failing unit test first, minimal implementation, then refactor with tests green.
- **This phase:** unit tests only in `crates/embedder` (`cargo test -p embedder`). No E2E (not user-facing). No integration tests required unless behavior crosses crate boundaries — Phase 20 does not.
- Done gate: `cargo test -p embedder` green, including existing `fulfill_unblocks_embed` and `fail_surfaces_error`.

### Surgical Changes (`.cursor/rules/karpathy-guidelines.mdc`)

- Harden existing `DeferredEmbedder`; do not rewrite the fulfill/fail/`embed` wait loop.
- Touch only `crates/embedder/src/deferred.rs` and `crates/embedder/src/lib.rs` (re-export).
- Do not refactor adjacent embedder providers, Tauri setup, or error enum variants.

### Stack (`.cursor/rules/jarvis-stack.mdc`)

- Rust **stable**, MSRV **1.85**, edition **2021** [VERIFIED: workspace `Cargo.toml`].
- Errors via **`thiserror`** — reuse `EmbedError::Init` [VERIFIED: `crates/embedder/src/error.rs`].
- Async via **`tokio`** + **`async-trait`** on `Embedder` trait [VERIFIED: `crates/embedder/Cargo.toml`, `lib.rs`].
- **No new dependencies** for this phase — `std::sync::{Mutex, Condvar, Arc}` and `std::time::Duration` already in use [VERIFIED: `deferred.rs`].

### Global v1.12 constraints (from spec/plan, relevant to Phase 20)

- Do not change RAG / `RetrieverConfig::default` [VERIFIED: out of scope — no retriever edits].
- No auto-fallback embedder provider on FastEmbed failure.
- `store` remains sole SQLite owner (N/A — embedder crate has no DB).

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| **BOOT-01** | Non-blocking readiness API: Pending / Ready / Failed (+ message) on `DeferredEmbedder` | Add `EmbedderReadyState` + `ready_state()` snapshot over existing `Slot` enum [VERIFIED: internal `Slot` already models three states in `deferred.rs`] |
| **BOOT-04** | Failed/timed-out embed attempts return clear errors, not indefinite wait | Existing `fail()` → `EmbedError::Init("deferred embedder failed: {msg}")`; timeout path → `"timed out waiting for embedder init"` — preserve strings; make timeout injectable for tests [VERIFIED: `wait_ready()` lines 51–66] |

## Summary

Phase 20 is a **crate-only hardening** of the existing `DeferredEmbedder` introduced in v1.9 UAT [VERIFIED: `src-tauri/src/state.rs` FastEmbed cold-start path]. The internal state machine (`Slot::Pending | Ready | Failed`) already exists; the gap is a **public, non-blocking observer** (`ready_state`) and a **testable wait timeout** (`with_wait_timeout`) without changing production's 300s default.

**Primary recommendation:** Follow Task 1 in `docs/superpowers/plans/2026-07-29-release-hardening.md` — three new tests, two new public types/methods, one struct field, one line change in `wait_ready`. Re-export `EmbedderReadyState` from `lib.rs`. No Tauri, IPC, or frontend work.

## Architecture (Existing Baseline — Preserve)

```
DeferredEmbedder (placeholder Arc<dyn Embedder>)
  ├─ pair: Arc<(Mutex<Slot>, Condvar)>
  │     Slot::Pending | Ready(Arc<dyn Embedder>) | Failed(String)
  ├─ fulfill(embedder) → Ready + notify_all
  ├─ fail(msg)         → Failed + notify_all
  ├─ embed()           → wait_ready() → inner.embed()
  └─ [NEW] ready_state() → non-blocking match on Slot
       wait_ready()    → [CHANGE] use self.wait_timeout instead of hardcoded 300s
```

**Production wiring (unchanged this phase):** Tauri setup spawns background thread → `FastEmbedder::try_new` → `fulfill` / `fail` on the same `DeferredEmbedder` instance stored as `Arc<dyn Embedder>` [VERIFIED: `src-tauri/src/state.rs:68–87]. Phase 21 will add `AppState.deferred_embedder` watch handle — **out of scope here** (D-05).

**E2E / Mock path:** E2E uses `build_embedder` synchronously; never constructs `DeferredEmbedder` [VERIFIED: `state.rs` — deferred only when `EmbedderProvider::FastEmbed` and not E2E].

## Standard Stack

### Core (this phase)

| Component | Location | Purpose | Why Standard |
|-----------|----------|---------|--------------|
| `DeferredEmbedder` | `crates/embedder/src/deferred.rs` | Placeholder + wait loop | Existing v1.9 cold-start fix [VERIFIED: codebase] |
| `Slot` (private) | same file | Pending/Ready/Failed storage | Already maps 1:1 to `EmbedderReadyState` [VERIFIED: `deferred.rs:8–12`] |
| `Mutex` + `Condvar` | `std::sync` | Block `embed()` until ready | Existing pattern; no async mutex needed for init gate [VERIFIED: `deferred.rs:19,45–71`] |
| `EmbedError::Init` | `crates/embedder/src/error.rs` | Fail + timeout surface | `#[error("embedder init: {0}")]` [VERIFIED: `error.rs:13–14`] |
| `MockEmbedder` | `crates/embedder/src/mock.rs` | Test double for fulfill path | Used by existing deferred tests [VERIFIED: `deferred.rs` tests] |
| `tokio` (dev) | workspace | `#[tokio::test]` for async embed tests | Existing deferred test harness [VERIFIED: `embedder/Cargo.toml`] |

### Files to modify

| File | Change |
|------|--------|
| `crates/embedder/src/deferred.rs` | `EmbedderReadyState`, `wait_timeout` field, `with_wait_timeout`, `ready_state`, update `wait_ready`, new tests |
| `crates/embedder/src/lib.rs` | `pub use deferred::{DeferredEmbedder, EmbedderReadyState}` |

**Installation:** None — zero new crates.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Readiness state machine | New atomic enum / channel / watch crate | Snapshot existing `Slot` via `ready_state()` | State already in `Mutex<Slot>` [VERIFIED: codebase] |
| Async notification for observers | `tokio::sync::watch`, callbacks | Non-blocking snapshot only (D-02); Phase 22 polls IPC | YAGNI for Phase 20; UI polling is Phase 22 |
| Timeout testing | Sleep 300s in CI | `with_wait_timeout(..., Duration::from_millis(30))` (D-03) | Plan Task 1 pattern [VERIFIED: release-hardening plan] |
| New error variant for timeout/fail | `EmbedError::DeferredTimeout` | Keep `EmbedError::Init` with existing strings (D-04) | Downstream mapping unchanged [VERIFIED: `deferred.rs:52–65`] |
| Rewrite init architecture | Replace Condvar loop | Parameterize timeout on existing loop | Spec §4.1: harden, not rewrite [VERIFIED: design doc] |
| Readiness in Tauri this phase | `get_embedder_readiness` now | Defer to Phase 21 (D-05) | Scope fence |

**Key insight:** Phase 20 exposes what `DeferredEmbedder` already knows internally — no new concurrency primitives.

## Common Pitfalls

### Pitfall 1: Accidentally changing production 300s wait
**What goes wrong:** Tests pass but release cold-start times out sooner or waits longer.  
**Why:** Replacing `Duration::from_secs(300)` globally instead of via `self.wait_timeout`.  
**How to avoid:** `new()` must delegate to `with_wait_timeout(..., Duration::from_secs(300))` (D-03); only tests call short timeouts.  
**Warning signs:** `DeferredEmbedder::new` body no longer mentions 300.  
[VERIFIED: current hardcode at `deferred.rs:58`]

### Pitfall 2: Blocking inside `ready_state()`
**What goes wrong:** UI/shell thread hangs when polling readiness (BOOT-01 violated).  
**Why:** Calling `wait_ready()`, `Condvar::wait`, or `embed()` from `ready_state`.  
**How to avoid:** Lock `Mutex<Slot>`, match, clone message if Failed, return immediately — no wait (D-02).  
**Warning signs:** `ready_state` takes `&mut self` or returns `Result` from wait.

### Pitfall 3: Holding mutex across `.await` in `embed()`
**What goes wrong:** Deadlock or long critical sections (pre-existing risk if refactor is careless).  
**Why:** `wait_ready` is sync and called before `inner.embed().await` — current code drops lock before await [VERIFIED: `deferred.rs:84–87`].  
**How to avoid:** Do not merge `wait_ready` into async block while holding lock; keep existing sequence.

### Pitfall 4: Changing error strings
**What goes wrong:** Phase 21 UI / existing tests break on message text; BOOT-04 satisfied but regressions elsewhere.  
**Why:** D-04 locks string patterns.  
**How to avoid:** Preserve exactly `"deferred embedder failed: {msg}"` and `"timed out waiting for embedder init"`.  
**Warning signs:** New `EmbedError` variant or reworded Init messages.

### Pitfall 5: Scope creep into AppState / IPC
**What goes wrong:** Phase 20 ballooning into user-facing work without E2E plan for this PR.  
**Why:** Plan Task 2–3 are separate phases.  
**How to avoid:** D-05 fence — stop at `lib.rs` re-export; do not touch `state.rs` yet.  
**Warning signs:** `deferred_embedder` field added in same commit.

### Pitfall 6: Breaking existing deferred tests
**What goes wrong:** `fulfill_unblocks_embed` / `fail_surfaces_error` fail after refactor.  
**Why:** Constructor signature or wait behavior change.  
**How to avoid:** Run full `cargo test -p embedder` after implementation; keep `fulfill`/`fail`/`embed` semantics identical aside from timeout field.  
**Warning signs:** Only new tests run in CI locally.

### Pitfall 7: Forgetting `PartialEq` on `EmbedderReadyState`
**What goes wrong:** Phase 21 shell tests awkward (`matches!` only).  
**Why:** Discretion recommends `PartialEq`/`Eq`/`Clone`/`Debug`.  
**How to avoid:** `#[derive(Debug, Clone, PartialEq, Eq)]` on public enum — enables `assert_eq!(ready_state(), Pending)`.

## Code Examples

### Intended public types (D-01, D-03)

```rust
// crates/embedder/src/deferred.rs — new public API
// [INTENDED: matches CONTEXT D-01, D-03, plan Task 1]

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmbedderReadyState {
    Pending,
    Ready,
    Failed { message: String },
}

impl DeferredEmbedder {
    pub fn new(id: impl Into<String>, dim: usize) -> Self {
        Self::with_wait_timeout(id, dim, Duration::from_secs(300))
    }

    pub fn with_wait_timeout(
        id: impl Into<String>,
        dim: usize,
        wait_timeout: Duration,
    ) -> Self {
        Self {
            id: id.into(),
            dim,
            wait_timeout,
            pair: Arc::new((Mutex::new(Slot::Pending), Condvar::new())),
        }
    }

    /// Non-blocking snapshot (D-02, BOOT-01).
    pub fn ready_state(&self) -> EmbedderReadyState {
        let (lock, _) = &*self.pair;
        let slot = lock.lock().unwrap();
        match &*slot {
            Slot::Pending => EmbedderReadyState::Pending,
            Slot::Ready(_) => EmbedderReadyState::Ready,
            Slot::Failed(msg) => EmbedderReadyState::Failed {
                message: msg.clone(),
            },
        }
    }
}
```

### `wait_ready` timeout injection (D-03, D-04)

```rust
// Replace hardcoded Duration::from_secs(300) with self.wait_timeout
// [VERIFIED: current loop at deferred.rs:45-71; INTENDED change only line 58]

Slot::Pending => {
    let (next, result) = cv
        .wait_timeout(slot, self.wait_timeout)  // was: Duration::from_secs(300)
        .unwrap();
    slot = next;
    if result.timed_out() {
        if matches!(&*slot, Slot::Pending) {
            return Err(EmbedError::Init(
                "timed out waiting for embedder init".into(),
            ));
        }
    }
}
```

### Re-export (lib.rs)

```rust
// crates/embedder/src/lib.rs
// [INTENDED: plan Task 1 step 6]

pub use deferred::{DeferredEmbedder, EmbedderReadyState};
```

### Test shapes (from plan Task 1 — TDD red phase)

```rust
#[test]
fn ready_state_pending_then_ready() {
    let deferred = DeferredEmbedder::new("deferred:test", 8);
    assert_eq!(deferred.ready_state(), EmbedderReadyState::Pending);
    deferred.fulfill(Arc::new(MockEmbedder::new(8)));
    assert_eq!(deferred.ready_state(), EmbedderReadyState::Ready);
}

#[test]
fn ready_state_failed_includes_message() {
    let deferred = DeferredEmbedder::new("deferred:test", 8);
    deferred.fail("onnx missing");
    match deferred.ready_state() {
        EmbedderReadyState::Failed { message } => {
            assert!(message.contains("onnx missing"));
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

#[tokio::test]
async fn short_timeout_errors_while_pending() {
    let deferred = DeferredEmbedder::with_wait_timeout(
        "deferred:test",
        8,
        Duration::from_millis(30),
    );
    let err = deferred
        .embed(&[String::from("hi")])
        .await
        .expect_err("should time out");
    assert!(err.to_string().contains("timed out"), "got: {err}");
}
```

### Error mapping unchanged (D-04, BOOT-04)

```rust
// fail path — preserve [VERIFIED: deferred.rs:51-54]
Slot::Failed(msg) => {
    return Err(EmbedError::Init(format!(
        "deferred embedder failed: {msg}"
    )));
}

// Existing fail_surfaces_error test expects "boom" in err.to_string() [VERIFIED: deferred.rs:114-115]
```

## Current Codebase Gap Analysis

| Capability | Exists today? | Phase 20 action |
|------------|---------------|-----------------|
| Internal Pending/Ready/Failed | Yes (`Slot` enum) | Expose via `EmbedderReadyState` |
| `fulfill` / `fail` | Yes | No behavior change |
| `embed()` wait loop | Yes (300s hardcoded) | Use `self.wait_timeout` |
| `ready_state()` | **No** | Add |
| `with_wait_timeout()` | **No** | Add |
| `EmbedderReadyState` export | **No** | Add + re-export |
| Unit tests fulfill/fail | Yes (2 tests) | Keep green + add 3 tests |
| AppState watch | **No** | Phase 21 |
| IPC / UI | **No** | Phases 21–22 |

[VERIFIED: codebase — `deferred.rs`, `lib.rs`]

## Verification Commands

```bash
# Red phase (expect compile fail / missing symbols)
cargo test -p embedder ready_state_ -- --nocapture
cargo test -p embedder short_timeout_errors_while_pending -- --nocapture

# Green phase (full crate)
cargo test -p embedder -- --test-threads=4
```

Expected: all tests pass including `fulfill_unblocks_embed` and `fail_surfaces_error`.  
[VERIFIED: ROADMAP Phase 20 success criteria #4]

## Spec / Plan Alignment

| Source | Phase 20 coverage |
|--------|-------------------|
| Design §4.1 | Keep deferred path; harden `DeferredEmbedder` |
| Design §4.2 | Readiness observation starts at crate API (shell in later phases) |
| Design §5 | Init fail + timeout errors; no auto-fallback |
| Design §6.1 | Unit: fulfill / fail / timeout with injectable wait |
| Plan Task 1 | Exact API + test list for implementation |
| ROADMAP SC 1–4 | Maps to `ready_state`, `with_wait_timeout`, 300s default, `cargo test -p embedder` |

## Blockers

**None identified.** All work is localized to `embedder` crate with existing patterns and no new dependencies. Phase 21 depends on this API but does not block research/planning.

## Open Questions (for planner discretion only)

1. **Test timeout duration:** Plan uses 30ms — sufficient on Windows CI? If flaky, bump to 50–100ms while staying ≪ 300s. [ASSUMED: 30ms adequate under `--test-threads=4`]
2. **`ready_state` after Ready:** Should repeated calls stay `Ready` forever? Yes — matches fulfilled slot until process restart; no reset API needed. [VERIFIED: `Slot` never reverts in current code]

---

*Research complete. Ready for `/gsd-plan-phase 20`.*
