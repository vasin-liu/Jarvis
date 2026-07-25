---
phase: 13
slug: e2e-citation-trust
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-24
---

# Phase 13 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

**Nyquist backfill (Phase 14):** Marked compliant from existing `13-VERIFICATION.md` (`status: passed`, 8/8 must-haves) and green automated tests (`wiki.spec.ts`, `full-ui`/`qa`, Vitest export-path harness). Full `/gsd-validate-phase` re-run not required (D-10..D-12).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | WebdriverIO 9 + Mocha (`@wdio/mocha-framework`); Vitest 3 (optional FE units) |
| **Config file** | `e2e/wdio.conf.ts`; Vitest via `vite.config.ts` |
| **Quick run command** | `cross-env JARVIS_E2E=1 JARVIS_E2E_FIXTURE=./e2e/fixtures/sample.md npx wdio run e2e/wdio.conf.ts --spec e2e/specs/wiki.spec.ts` |
| **Full suite command** | `npm run test:e2e:local` |
| **Estimated runtime** | ~120–300 seconds (full local E2E); wiki-only faster after `build:e2e` |

---

## Sampling Rate

- **After every task commit:** Run focused `--spec e2e/specs/wiki.spec.ts` when FE/E2E changed; Vitest for touched hook/view
- **After every plan wave:** Run `npm run test:e2e:local` (or at least `wiki` + `qa` + `full-ui`)
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~300 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 13-01-01 | 01 | 1 | WIKI-09 | T-13-01 | forced-path Vitest RED | unit | `npx vitest run src/hooks/useLibrary.test.ts` | ✅ gap | ⬜ pending |
| 13-01-02 | 01 | 1 | WIKI-08 / WIKI-09 | T-13-01/03 | export bypass + `data-source-uri` + `settings-save-config` | unit + rg | `npx vitest run src/hooks/useLibrary.test.ts` && `rg -n "data-source-uri" src/views/ChatView.tsx` && `rg -n "settings-save-config" src/views/SettingsView.tsx` | ✅ / ❌ attr | ⬜ pending |
| 13-01-03 | 01 | 1 | WIKI-09 | T-13-02 | helpers export path + zip size | rg (+ optional tsc) | `rg -n "setWikiExportPath\|wikiE2eZipPath\|assertWikiZipNonEmpty" e2e/helpers.ts` | ✅ extend | ⬜ pending |
| 13-02-01 | 02 | 2 | WIKI-08 / WIKI-09 | T-13-05/06 | wiki journey + citation URI trust | E2E | `npm run build:e2e && cross-env JARVIS_E2E=1 JARVIS_E2E_FIXTURE=./e2e/fixtures/sample.md npx wdio run e2e/wdio.conf.ts --spec e2e/specs/wiki.spec.ts` | ❌ W0 | ⬜ pending |
| 13-02-02 | 02 | 2 | WIKI-09 | T-13-07 | full-ui Library default-off + docs map | E2E + rg | `rg -n "wiki.spec.ts" .cursor/rules/e2e-required.mdc e2e/README.md && rg -n "wiki-export" e2e/specs/full-ui.spec.ts && cross-env JARVIS_E2E=1 JARVIS_E2E_FIXTURE=./e2e/fixtures/sample.md npx wdio run e2e/wdio.conf.ts --spec e2e/specs/full-ui.spec.ts --spec e2e/specs/qa.spec.ts` | ✅ extend | ⬜ pending |
| 13-02-03 | 02 | 2 | WIKI-08 / WIKI-09 | — | phase gate | E2E | `npm run test:e2e:local` | ✅ suite | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `e2e/specs/wiki.spec.ts` — WIKI-08 + WIKI-09 journey (does not exist)
- [ ] `e2e/helpers.ts` — `setWikiExportPath` / zip size assert helpers (or inline)
- [ ] `src/hooks/useLibrary.ts` — honor `__JARVIS_E2E_WIKI_EXPORT_PATH__`
- [ ] `src/views/ChatView.tsx` — `data-source-uri` on citation buttons
- [ ] `e2e/specs/full-ui.spec.ts` — light Library default-off asserts
- [ ] `.cursor/rules/e2e-required.mdc` + `e2e/README.md` — Wiki → `wiki.spec.ts` row
- [ ] (Recommended) Vitest: forced export path skips `save`
- [ ] (Recommended) `data-testid="settings-save-config"` on Settings save

*Framework install: none — existing infrastructure covers all phase requirements once Wave 0 artifacts exist.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Spec map docs updated | WIKI-09 / D-13 | Docs not run by WDIO | `rg "wiki.spec" .cursor/rules/e2e-required.mdc e2e/README.md` |

*All product behaviors have automated verification via E2E/Vitest once Wave 0 is complete.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 300s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
