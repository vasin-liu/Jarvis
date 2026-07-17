# Phase 4: Memory Model + View - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-27
**Phase:** 4-Memory Model + View
**Areas discussed:** URI scheme & migration, Strict ID resolution, MemoryView extraction, Command split

---

## URI Scheme & Migration

| Option | Description | Selected |
|--------|-------------|----------|
| memory://{uuid-v4} + migrate old timestamp URIs | New UUID; startup migration for legacy | ✓ |
| New UUID only, keep old URIs | No migration | |
| id equals uri suffix | source.id = UUID | |

**User's choice:** uuid-v4 with startup migration for legacy `memory://{timestamp}`
**Notes:** Aligns with ROADMAP MEM-01/MEM-02

---

## Strict ID Resolution

| Option | Description | Selected |
|--------|-------------|----------|
| One-release warn on fuzzy contains | Exact id/uri/title first; deprecate fuzzy | ✓ |
| Strict immediately | Reject title fragments now | |
| Keep fuzzy indefinitely | Log only, no removal plan | |

**User's choice:** One-release deprecation with warn log before removing fuzzy `contains()` matching

---

## MemoryView Extraction

| Option | Description | Selected |
|--------|-------------|----------|
| Inline edit panel (same as today) | Move JSX to MemoryView, no UX change | ✓ |
| Modal/ drawer edit | Redesign interaction | |

**User's choice:** Keep inline `memory-edit-panel`; full JSX extraction to MemoryView + useMemory

---

## Command Split

| Option | Description | Selected |
|--------|-------------|----------|
| commands/memory.rs for 5 CRUD cmds | list/get/forget/update/add | ✓ |
| learn_from_exchange stays in chat.rs | Chat-triggered learning | ✓ |

**User's choice:** (Claude discretion from roadmap — user selected area without override)

---

## Claude's Discretion

- Migration function placement (`memory::migrate_legacy_uris` vs store init)
- UUID crate version
- Deprecation log mechanism (minimal deps)

## Deferred Ideas

- ARCH-02 dedicated memories table
- Hard removal of fuzzy title fallback (after one release)
