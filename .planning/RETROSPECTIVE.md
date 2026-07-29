# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.11 — Related-docs + MCP

**Shipped:** 2026-07-29  
**Phases:** 5 (15–19) | **Plans:** 17

### What Was Built
- `related_sources` hybrid overlap API + Library related-docs panel (navigate)
- `jarvis-mcp` stdio MCP with allowlisted `search` / `list_sources`
- Shared `kb_readonly` helpers (agent + MCP anti-drift) + Store WAL
- Phase 19 citation/E2E trust gate (qa / full-ui / wiki / related-docs + cargo mcp)

### What Worked
- Parallel track after Phase 15 (panel ∥ MCP scaffold/tools) with joint Phase 19 gate
- CONTEXT D-06 lock documented in VERIFICATION override — audit stayed `passed`
- Shared `kb_readonly` prevented agent/MCP semantic fork
- Formal `/gsd-audit-milestone` before complete (unlike informal v1.9 close)

### What Was Inefficient
- Plans 17–19 SUMMARY.md missing YAML `requirements-completed` (manual 3-source verify)
- Nyquist VALIDATION flags on 15/16 never flipped to `true` after green gates
- Wiki E2E flake under parallel wdio workers required sequential retry in ship gate
- `npm run test:e2e:local -- --spec` does not forward `--spec` (use `npx wdio` directly)

### Patterns Established
- Read-only MCP binary separate from Windows GUI subsystem app
- Trust freeze phase: no RAG/`RetrieverConfig` knobs unless gate forces surgical fix
- Dual E2E fixtures for related-docs navigate (shared unique token in summaries)

### Key Lessons
1. Run audit before complete; document CONTEXT overrides so REL wording ≠ ship blocker
2. Prefer shared helpers (`kb_readonly`) when two surfaces must match retrieval semantics
3. Flip Nyquist/`requirements-completed` frontmatter in the same closeout wave as VERIFICATION

### Cost Observations
- Model mix: executor/verifier typically sonnet-class; integration checker composer-class
- Notable: 5 phases / 17 plans in ~4 calendar days with yolo auto-chain

---

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
| v1.11 | 5 | 17 | Related-docs + MCP; audit-before-complete; shared kb_readonly |

### Cumulative Quality

| Milestone | Requirements | Audit | Closeout |
|-----------|--------------|-------|----------|
| v1.9 | 22/22 | informal | verified_closeout |
| v1.10 | 9/9 | passed | verified_closeout |
| v1.11 | 10/10 | passed | verified_closeout |

### Top Lessons (Verified Across Milestones)

1. Keep E2E green as the ship gate for user-facing slices
2. Prefer insert-phase closeout over shipping with known audit gaps
3. Default-off nested config prevents upgrade surprises
4. Run `/gsd-audit-milestone` before `/gsd-complete-milestone`
5. Share retrieval helpers across agent/MCP/UI surfaces to prevent drift
