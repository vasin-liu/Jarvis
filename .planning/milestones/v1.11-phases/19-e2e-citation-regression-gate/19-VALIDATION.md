---
phase: 19
slug: e2e-citation-regression-gate
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-28
---

# Phase 19 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Nyquist: every plan task has `<automated>` verify; Wave 0 gaps closed by Plan 19-01 (thin full-ui step) — harness otherwise exists.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | WebdriverIO 9 + Mocha + `@wdio/tauri-service`; Rust `cargo test` for MCP |
| **Config file** | `e2e/wdio.conf.ts`; `crates/mcp/tests/*` |
| **Quick run command** | `npx wdio run e2e/wdio.conf.ts --spec e2e/specs/related-docs.spec.ts` (after `npm run build:e2e` + `JARVIS_E2E=1`) |
| **Full suite command** | `npm run test:e2e:local` && `cargo test -p mcp` |
| **Estimated runtime** | Focused 4-spec gate ~10–25 min; full local suite longer; cargo mcp ~1–2 min |

---

## Sampling Rate

- **After every task commit:** Focused verify from the plan (`rg` for docs/spec edits; `cargo test -p mcp` for MCP plan)
- **After Wave 1 merge:** Plan 01 `rg` gates + Plan 02 `cargo test -p mcp` green
- **Before `/gsd-verify-work`:** Plan 03 E2E gate + `19-VERIFICATION.md` evidence
- **Max feedback latency:** Seconds for `rg`/cargo unit; E2E at Wave 2 only
- **Windows focused pattern:** `npm run build:e2e` then `npx wdio run e2e/wdio.conf.ts --spec …` — never `npm run test:e2e:local -- --spec`

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 19-01-01 | 01 | 1 | TRUST-02 | T-19-01 | Thin related panel in primary journey | E2E scaffold | `rg -n "related-docs-panel" e2e/specs/full-ui.spec.ts` | ❌ → Plan 01 | ⬜ pending |
| 19-01-02 | 01 | 1 | TRUST-02 | T-19-02 | Spec map accurate | docs | `rg -n "related-docs.spec.ts" .cursor/rules/e2e-required.mdc` | ✅ | ⬜ pending |
| 19-02-01 | 02 | 1 | TRUST-02 | T-19-03 | MCP allowlist + tools_kb + paths | integration | `cargo test -p mcp` | ✅ | ⬜ pending |
| 19-02-02 | 02 | 1 | TRUST-02 | T-19-04 | Live-tool docs / README pointer | docs | `rg` docs/mcp.md + README; `cargo test -p mcp --test allowlist` | ✅ | ⬜ pending |
| 19-03-01 | 03 | 2 | TRUST-01, TRUST-02 | T-19-06..08 | Citation + related E2E under mocks | E2E | `npx wdio run e2e/wdio.conf.ts --spec qa/full-ui/wiki/related-docs` after `build:e2e` | ✅ specs | ⬜ pending |
| 19-03-02 | 03 | 2 | TRUST-01, TRUST-02 | T-19-09 | Recorded gate evidence | docs | `rg` 19-VERIFICATION.md | ❌ → Plan 03 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `e2e/specs/qa.spec.ts`, `wiki.spec.ts`, `related-docs.spec.ts` — exist (Phases 13/16)
- [x] Dual fixture seed + `JARVIS_E2E=1` harness — exist
- [x] `cargo test -p mcp` allowlist/tools_kb/paths — exist (Phase 18)
- [x] `docs/mcp.md` + README MCP pointer — exist (confirm in Plan 19-02)
- [ ] Thin related-docs step in `e2e/specs/full-ui.spec.ts` — delivered by Plan 19-01 (only code gap)
- [ ] `19-VERIFICATION.md` — delivered by Plan 19-03

*Framework install: none. Wave 0 scaffold plans not required; Plan 19-01 is the gap fill.*

---

## Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | Covered By |
|--------|----------|-----------|-------------------|------------|
| TRUST-01 | qa citation excerpts + Mock answer | E2E | `npx wdio … --spec e2e/specs/qa.spec.ts` | 19-03 |
| TRUST-01 | full-ui citation/trust paths | E2E | `npx wdio … --spec e2e/specs/full-ui.spec.ts` | 19-03 |
| TRUST-01 | wiki non-wiki:// citation URI | E2E | `npx wdio … --spec e2e/specs/wiki.spec.ts` | 19-03 |
| TRUST-01 | No opportunistic RAG default changes | review + E2E | git diff attestation in VERIFICATION | 19-03 |
| TRUST-02 | related-docs panel + navigate | E2E | `npx wdio … --spec e2e/specs/related-docs.spec.ts` | 19-03 |
| TRUST-02 | Related in primary journey | E2E | thin step in full-ui + same full-ui wdio | 19-01, 19-03 |
| TRUST-02 | MCP search/list happy path | integration | `cargo test -p mcp` | 19-02, 19-03 evidence |
| TRUST-02 | docs/map accuracy | docs | Plan 19-01/02 verifies | 19-01, 19-02 |

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| — | — | — | All phase behaviors have automated verification or docs `rg` gates. |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 gaps assigned to Plans 19-01 / 19-03
- [x] No watch-mode flags
- [x] Feedback latency acceptable (unit/docs fast; E2E at phase gate)
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending execution
