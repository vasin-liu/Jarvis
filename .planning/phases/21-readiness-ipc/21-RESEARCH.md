# Phase 21: Readiness IPC — Research

**Researched:** 2026-07-29  
**Domain:** Tauri shell IPC exposing `DeferredEmbedder` readiness to frontend (BOOT-02)  
**Confidence:** HIGH  
**Depends on:** Phase 20 (`EmbedderReadyState`, `ready_state()`) — shipped [VERIFIED: `crates/embedder/src/deferred.rs`, `20-02-SUMMARY.md`]

## User Constraints

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

### Claude's Discretion

- Exact placement of `EmbedderReadinessView` (config.rs preferred)
- Whether tests live in `state.rs` `#[cfg(test)]` only (preferred) vs additional tauri-app integration test

### Deferred Ideas

- Settings UI + poll — Phase 22
- Release smoke / 1.12.0 — Phase 23
- Deferred reload_providers — post-v1.12

## Project Constraints (from `.cursor/rules/`)

### TDD & Goal-Driven (`.cursor/rules/tdd-goal-driven.mdc`)

- State goal + acceptance checks before coding; Red → Green → Refactor.
- **This phase:** unit tests for `readiness_from_watch` in `state.rs` `#[cfg(test)]` [VERIFIED: plan Task 2, CONTEXT discretion]. No E2E — not user-facing until Phase 22 [VERIFIED: D-05].
- Done gate: `cargo test -p tauri-app readiness_` + `cargo check -p tauri-app` green [VERIFIED: ROADMAP Phase 21 success criteria #4].

### Surgical Changes (`.cursor/rules/karpathy-guidelines.mdc`)

- Touch only: `state.rs`, `commands/config.rs`, `commands/mod.rs`, `lib.rs` (handler registration) [VERIFIED: `docs/superpowers/plans/2026-07-29-release-hardening.md` Task 2 file map].
- Do not refactor `DeferredEmbedder`, RAG, retriever defaults, or frontend.
- Match existing IPC patterns (`IndexStatusView`, `ApiKeyStatus` in `config.rs`) [VERIFIED: `src-tauri/src/commands/config.rs`].

### Stack & Architecture (`.cursor/rules/jarvis-stack.mdc`, `AGENTS.md`)

- Rust **stable**, MSRV **1.85**; thin Tauri commands delegate to `AppState` helpers [VERIFIED: workspace conventions].
- **`store` sole SQLite owner** — readiness IPC must not open DB [VERIFIED: BOOT-02 in `.planning/REQUIREMENTS.md`].
- Errors at Tauri boundary: `Result<_, String>` on commands [VERIFIED: `get_index_status` pattern in `config.rs`].

### Global v1.12 constraints (spec/plan, relevant to Phase 21)

- Keep existing `DeferredEmbedder` + deferred `initial_scan`; no RAG/`RetrieverConfig` changes [VERIFIED: `docs/superpowers/plans/2026-07-29-release-hardening.md` Global Constraints].
- No auto-fallback embedder on FastEmbed failure.
- E2E/CI stays Mock — no real FastEmbed download in automated tests [VERIFIED: `src-tauri/src/e2e.rs` forces `EmbedderProvider::Mock`].

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| **BOOT-02** | App exposes embedder readiness over IPC without coupling UI to FastEmbed internals | `get_embedder_readiness` → `EmbedderReadinessView`; maps `EmbedderReadyState` to string states [VERIFIED: Phase 20 API ready] |

## Summary

Phase 21 bridges Phase 20's crate API to the Tauri shell. Production already creates `Arc<DeferredEmbedder>` on FastEmbed cold-start, spawns `FastEmbedder::try_new` on a background thread, and stores the same instance as `Arc<dyn Embedder>` in `AppState.embedder` [VERIFIED: `src-tauri/src/state.rs:68–87`]. The gap is retaining a **typed watch handle** (`Option<Arc<DeferredEmbedder>>`) before the trait-object cast, plus a thin IPC command. Non-deferred paths (Mock, Ollama, Cloud, E2E) report **Ready** via `None` watch [VERIFIED: D-02, E2E Mock path `state.rs:68–69` + `e2e.rs:23`].

---

## Architecture

### Current cold-start flow [VERIFIED: `state.rs`]

```
init_state()
  ├─ E2E OR non-FastEmbed → build_embedder() → Arc<dyn Embedder> (sync)
  └─ FastEmbed (non-E2E)
       ├─ Arc::new(DeferredEmbedder::new(...))
       ├─ thread::spawn → FastEmbedder::try_new → fulfill / fail
       └─ embedder mutex ← deferred as Arc<dyn Embedder>   ← typed Arc LOST here today
```

### Target wiring [VERIFIED: CONTEXT D-01, plan Task 2]

```
init_state()
  └─ FastEmbed branch:
       deferred = Arc::new(DeferredEmbedder::new(...))
       deferred_embedder = Some(deferred.clone())   ← NEW watch (same Arc as fulfill target)
       embedder mutex = deferred as Arc<dyn Embedder>
       spawn thread with deferred.clone() → fulfill/fail
  └─ else:
       deferred_embedder = None
       embedder mutex = build_embedder(...)

AppState.embedder_readiness()
  └─ readiness_from_watch(self.deferred_embedder.as_deref())

get_embedder_readiness (IPC)
  └─ readiness_view(state.embedder_readiness())  → EmbedderReadinessView
```

**Arc sharing:** `DeferredEmbedder` holds internal state in `pair: Arc<(Mutex<Slot>, Condvar)>` [VERIFIED: `deferred.rs:27`]. Cloning `Arc<DeferredEmbedder>` shares the same `Slot`; `ready_state()` reads it without blocking [VERIFIED: `deferred.rs:62–71`]. The background fulfill/fail thread and IPC polls therefore observe the same state machine.

**IPC registration pattern** [VERIFIED: existing commands]:

1. Define struct + `#[tauri::command]` in `commands/config.rs` (alongside `get_index_status`).
2. `pub use` in `commands/mod.rs` [VERIFIED: `commands/mod.rs:15–17` exports config commands].
3. Import + add to `generate_handler![...]` in `lib.rs` near `get_index_status` [VERIFIED: `lib.rs:61–70`].

**Frontend contract (Phase 22 consumer, not implemented here):** JSON `{ "state": "pending"|"ready"|"failed", "message": string|null }` via camelCase serde [VERIFIED: D-03, plan Task 3 `src/types/ipc.ts` sketch].

### File touch map

| File | Change |
|------|--------|
| `src-tauri/src/state.rs` | `deferred_embedder` field; init_state branch sets watch; `readiness_from_watch`; `embedder_readiness()`; `#[cfg(test)]` |
| `src-tauri/src/commands/config.rs` | `EmbedderReadinessView`, `get_embedder_readiness` |
| `src-tauri/src/commands/mod.rs` | re-export `get_embedder_readiness` |
| `src-tauri/src/lib.rs` | handler import + `generate_handler!` entry |

**Out of scope (D-05):** `src/types/ipc.ts`, `src/lib/tauri.ts`, `SettingsView`, Vitest, version bump.

---

## Pitfalls

### 1. Losing the typed Arc at cast time

**Problem:** After `deferred as Arc<dyn Embedder>`, the only handle is a trait object. Rust has no safe downcast from `Arc<dyn Embedder>` to `Arc<DeferredEmbedder>` without `Any` plumbing (not present today) [VERIFIED: `Embedder` trait has no `as_any`, `state.rs:87`].

**Fix:** Clone **before** cast: `Some(deferred.clone())` into `deferred_embedder`, then `deferred as Arc<dyn Embedder>` into mutex [VERIFIED: CONTEXT D-01].

**Warning sign:** Implementer tries `ready_state()` via `embedder.lock()` + downcast — will not compile or will require invasive trait changes.

### 2. Assuming `embedder` mutex reflects readiness after Settings reload

**Problem:** `reload_providers` replaces `embedder` mutex with sync `build_embedder(cfg)` [VERIFIED: `state.rs:151–154`]. It does **not** clear or update `deferred_embedder`. If user saves Settings switching FastEmbed → Mock, watch may still say `Pending` while mutex holds Mock.

**Mitigation (D-04):** Intentionally leave unchanged this phase. Document field:

```rust
// Cold-start watch only; Settings reload stays sync build_embedder (v1.12 debt).
deferred_embedder: Option<Arc<DeferredEmbedder>>,
```

Post-v1.12 debt: unify reload with deferred path or set `deferred_embedder = None` on reload [VERIFIED: ROADMAP backlog, plan Task 2 step 6].

**Callers of `reload_providers`:** `save_config`, `repopulate_runtime_api_key`, `reinit_and_rebuild_index` [VERIFIED: `state.rs:160`, `config.rs:27`, `commands/index.rs:44`] — all unaffected by design.

### 3. E2E / Mock path must report Ready

**Problem:** E2E skips FastEmbed branch (`is_e2e_mode()` false guard on line 68) and uses Mock [VERIFIED: `e2e.rs:23`, `state.rs:68–69`]. Watch must be `None` so `readiness_from_watch` → `Ready` [VERIFIED: D-02]. CI must not require FastEmbed for readiness tests.

### 4. Blocking in IPC handler

**Problem:** Calling `embed().await` or anything that waits on Condvar would block the Tauri command thread during pending init.

**Fix:** IPC uses only `ready_state()` (Mutex lock, no wait) [VERIFIED: Phase 20 D-02, `deferred.rs:62–71`]. Embed path still blocks on wait — unchanged and acceptable for index/RAG callers.

### 5. Stale Failed state after successful cold-start

**Not a pitfall:** Once `fulfill` runs, watch reads `Ready` permanently for that process lifetime [VERIFIED: `deferred.rs` Slot transition is one-way for a given instance]. New init only on app restart.

---

## Code Sketches

### `readiness_from_watch` + `AppState` (state.rs)

```rust
use embedder::{DeferredEmbedder, EmbedderReadyState};

// Cold-start watch only; Settings reload stays sync build_embedder (v1.12 debt).
pub(crate) deferred_embedder: Option<Arc<DeferredEmbedder>>,

pub(crate) fn readiness_from_watch(
    deferred: Option<&DeferredEmbedder>,
) -> EmbedderReadyState {
    match deferred {
        None => EmbedderReadyState::Ready,
        Some(d) => d.ready_state(),
    }
}

impl AppState {
    pub(crate) fn embedder_readiness(&self) -> EmbedderReadyState {
        readiness_from_watch(self.deferred_embedder.as_deref())
    }
}
```

**init_state FastEmbed branch change** [VERIFIED: extends `state.rs:73–87`]:

```rust
let deferred = Arc::new(DeferredEmbedder::new(
    format!("fastembed:{}", config.embedding.fastembed_model),
    config_dim,
));
let deferred_embedder = Some(deferred.clone());
let pending = deferred.clone();
// ... spawn thread unchanged ...
let embedder = deferred as Arc<dyn Embedder>;

// In Ok(AppState { ... }):
deferred_embedder,  // Some on FastEmbed path, None on else branch
```

### `EmbedderReadinessView` + IPC (config.rs)

```rust
use embedder::EmbedderReadyState;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbedderReadinessView {
    pub state: String,
    pub message: Option<String>,
}

fn readiness_view(state: EmbedderReadyState) -> EmbedderReadinessView {
    match state {
        EmbedderReadyState::Pending => EmbedderReadinessView {
            state: "pending".into(),
            message: None,
        },
        EmbedderReadyState::Ready => EmbedderReadinessView {
            state: "ready".into(),
            message: None,
        },
        EmbedderReadyState::Failed { message } => EmbedderReadinessView {
            state: "failed".into(),
            message: Some(message),
        },
    }
}

#[tauri::command]
pub fn get_embedder_readiness(
    state: State<'_, AppState>,
) -> Result<EmbedderReadinessView, String> {
    Ok(readiness_view(state.embedder_readiness()))
}
```

Alternative: inline match in command body (plan Task 2) — equivalent; extracting `readiness_view` keeps command thin and testable without Tauri [VERIFIED: mirrors `index_status_view` separation in `index_ops.rs`].

### Unit tests (state.rs `#[cfg(test)]`) — preferred [VERIFIED: CONTEXT discretion]

```rust
#[cfg(test)]
mod readiness_tests {
    use super::readiness_from_watch;
    use embedder::{DeferredEmbedder, EmbedderReadyState};

    #[test]
    fn no_deferred_is_ready() {
        assert_eq!(readiness_from_watch(None), EmbedderReadyState::Ready);
    }

    #[test]
    fn deferred_pending_and_failed() {
        let d = DeferredEmbedder::new("deferred:t", 8);
        assert_eq!(readiness_from_watch(Some(&d)), EmbedderReadyState::Pending);
        d.fail("boom");
        assert!(matches!(
            readiness_from_watch(Some(&d)),
            EmbedderReadyState::Failed { .. }
        ));
    }
}
```

**TDD sequence** [VERIFIED: plan Task 2, `tdd-goal-driven.mdc`]:

1. Add failing tests for `readiness_from_watch`.
2. `cargo test -p tauri-app readiness_` — expect fail until helper exists.
3. Implement field + init + IPC + handler registration.
4. `cargo test -p tauri-app readiness_` + `cargo check -p tauri-app` — green.

---

## Existing Patterns to Mirror

| Pattern | Reference | Use in Phase 21 |
|---------|-----------|-----------------|
| camelCase IPC struct | `ApiKeyStatus`, `IndexStatusView` [VERIFIED: `config.rs:12–16`, `index_ops.rs:33`] | `EmbedderReadinessView` |
| Config-area status command | `get_index_status` [VERIFIED: `config.rs:35–42`] | `get_embedder_readiness` placement |
| Pure helper + command | `index_status_view(store, cfg, embedder)` [VERIFIED: `index_ops.rs:44`] | `readiness_from_watch` + `embedder_readiness` |
| Phase 20 re-export | `embedder::EmbedderReadyState` [VERIFIED: `crates/embedder/src/lib.rs:8`] | Import in state.rs / config.rs |

---

## Success Criteria Mapping

| # | Criterion | Verification |
|---|-----------|--------------|
| 1 | FastEmbed cold-start keeps `Option<Arc<DeferredEmbedder>>` on AppState | `init_state` sets `Some(deferred.clone())` before trait cast [VERIFIED: ROADMAP] |
| 2 | `get_embedder_readiness` returns `{ state, message }` | IPC command + `EmbedderReadinessView` serde [VERIFIED: D-03] |
| 3 | Non-deferred providers report Ready | `deferred_embedder: None` → `readiness_from_watch` → Ready [VERIFIED: D-02] |
| 4 | store-only SQLite; shell compiles | No store imports in new code; `cargo check -p tauri-app` [VERIFIED: ROADMAP #4] |

---

## Phase 22 Handoff

Phase 22 will consume `get_embedder_readiness` from React (`invoke`), add TS types, Settings banner with `data-testid`s, and Vitest [VERIFIED: plan Task 3, ROADMAP Phase 22]. Phase 21 only needs the IPC endpoint stable and string states aligned with `"pending"|"ready"|"failed"`.

---

*Phase: 21-readiness-ipc*  
*Researched: 2026-07-29*
