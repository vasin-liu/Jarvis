# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.10 — Wiki Compile Layer

**Shipped:** 2026-07-25  
**Phases:** 8 (07–14) | **Plans:** 18 | **Tasks:** 41

### What Was Built
- Default-off `WikiConfig` + `SourceKind::WikiPage` with Library「笔记页」
- LLM analyze → deterministic Markdown render → persist/index under `wiki://`
- Settings/Library compile + Obsidian zip export
- E2E `wiki.spec.ts` + citation URI trust (WIKI-08/09)
- Pre-ship closeout: soft-skip reindex, preflight WikiDisabled gate, Nyquist 12/13

### What Worked
- Vertical MVP slices (config → render → analyze → persist → UI → export → E2E)
- Inserted Phase 14 after audit `tech_debt` instead of accepting known gaps
- Explicit D-14 deferrals (F01 / dual writers) kept audit path to `passed`
- Fail-closed parse + soft-skip reindex avoided vault corruption / Failed stubs

### What Was Inefficient
- First milestone audit surfaced tech debt that needed an inserted closeout phase
- Nyquist flags on 12/13 lagged VERIFICATION — backfill docs rather than re-running validate-phase
- Dual `index.md` writers left as intentional debt for test helper vs production rebuild

### Patterns Established
- Wiki features gated solely by nested `wiki.enabled` (IPC + UI)
- Soft-skip (`Ok(false)`) for unsupported reindex kinds vs fail-closed `mark_failed`
- Audit closeout phases with no new REQ-IDs (`AUDIT-CLOSEOUT`) when gaps are process/debt

### Key Lessons
1. Run `/gsd-audit-milestone` before `/gsd-complete-milestone`; close actionable debt in an inserted phase when status is `tech_debt`
2. Prefer crate-level gates + unit tests for IPC defense-in-depth (preflight WikiDisabled)
3. Document E2E waivers explicitly when Vitest + Rust cover UX (D-15) to avoid e2e-required.mdc fights

### Cost Observations
- Model mix: executor/verifier typically sonnet-class; orchestrator parent model
- Notable: Phase 14 (3 plans) unblocked `passed` audit after 07–13 feature work

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v1.9 | 6 | 22 | Structural refactor; closeout without formal audit file |
| v1.10 | 8 | 18 | Feature + inserted closeout phase after formal audit |

### Cumulative Quality

| Milestone | Requirements | Audit | Closeout |
|-----------|--------------|-------|----------|
| v1.9 | 22/22 | informal | verified_closeout |
| v1.10 | 9/9 | passed | verified_closeout |

### Top Lessons (Verified Across Milestones)

1. Keep E2E green as the ship gate for user-facing slices
2. Prefer insert-phase closeout over shipping with known audit gaps
3. Default-off nested config prevents upgrade surprises
