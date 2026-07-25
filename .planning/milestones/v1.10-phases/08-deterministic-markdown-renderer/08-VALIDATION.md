---
phase: 08
slug: deterministic-markdown-renderer
status: draft
nyquist_compliant: true
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
| 08-01-01 | 01 | 1 | WIKI-03 | T-08-01 | Module/types wired; no FS/Store/ChatModel | build + grep | `cargo test -p insights --no-run && rg -n "mod wiki\|render_wiki_pages\|WikiPageType" crates/insights/src/lib.rs crates/insights/src/wiki.rs` | ❌ W0 | ⬜ pending |
| 08-01-02 | 01 | 1 | WIKI-03 | T-08-01 | CJK → `e-{6 hex}`; same-dir `-2`; strip `/` `\` | unit | `cargo test -p insights cjk_name_uses_hash_slug slug_collision_suffix slugify_ascii_name unsafe_path_chars_stripped -- --exact --test-threads=1` | ❌ W0 | ⬜ pending |
| 08-01-03 | 01 | 1 | WIKI-03 | T-08-02, T-08-03 | Frontmatter contract + display sanitize | unit | `cargo test -p insights frontmatter_includes_required_keys frontmatter_omits_content_hash wikilink_uses_path_display_form wikilink_sanitizes_display frontmatter_escapes_title_quotes -- --exact --test-threads=1` | ❌ W0 | ⬜ pending |
| 08-02-01 | 02 | 2 | WIKI-03 | T-08-05 | RED: ROADMAP 1–2 + bidirectional link tests exist and fail | unit (RED) | `cargo test -p insights render_always_emits_source_summary render_links_entities_bidirectional render_links_concepts_bidirectional -- --exact --test-threads=1 2>&1 \| rg "test result: FAILED\|passed: 0"` | ❌ W0 | ⬜ pending |
| 08-02-02 | 02 | 2 | WIKI-03 | T-08-01, T-08-05 | Render drafts + bidirectional links + CJK slug | unit | `cargo test -p insights render_always_emits_source_summary render_links_entities_bidirectional render_links_concepts_bidirectional cjk_name_uses_hash_slug -- --exact --test-threads=1` | ❌ W0 | ⬜ pending |
| 08-02-03 | 02 | 2 | WIKI-03 | T-08-08 | Index omits empty sections; full crate green | unit | `cargo test -p insights -- --test-threads=1` | ❌ W0 | ⬜ pending |

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
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
