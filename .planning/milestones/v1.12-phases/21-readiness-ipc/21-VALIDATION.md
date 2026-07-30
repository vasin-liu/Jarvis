---
phase: 21
slug: readiness-ipc
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-29
---

# Phase 21 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Shell IPC phase — behavioral contracts covered by `cargo test -p tauri-app readiness_`; compile gate via `cargo check -p tauri-app`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test`; `DeferredEmbedder` + `EmbedderReadyState` from Phase 20 |
| **Config file** | `src-tauri/Cargo.toml` (existing workspace deps) |
| **Quick run command** | `cargo test -p tauri-app readiness_ -- --test-threads=4` |
| **RED gate command** | `cargo test -p tauri-app readiness_ -- --nocapture` (expect compile fail in Wave 1) |
| **Compile gate command** | `cargo check -p tauri-app` |
| **Full shell unit filter** | `cargo test -p tauri-app no_deferred_is_ready deferred_pending deferred_failed` |
| **Estimated runtime** | &lt; 30 seconds |

---

## Sampling Rate

- **After Plan 21-01 (RED):** Run RED gate commands; expect non-zero exit (compile fail for `readiness_from_watch`)
- **After every task in Plan 21-02:** Run task `<automated>` verify from plan
- **After Wave 2 complete:** Run `cargo test -p tauri-app readiness_` + `cargo check -p tauri-app`
- **Before `/gsd-verify-work`:** Readiness unit tests and tauri-app check must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Decision Ref | Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|--------------|----------|-----------|-------------------|-------------|--------|
| 21-01-01 | 01 | 1 | BOOT-02 | D-02 | readiness_from_watch(None) → Ready | unit | `cargo test -p tauri-app no_deferred_is_ready` | ❌ W1 | ⬜ pending |
| 21-01-02 | 01 | 1 | BOOT-02 | D-01 | readiness_from_watch(Some) → Pending | unit | `cargo test -p tauri-app deferred_pending` | ❌ W1 | ⬜ pending |
| 21-01-03 | 01 | 1 | BOOT-02 | D-02, D-03 | readiness_from_watch after fail → Failed + message | unit | `cargo test -p tauri-app deferred_failed` | ❌ W1 | ⬜ pending |
| 21-02-01 | 02 | 2 | BOOT-02 | D-02 | readiness_from_watch + embedder_readiness compile; tests pass | unit | `cargo test -p tauri-app readiness_ -- --test-threads=4` | ❌ W2 | ⬜ pending |
| 21-02-02 | 02 | 2 | BOOT-02 | D-01, D-04 | init_state wires deferred_embedder on FastEmbed path | compile | `cargo check -p tauri-app` | ❌ W2 | ⬜ pending |
| 21-02-03 | 02 | 2 | BOOT-02 | D-03 | EmbedderReadinessView + get_embedder_readiness | compile | `cargo check -p tauri-app` | ❌ W2 | ⬜ pending |
| 21-02-04 | 02 | 2 | BOOT-02 | D-03 | Handler registered in lib.rs | compile | `cargo check -p tauri-app` | ❌ W2 | ⬜ pending |
| 21-02-05 | 02 | 2 | BOOT-02 | D-01..D-05 | Full regression gate | unit + compile | `cargo test -p tauri-app readiness_ && cargo check -p tauri-app` | ❌ W2 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

### Phase 20 dependency (must be green before Wave 1)

| Test suite | Requirement | Command | Status |
|------------|-------------|---------|--------|
| embedder ready_state API | Phase 21 consumes EmbedderReadyState | `cargo test -p embedder -- --test-threads=4` | ⬜ pending |

---

## Wave 1 Requirements (RED)

- [ ] Three tests in `src-tauri/src/state.rs` `#[cfg(test)] mod readiness_tests`
- [ ] Tests reference `readiness_from_watch` before implementation
- [ ] RED commands exit non-zero (compile error — expected)
- [ ] No production API, IPC, or init_state changes in Wave 1 (D-05)

---

## Wave 2 Requirements (GREEN)

- [ ] `AppState.deferred_embedder: Option<Arc<DeferredEmbedder>>` on cold-start path (D-01)
- [ ] `readiness_from_watch` + `embedder_readiness` non-blocking (D-02)
- [ ] `get_embedder_readiness` → camelCase `{ state, message }` (D-03)
- [ ] `reload_providers` unchanged; field comment documents debt (D-04)
- [ ] Scope fence: no src/, e2e/, version bump (D-05)
- [ ] `cargo test -p tauri-app readiness_` exits 0
- [ ] `cargo check -p tauri-app` exits 0

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| IPC invoke returns JSON at runtime | BOOT-02 | Tauri invoke needs running app; Phase 22 Vitest mocks cover consumer | Optional dev smoke: `invoke('get_embedder_readiness')` in E2E Mock app → `{ state: "ready", message: null }` |
| FastEmbed cold-start watch shares Arc with fulfill thread | D-01 | Concurrency correctness; unit tests cover state machine not thread timing | Code review: `Some(deferred.clone())` before cast; spawn uses `pending = deferred.clone()` |
| reload_providers stale watch after Settings provider switch | D-04 | Intentional v1.12 debt | Code review: `reload_providers` does not touch `deferred_embedder` |
| Production FastEmbed pending→ready lifecycle | ROADMAP | Real ONNX load is release smoke (Phase 23) | Not in Phase 21 scope |

*Behavioral contracts for the pure helper and IPC mapping are fully automatable.*

---

## Nyquist Compliance Rationale

| Criterion | Assessment |
|-----------|------------|
| Every task has `<automated>` verify | ✅ All 8 tasks mapped |
| Wave 1 tests before implementation | ✅ Plan 21-01 RED before 21-02 GREEN |
| Sampling continuity | ✅ No 3 consecutive tasks without automated verify |
| Feedback latency &lt; 60s | ✅ tauri-app readiness filter &lt; 30s |
| E2E gap | ✅ N/A — IPC not user-facing until Phase 22 (D-05); BOOT-03 covers UI + Vitest |
| Store boundary | ✅ No new SQLite access; BOOT-02 satisfied at IPC layer |

**Nyquist notes:**

1. **Wave 0:** Phase 20 embedder crate must be GREEN before 21-01 — dependency sampling, not optional.
2. **RED vs GREEN signal:** Wave 1 success = compile **failure** on missing `readiness_from_watch`; do not misread RED exit 101 as regression.
3. **IPC runtime:** `get_embedder_readiness` has no dedicated invoke integration test this phase; compile + pure helper tests satisfy Nyquist for shell-only delivery. Phase 22 adds consumer-side Vitest with mocked invoke — closes the observation loop for BOOT-03, not BOOT-02.
4. **No watch-mode:** Standard `cargo test` one-shot runs only.
5. **Blocking pitfall:** If executor adds `embed().await` in IPC handler, tests may still pass but violates D-02 — code review checkpoint on Task 21-02-03.

**`nyquist_compliant: true`** — TDD RED/GREEN waves with per-task `cargo test` / `cargo check`; manual-only items are post-v1.12 smoke or Phase 22 consumer verification.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 1 RED verified before Wave 2 GREEN
- [ ] Phase 20 embedder tests green before Wave 1
- [ ] No watch-mode flags
- [ ] Feedback latency &lt; 60s
- [ ] `nyquist_compliant: true` set in frontmatter
- [ ] BOOT-02 fully mapped to automated unit + compile verification

**Approval:** pending
