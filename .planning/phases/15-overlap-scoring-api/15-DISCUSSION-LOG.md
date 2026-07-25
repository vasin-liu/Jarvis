# Phase 15: Overlap Scoring API - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-25
**Phase:** 15-overlap-scoring-api
**Areas discussed:** Seed query text, Weak affinity / empty results, Return payload, Source kind policy

---

## Seed query text

| Option | Description | Selected |
|--------|-------------|----------|
| Prefer summary, else title | Use summary when present; fallback to title | ✓ |
| Title only | Always use title as seed query | |
| Summary only | Empty when no summary | |
| Title + summary concat | Combine both into one query string | |

**User's choice:** Prefer summary, else title (1)
**Follow-ups:** Blank summary → title (1). Empty title → empty list, no error (2).

---

## Weak affinity / empty results

| Option | Description | Selected |
|--------|-------------|----------|
| top_n=5, empty OK, no threshold | Default 5; []; truncate only | ✓ |
| Require min score | Filter weak RRF hits below threshold | |
| Larger default top_n | e.g. 10 | |
| Error on empty | Fail when no neighbors | |

**User's choice:** Default top_n=5; empty list OK; no score threshold (1)

---

## Return payload

| Option | Description | Selected |
|--------|-------------|----------|
| id + title + kind + snippet | Snippet from top ChunkHit text | ✓ |
| id + title only | Minimal DTO | |
| Include raw score | Expose RRF/ChunkHit.score | |
| Snippet from source summary | Prefer Source.summary over chunk text | |

**User's choice:** source_id + title + kind + snippet from top-ranked chunk text (1)

---

## Source kind policy

| Option | Description | Selected |
|--------|-------------|----------|
| All kinds equal | Wiki/Memory/etc. can be neighbors | ✓ |
| Exclude Wiki/Memory | Only “document-like” kinds | |
| Configurable filter | Caller passes kind allowlist | |

**User's choice:** All kinds equal (1). Wiki seed same rules, not forced empty (1).

---

## Claude's Discretion

- Snippet truncation length / Unicode handling
- Crate placement (`retriever` vs thin Tauri wrapper)
- Internal over-fetch `top_k` before collapsing to unique sources

## Deferred Ideas

- Library panel UX — Phase 16
- MCP — Phases 17–18
- Citation trust — Phase 19
- Link graph / score threshold / exposing scores in UI — out of this phase
