---
phase: 07
slug: wiki-kind-config
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-17
---

# Phase 07 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + Vitest (frontend) |
| **Config file** | `Cargo.toml` workspace; `vite.config.ts` (Vitest) |
| **Quick run command** | `cargo test -p store -p config` && `npm test -- --run src/lib/sourceDisplay.test.ts` |
| **Full suite command** | `cargo test -p store -p config` && `npm test` |
| **Estimated runtime** | ~60–180 seconds (cold compile higher) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p store -p config` and Vitest for touched FE files
- **After every plan wave:** Run full suite command above
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 07-01-01 | 01 | 1 | WIKI-02 | — | N/A | unit | `cargo test -p store` | ❌ W0 | ⬜ pending |
| 07-02-01 | 02 | 1 | WIKI-01 | — | Defaults off; no Settings | unit | `cargo test -p config` | ❌ W0 | ⬜ pending |
| 07-02-02 | 02 | 1 | WIKI-02 | — | Label only | unit | `npm test -- --run src/lib/sourceDisplay.test.ts` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Store test stubs / extensions for `wiki_page` round-trip (extend `source_kind_roundtrips` or add `wiki_page_kind_roundtrips`)
- [ ] Config tests for missing `wiki` key → both false; explicit `wiki` object round-trip
- [ ] Vitest case: `sourceKindLabel("wiki_page") === "笔记页"`

*Existing infrastructure (cargo test + Vitest) covers frameworks — Wave 0 is test stubs only, not framework install.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| No Settings wiki toggle | D-10 / WIKI-01 | No automated Settings absence assert in phase | Open Settings; confirm no wiki enable control |
| `auto_on_insights` unwired | D-09 | Negative compile/grep check optional | Confirm `insights_ops` does not read `wiki.auto_on_insights` |

*E2E wiki journey deferred to Phase 13.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
