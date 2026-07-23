---
phase: 12
slug: obsidian-zip-export
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-23
---

# Phase 12 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (insights) + Vitest ^3.2.4 + Testing Library |
| **Config file** | workspace Cargo / `vite.config.ts` (`test.include`: `src/**/*.test.ts(x)`) |
| **Quick run command** | `cargo test -p insights export_wiki -- --nocapture` && `npx vitest run src/views/LibraryView.test.tsx src/hooks/useLibrary.test.ts` |
| **Full suite command** | `cargo test -p insights` && `npm test` |
| **Estimated runtime (task-scoped quick)** | **≤ ~30s** — prefer per-task filters below (cargo export filter **or** single Vitest file) |
| **Estimated runtime (full-wave / phase gate)** | **~60s** — full `cargo test -p insights` + `npm test`; exceeds 30s Nyquist warning threshold by design; do not block on wave-full latency |

---

## Sampling Rate

- **After every task commit:** Run that task’s `<automated>` command (task-scoped; stay under ~30s feedback)
- **After every plan wave:** Run full suite command (~60s acceptable for wave merge)
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~30s task-scoped; ~60s wave-full noted (not a task-loop blocker)
- **Phase gate exception:** Do not require `npm run test:e2e:local` for Phase 12 (E2E deferred to Phase 13 per CONTEXT)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 12-01-01 | 01 | 0 | WIKI-07 | T-12-01 | Path-safe zip entries; no Zip Slip | unit | `cargo test -p insights export_wiki` | ❌ W0 | ⬜ pending |
| 12-01-02 | 01 | 1 | WIKI-07 | T-12-01 | Stub inject; skip on-disk `.obsidian`; empty/disabled reject | unit | `cargo test -p insights export_wiki` | ❌ W0 | ⬜ pending |
| 12-02-01 | 02 | 2 | WIKI-07 | T-12-02 | Toolbar hide when disabled; preflight empty blocks save | Vitest | `npx vitest run src/views/LibraryView.test.tsx src/hooks/useLibrary.test.ts` | ❌ W0 | ⬜ pending |
| 12-02-02 | 02 | 2 | WIKI-07 | T-12-02 | Preflight→save→export hook; thin cmds registered | Vitest | `npx vitest run src/hooks/useLibrary.test.ts` | ❌ W0 | ⬜ pending |
| 12-02-03 | 02 | 2 | WIKI-07 | T-12-02 | Toolbar + busy + soft wiki-export-done notice | Vitest | `npx vitest run src/views/LibraryView.test.tsx src/hooks/useLibrary.test.ts` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/insights` unit tests for `export_wiki_zip` / path safety / empty / stub inject / no disk stub
- [ ] Promote or duplicate `any_content_md` → `wiki_has_exportable_notes`
- [ ] Add workspace + insights `zip` dependency (`7.2.0`, deflate, MSRV 1.85-safe)
- [ ] Extend `LibraryView.test.tsx` for `wiki-export` visibility/placement
- [ ] Extend `useLibrary.test.ts` for preflight → save → export / cancel / empty copy
- [ ] Register Tauri commands + `src/lib/tauri.ts` wrappers (implementation wave; tests mock invoke)

*E2E `e2e/specs/wiki.spec.ts` is Phase 13 — not a Phase 12 Wave 0 gap.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Zip opens as Obsidian vault | WIKI-07 | Obsidian desktop not in CI | Export zip → open folder as vault in Obsidian; confirm pages + no editor legacy prompt |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Task-scoped feedback ≤ ~30s; wave-full ~60s documented (not a task-loop blocker)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
