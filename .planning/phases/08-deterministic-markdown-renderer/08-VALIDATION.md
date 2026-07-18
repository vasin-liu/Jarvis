---
phase: 08
slug: deterministic-markdown-renderer
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-18
---

# Phase 08 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (lib unit tests via `#[cfg(test)]`) |
| **Config file** | none — Cargo workspace default |
| **Quick run command** | `cargo test -p insights wiki -- --test-threads=1` |
| **Full suite command** | `cargo test -p insights -- --test-threads=1` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p insights wiki -- --test-threads=1`
- **After every plan wave:** Run `cargo test -p insights -- --test-threads=1`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 08-01-01 | 01 | 1 | WIKI-03 | T-08-01 | Slug never contains `/` `\` or raw name | unit | `cargo test -p insights render_always_emits_source_summary -- --exact` | ❌ W0 | ⬜ pending |
| 08-01-02 | 01 | 1 | WIKI-03 | T-08-02 | Display sanitize for `\|` `]` | unit | `cargo test -p insights render_links_entities_bidirectional -- --exact` | ❌ W0 | ⬜ pending |
| 08-01-03 | 01 | 1 | WIKI-03 | — | Concepts mirror entity link rules | unit | `cargo test -p insights wiki -- --test-threads=1` | ❌ W0 | ⬜ pending |
| 08-01-04 | 01 | 1 | WIKI-03 | T-08-01 | CJK → `e-{6 hex}` slug | unit | `cargo test -p insights cjk_name_uses_hash_slug -- --exact` | ❌ W0 | ⬜ pending |
| 08-01-05 | 01 | 1 | WIKI-03 | T-08-01 | Same-dir collision → `-2` suffix | unit | `cargo test -p insights slug_collision_suffix -- --exact` | ❌ W0 | ⬜ pending |
| 08-01-06 | 01 | 1 | WIKI-03 | — | Index omits empty sections | unit | `cargo test -p insights index_omits_empty_sections -- --exact` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/insights/src/wiki.rs` — stubs / failing `#[cfg(test)]` covering ROADMAP criteria 1–3 + collisions + index omit-empty
- [ ] `crates/insights/Cargo.toml` — add `sha2` / `hex` workspace deps
- [ ] `crates/insights/src/lib.rs` — `mod wiki` + pub re-exports
- [ ] Framework install: none (Cargo already available)

*Existing infrastructure covers the test runner; Wave 0 adds the wiki module test surface.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| No `std::fs` / `Store` / `ChatModel` in `wiki.rs` | WIKI-03 | Static purity check | Grep `wiki.rs` for fs/Store/ChatModel; must be absent |

*All ROADMAP success criteria have automated unit verification planned.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
