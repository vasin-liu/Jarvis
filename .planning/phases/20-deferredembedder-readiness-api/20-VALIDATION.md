---
phase: 20
slug: deferredembedder-readiness-api
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-29
---

# Phase 20 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Crate-only unit-test phase — all behaviors automatable via `cargo test -p embedder`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` + `tokio` async tests; `MockEmbedder` |
| **Config file** | `crates/embedder/Cargo.toml` (existing workspace deps) |
| **Quick run command** | `cargo test -p embedder ready_state_ short_timeout_errors_while_pending -- --test-threads=4` |
| **RED gate command** | `cargo test -p embedder ready_state_ -- --nocapture` (expect compile fail in Wave 1) |
| **Full suite command** | `cargo test -p embedder` |
| **Estimated runtime** | &lt; 10 seconds |

---

## Sampling Rate

- **After Plan 20-01 (RED):** Run RED gate commands; expect non-zero exit (compile fail)
- **After every task in Plan 20-02:** Run task `<automated>` verify from plan
- **After Wave 2 complete:** Run `cargo test -p embedder`
- **Before `/gsd-verify-work`:** Full embedder suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Decision Ref | Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|--------------|----------|-----------|-------------------|-------------|--------|
| 20-01-01 | 01 | 1 | BOOT-01 | D-01, D-02 | ready_state Pending → Ready after fulfill | unit | `cargo test -p embedder ready_state_pending_then_ready` | ❌ W1 | ⬜ pending |
| 20-01-02 | 01 | 1 | BOOT-01 | D-01 | ready_state Failed includes fail message | unit | `cargo test -p embedder ready_state_failed_includes_message` | ❌ W1 | ⬜ pending |
| 20-01-03 | 01 | 1 | BOOT-04 | D-03, D-04 | embed while Pending times out with clear Init error | unit async | `cargo test -p embedder short_timeout_errors_while_pending` | ❌ W1 | ⬜ pending |
| 20-02-01 | 02 | 2 | BOOT-01, BOOT-04 | D-01, D-03 | Types + constructors compile | unit | `cargo check -p embedder` | ❌ W2 | ⬜ pending |
| 20-02-02 | 02 | 2 | BOOT-01, BOOT-04 | D-02, D-04 | ready_state + wait_timeout wired | unit | `cargo test -p embedder ready_state_ short_timeout_errors_while_pending -- --test-threads=4` | ❌ W2 | ⬜ pending |
| 20-02-03 | 02 | 2 | BOOT-01 | D-01 | EmbedderReadyState re-exported | unit | `cargo test -p embedder -- --test-threads=4` | ❌ W2 | ⬜ pending |
| 20-02-04 | 02 | 2 | BOOT-01, BOOT-04 | D-03, D-05 | Full regression + legacy deferred tests | unit | `cargo test -p embedder` | ❌ W2 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

### Legacy tests (must stay green after Wave 2)

| Test name | Requirement | Command | Status |
|-----------|-------------|---------|--------|
| `fulfill_unblocks_embed` | No regressions in fulfill/wait path | `cargo test -p embedder fulfill_unblocks_embed` | ⬜ pending |
| `fail_surfaces_error` | D-04 fail string preserved | `cargo test -p embedder fail_surfaces_error` | ⬜ pending |

---

## Wave 1 Requirements (RED)

- [ ] Three new tests in `crates/embedder/src/deferred.rs` `#[cfg(test)]`
- [ ] Tests reference `EmbedderReadyState`, `ready_state`, `with_wait_timeout` before implementation
- [ ] RED commands exit non-zero (compile error — expected)
- [ ] No production API added in Wave 1

---

## Wave 2 Requirements (GREEN)

- [ ] `EmbedderReadyState` public enum (D-01)
- [ ] `ready_state()` non-blocking (D-02)
- [ ] `with_wait_timeout` + `new()` → 300s (D-03)
- [ ] Init error strings unchanged (D-04)
- [ ] Scope fence: embedder crate only (D-05)
- [ ] `cargo test -p embedder` exits 0

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Production FastEmbed cold-start 300s wait | ROADMAP SC#3 | Real ONNX load is release smoke (Phase 23), not unit test | Code review: `new()` delegates to `Duration::from_secs(300)` |
| Tauri deferred path unchanged | D-05 | Out of Phase 20 scope | Confirm no diff under `src-tauri/` |

*All Phase 20 behavioral contracts have automated unit verification.*

---

## Nyquist Compliance Rationale

| Criterion | Assessment |
|-----------|------------|
| Every task has `<automated>` verify | ✅ All 7 tasks mapped |
| Wave 0/1 tests before implementation | ✅ Plan 20-01 RED before 20-02 GREEN |
| Sampling continuity | ✅ No 3 consecutive tasks without automated verify |
| Feedback latency &lt; 60s | ✅ embedder suite &lt; 10s |
| E2E gap | ✅ N/A — not user-facing; Phase 22 covers UI |

**`nyquist_compliant: true`** — crate-only TDD phase with full `cargo test -p embedder` coverage; no manual-only behaviors in scope.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 1 RED verified before Wave 2 GREEN
- [ ] No watch-mode flags
- [ ] Feedback latency &lt; 60s
- [ ] `nyquist_compliant: true` set in frontmatter
- [ ] BOOT-01 and BOOT-04 fully mapped to automated tests

**Approval:** pending
