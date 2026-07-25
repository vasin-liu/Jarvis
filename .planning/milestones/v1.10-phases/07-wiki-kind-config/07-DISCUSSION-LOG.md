# Phase 07: Wiki kind + config - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-17
**Phase:** 07-Wiki kind + config
**Areas discussed:** Library label, config.json shape, auto_on_insights timing, unknown kind / old UI

---

## Library label

| Option | Description | Selected |
|--------|-------------|----------|
| 笔记页 | Chinese label matching existing kinds | ✓ |
| Wiki | English product term | |
| 知识笔记 | Longer product phrasing | |

**User's choice:** 笔记页 for all related Qs (no bilingual, no chip, Vitest exact match)

---

## config.json shape

| Option | Description | Selected |
|--------|-------------|----------|
| Nested `"wiki": {…}` | Non-flatten WikiConfig | ✓ |
| Top-level wiki_enabled | Flatten-style fields | |

**User's choice:** Nested object; missing key → default; ignore unknown fields; dual round-trip tests

---

## auto_on_insights timing

| Option | Description | Selected |
|--------|-------------|----------|
| Include field now (dormant) | Shape stable early | ✓ |
| enabled-only in Phase 07 | Defer field | |

**User's choice:** Ship field default false; no wire this milestone; no Settings in Phase 07; test default + explicit true

---

## unknown kind / old UI

| Option | Description | Selected |
|--------|-------------|----------|
| Old FE shows wiki_page OK | Existing default branch | ✓ |
| Version gate / migration prompt | Out of phase scope | |

**User's choice:** parse→None; fix exhaustive matches; keep FE default `return kind`

---

## Claude's Discretion

- Test file layout and whether WikiConfig stays in `types.rs`

## Deferred Ideas

- Settings UI (Phase 11); auto_on_insights wiring (Future); chip/icon (later UI)
