# Phase 19: E2E + citation regression gate - Research

**Researched:** 2026-07-28
**Domain:** Ship-gate verification — WebDriver citation/related-docs regression + offline MCP cargo suite + docs/spec-map drift check
**Confidence:** HIGH (harness, scripts, prior Phase 16/18 evidence verified in-repo)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Trust freeze (TRUST-01)
- **D-01:** Do **not** change default `rag::ask` / citation fusion / `RetrieverConfig::default` / related scoring knobs in this phase **unless** a failing gate test proves a regression — then fix surgically with a regression test, no opportunistic refactors.
- **D-02:** Citation gate suite (must stay green under `JARVIS_E2E=1`): **`qa.spec.ts`**, **`full-ui.spec.ts`** (citation/trust paths), and **`wiki.spec.ts`** citation trust journey. Treat failures as ship blockers.

#### Related-docs coverage (TRUST-02)
- **D-03:** Keep and require green **`e2e/specs/related-docs.spec.ts`** (panel visibility + navigate) with existing dual-fixture seed; no live LLM.
- **D-04:** Extend **`full-ui.spec.ts`** with a **short** related-docs step (select indexed source → panel → optional navigate) so the primary journey covers Library related discovery — matches e2e-required “extend full-ui if part of primary journey.” Do not duplicate the entire focused spec.

#### MCP offline happy path (TRUST-02)
- **D-05:** MCP verification remains **`cargo test -p mcp`** (plus `retriever`/`agent` if shared helpers touched). **No WebDriver / stdio-host E2E** for MCP in v1.11.
- **D-06:** Confirm `docs/mcp.md` + README MCP pointer still match Phase 18 live tools (no stub wording drift). Docs-only edits allowed; no Settings UI.

#### Spec map & harness
- **D-07:** Confirm `.cursor/rules/e2e-required.mdc` Related-docs row stays accurate; update only if paths/names changed.
- **D-08:** Ship gate commands: `npm run test:e2e:local` covering related-docs + citation specs (or equivalent focused wdio runs after `build:e2e`), plus cargo MCP suite. Document in SUMMARY/VERIFICATION what ran.

#### UI / product
- **D-09:** **No new Library UI chrome** this phase (UI hint on ROADMAP means E2E/UX verification of existing panel, not redesign). Vitest updates only if a gate fix requires them.

### Claude's Discretion
- Exact full-ui related-docs assertion depth (minimum: panel displayed after select)
- Whether to run `npm run test:e2e:local` (all specs) vs explicit `--spec` list for faster iteration — final VERIFICATION must still prove D-02/D-03 suites green
- Windows path / `data-testid` click helpers: reuse Phase 16 patterns; fix flakiness surgically

### Deferred Ideas (OUT OF SCOPE)
- MCP stdio WebDriver / Cursor live-client smoke
- REL-F01 WikiPage soft-exclude in related-docs
- MCP-F01/F02/F03
- Hard WikiPage RAG citation filter
- Cross-corpus entity merge
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TRUST-01 | Related-docs and MCP do not change default RAG/citation behavior; existing qa / citation E2E stays green | Trust freeze D-01; re-run `qa` / `full-ui` citation paths / `wiki` URI trust; treat red as ship blocker; no opportunistic `RetrieverConfig::default` / `rag::ask` edits |
| TRUST-02 | Automated coverage includes related-docs panel visibility + navigate and MCP tool happy paths under `JARVIS_E2E=1` mocks (no live LLM) | Keep `related-docs.spec.ts` green; thin `full-ui` related step; `cargo test -p mcp` (allowlist + tools_kb + paths); docs/mcp.md + e2e-required map drift check |
</phase_requirements>

## Summary

Phase 19 is the **v1.11 ship gate**, not a feature phase. Related-docs (15–16) and MCP (17–18) already shipped product behavior. This phase proves they did **not** erode Core Value citation trust and that automated coverage still matches the e2e-required map + MCP docs.

Concrete work is thin: (1) re-run citation E2E under `JARVIS_E2E=1` mocks, (2) keep focused `related-docs.spec.ts` green, (3) add a **short** related-docs assertion to `full-ui.spec.ts` (panel after select; optional navigate), (4) re-run `cargo test -p mcp`, (5) confirm `docs/mcp.md` / README / e2e-required Related-docs row have no stub drift. Only touch RAG/retriever defaults if a gate failure proves a regression — then surgical fix + regression test.

**Primary recommendation:** Plan 1–2 waves: Wave 0/1 = focused wdio gate after `build:e2e` + cargo MCP + docs/map check; Wave 2 = thin full-ui related-docs step + record exact commands in VERIFICATION. Prefer `npx wdio run … --spec` for iteration; use full `npm run test:e2e:local` (or CI-equivalent) as the final ship bar.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Citation trust regression | Browser / Client (WebDriver) | API / Backend (rag + mock providers) | TRUST-01 proved by qa/full-ui/wiki E2E, not by changing fusion |
| Related-docs focused journey | Browser / Client | E2E seed in `e2e.rs` | Already GREEN Phase 16; gate re-runs only |
| Primary-journey related coverage | Browser / Client (`full-ui.spec.ts`) | — | D-04 thin step; no new UI chrome (D-09) |
| MCP happy path | API / Backend (`cargo test -p mcp`) | — | D-05; stdio MCP is not WebDriver |
| Trust freeze / no RAG fork | API / Backend (`rag`, `retriever`) | — | Diff review + failing gate only triggers surgical fix |
| Docs / spec-map accuracy | CDN / Static (docs + `.cursor/rules`) | — | Drift check; docs-only edits OK |
| E2E harness env | Desktop shell + drivers | — | `JARVIS_E2E=1`, release binary, tauri-driver, msedgedriver |

## Standard Stack

### Core

| Library / Tool | Version | Purpose | Why Standard |
|----------------|---------|---------|--------------|
| WebdriverIO | `^9.20.0` | E2E runner | Existing harness [VERIFIED: package.json] |
| `@wdio/mocha-framework` | `^9.20.0` | Mocha BDD specs | Existing [VERIFIED: package.json] |
| `@wdio/tauri-service` | `^1.1.0` | Tauri capability attach | Existing [VERIFIED: package.json] |
| `tauri-driver` + `msedgedriver` | installed locally / CI | Native WebDriver bridge | `e2e/wdio.conf.ts` [VERIFIED: e2e/wdio.conf.ts] |
| `cross-env` | package.json | Set `JARVIS_E2E*` on Windows | Used by `test:e2e:local` [VERIFIED: package.json] |
| Mock providers via `JARVIS_E2E=1` | app E2E mode | Deterministic Q&A / embeds | `src-tauri/src/e2e.rs` [VERIFIED: e2e.rs] |
| `crates/mcp` cargo tests | workspace | Offline MCP allowlist + tools_kb | [VERIFIED: crates/mcp/tests/*] |

### Supporting

| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| `npm run build:e2e` (`tauri build --no-bundle`) | tauri-cli workspace | Produce `target/release/tauri-app.exe` | **Always** before wdio if binary stale |
| Vitest | `^3.x` | Only if a gate fix needs FE unit coverage | D-09 — not a primary deliverable |
| `cargo test -p retriever` / `-p agent` | — | Only if shared helpers touched during surgical fix | D-05 optional extension |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Full `npm run test:e2e:local` | Focused `npx wdio … --spec` list | Faster iteration; final VERIFICATION must still prove D-02/D-03 (discretion) |
| WebDriver MCP stdio smoke | `cargo test -p mcp` | Locked out — D-05 / Pitfall 4 |
| Redesign related panel for “better UX” | Keep Phase 16 UI | Violates D-09; out of gate scope |
| Change `RetrieverConfig::default` for “MCP quality” | Shared `kb_readonly` already uses defaults | Violates D-01 / Pitfall 5 |

**Installation:**

```bash
# No new npm or crates packages for Phase 19.
# Prerequisite tools (already project-standard):
#   cargo install tauri-driver
#   msedgedriver.exe at repo root (or MSEDGEDRIVER_PATH)
```

**Version verification:** WebdriverIO `^9.20.0`, Node 22 (CI) / local Node `v24.5.0` observed this session — no dep bumps planned [VERIFIED: package.json + `node --version`].

## Package Legitimacy Audit

> Phase installs **no** new external packages.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| — | — | — | — | — | N/A | No installs |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** none

## Architecture Patterns

### System Architecture Diagram

```text
[Planner / executor]
        |
        v
 +------------------+     build:e2e      +---------------------------+
 | Gate checklist   | -----------------> | target/release/tauri-app  |
 | TRUST-01/02     |                    | (JARVIS_E2E=1 + fixtures) |
 +------------------+                    +-------------+-------------+
        |                                              |
        | focused / full wdio                          | spawn via tauri-driver
        v                                              v
 +------------------+                    +---------------------------+
 | Specs:           | <--- WebDriver --->| React UI (Library/Chat)   |
 | qa, full-ui,     |                    | related-docs panel        |
 | wiki, related-   |                    | citation-excerpt / URI     |
 | docs             |                    +---------------------------+
 +------------------+
        |
        | parallel offline
        v
 +------------------+
 | cargo test -p mcp|
 | allowlist        |
 | tools_kb         |
 | paths            |
 +------------------+
        |
        v
 +------------------+
 | docs/mcp.md +    |
 | README +         |
 | e2e-required.mdc |
 +------------------+
```

### Recommended Project Structure

```text
e2e/
├── wdio.conf.ts              # JARVIS_E2E defaults; specs glob; tauri-driver
├── helpers.ts                # openNav, askQuestion, clickViaDom, setReact*
├── fixtures/
│   ├── sample.md             # primary seed (xyzzy-plugh)
│   └── related-neighbor.md   # dual seed for related-docs
└── specs/
    ├── related-docs.spec.ts  # focused panel + navigate (keep green)
    ├── qa.spec.ts            # citation excerpt regression
    ├── wiki.spec.ts          # citation URI trust (non-wiki://)
    └── full-ui.spec.ts       # primary journey + NEW short related step

crates/mcp/tests/
├── allowlist.rs              # tools/list === {search, list_sources}
├── tools_kb.rs               # search/list happy paths
└── paths.rs                  # --db / JARVIS_DATA_DIR

docs/mcp.md                   # live tools docs (confirm no stub drift)
.cursor/rules/e2e-required.mdc
```

### Pattern 1: Focused wdio after build:e2e (Windows PowerShell)

**What:** Rebuild release app once, then filter specs with WDIO `--spec` (array).  
**When to use:** Iteration during Phase 19; Phase 16 verification used this exact pattern.  
**Example:**

```powershell
# Source: Phase 16 VERIFICATION + 16-04-SUMMARY [VERIFIED]
npm run build:e2e
$env:JARVIS_E2E = "1"
$env:JARVIS_E2E_FIXTURE = "./e2e/fixtures/sample.md"
npx wdio run e2e/wdio.conf.ts --spec e2e/specs/related-docs.spec.ts
npx wdio run e2e/wdio.conf.ts --spec e2e/specs/qa.spec.ts
npx wdio run e2e/wdio.conf.ts --spec e2e/specs/wiki.spec.ts
npx wdio run e2e/wdio.conf.ts --spec e2e/specs/full-ui.spec.ts
```

Or one multi-spec invocation:

```powershell
npx wdio run e2e/wdio.conf.ts `
  --spec e2e/specs/related-docs.spec.ts `
  --spec e2e/specs/qa.spec.ts `
  --spec e2e/specs/wiki.spec.ts `
  --spec e2e/specs/full-ui.spec.ts
```

`wdio.conf.ts` also defaults `JARVIS_E2E` to `"1"` in `onPrepare` / `beforeSession`, but **always set env explicitly** to match CI/`test:e2e:local` [VERIFIED: e2e/wdio.conf.ts].

### Pattern 2: Full local ship bar

**What:** `test:e2e:local` = `build:e2e` + `cross-env JARVIS_E2E=1 JARVIS_E2E_FIXTURE=./e2e/fixtures/sample.md` + all `specs/**/*.spec.ts`.  
**When to use:** Final VERIFICATION / CI parity (D-08).

```powershell
npm run test:e2e:local
```

CI already splits build + test with env set in workflow [VERIFIED: .github/workflows/ci.yml].

### Pattern 3: Thin full-ui related-docs step

**What:** After Library stats (or after an indexed select), assert `related-docs-panel` displayed; optionally click first `related-docs-row-*` and check selection — **do not** copy the whole focused spec (deselect D-08 path, dual waitUntil blocks, etc.).  
**When to use:** D-04 / e2e-required “extend full-ui if part of primary journey.”  
**Reuse:** Phase 16 `browser.execute` click helpers (Windows `\\?\` source ids break CSS attribute selectors) [VERIFIED: e2e/specs/related-docs.spec.ts].

### Pattern 4: MCP cargo gate

**What:** Offline happy path for TRUST-02 MCP half.  
**Commands:**

```powershell
cargo test -p mcp
# Optional if surgical shared-helper fix:
cargo test -p retriever
cargo test -p agent
```

Known integration tests [VERIFIED: `cargo test -p mcp -- --list`]:

| File | Tests |
|------|--------|
| `tests/allowlist.rs` | `tool_allowlist_is_exactly_search_and_list_sources` |
| `tests/tools_kb.rs` | `search_returns_results_json`, `search_empty_query_is_error`, `search_limit_clamped`, `list_sources_indexed_only_no_uri` |
| `tests/paths.rs` | 5 path/resolution tests |
| lib unit | `parse_db_flag_*` |

### Anti-Patterns to Avoid

- **`npm run test:e2e:local -- --spec …`:** Does **not** reliably forward `--spec` through the nested `build:e2e && … npm run test:e2e` script. Phase 16 already burned here — use `npx wdio run e2e/wdio.conf.ts --spec …` after `build:e2e` [VERIFIED: 16-04-SUMMARY.md].
- **`npm run test:e2e:local -- --spec` expecting skip of rebuild:** Script always runs `build:e2e` first — expensive; focused runs should separate build once.
- **Skipping qa/full-ui/wiki because related-docs passed:** Pitfall 5 — citation gate is mandatory.
- **WebDriver for MCP:** Locked out (D-05).
- **Changing RAG defaults “while here”:** Locked out (D-01) unless gate proves regression.
- **New Library chrome / Vitest drive-by:** Locked out (D-09).
- **Live LLM / live Cursor MCP client in CI:** Pitfall 4 — never.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Spec filtering | Custom npm wrapper | `npx wdio run … --spec` | WDIO native; npm compound scripts drop args [VERIFIED: 16-04-SUMMARY] |
| Related click on Windows paths | CSS `[data-testid="source-row-${id}"]` | `browser.execute` exact testid / index helpers | Path ids break attribute selectors [VERIFIED: related-docs.spec.ts] |
| MCP protocol E2E | WebDriver stdio host | `cargo test -p mcp` | D-05; Pitfall 4 |
| Citation trust check | Manual screenshot | `qa` excerpt + `wiki` `data-source-uri` assertions | Existing Phase 13 journeys |
| Dual fixture seeding | New seed pipeline | Existing `seed_e2e_fixture` + `related-neighbor.md` | Already Phase 16 [VERIFIED: e2e.rs] |
| MCP docs rewrite | New host guide | Diff-check `docs/mcp.md` vs live tools | Phase 18 already documented live search/list |

**Key insight:** Gate quality is **command correctness + evidence recording**, not new product code.

## Common Pitfalls

### Pitfall 1: Citation / RAG trust regression (research Pitfall 5)

**What goes wrong:** Related-docs or MCP ship while qa/full-ui/wiki go red or are skipped.  
**Why it happens:** Overlap/MCP “quality” knobs touch `RetrieverConfig::default` / `rag::ask`; duplicate retrieve forks; related injects into ask context.  
**How to avoid:** D-01 trust freeze; D-02 mandatory suites; treat failures as ship blockers.  
**Warning signs:** Diffs in `crates/rag` defaults; citation-excerpt count 0; wiki citations only `wiki://`.

### Pitfall 2: `--spec` not applied via `test:e2e:local`

**What goes wrong:** Executor thinks only related-docs ran; actually full suite or wrong args.  
**Why it happens:** `test:e2e:local` nests `npm run test:e2e` after `&&`; extra `--spec` does not forward cleanly [VERIFIED: 16-04-SUMMARY].  
**How to avoid:** Document PowerShell pattern: `build:e2e` then `npx wdio run e2e/wdio.conf.ts --spec …`. For full gate: plain `npm run test:e2e:local` with **no** `--spec`.  
**Warning signs:** WDIO logs show all `specs/**/*.spec.ts` when a filter was intended.

### Pitfall 3: Stale binary / missing `build:e2e`

**What goes wrong:** Wdio launches old `target/release/tauri-app.exe` without dual fixture / panel.  
**Why it happens:** Focused `npx wdio` skips rebuild; binary missing or stale.  
**How to avoid:** Run `npm run build:e2e` whenever UI/Rust E2E seed changed; `wdio.conf.ts` defaults `JARVIS_E2E_APP` to that path [VERIFIED: e2e/wdio.conf.ts].  
**Warning signs:** `library-stats` never reaches ≥2 indexed; related panel missing.

### Pitfall 4: Live LLM / missing `JARVIS_E2E=1`

**What goes wrong:** Flaky CI, FastEmbed download, non-Mock answers.  
**Why it happens:** Env not set; forgetting mocks are mandatory.  
**How to avoid:** `cross-env` in `test:e2e:local`; explicit `$env:JARVIS_E2E="1"` for focused runs; conf also forces env [VERIFIED: package.json + wdio.conf.ts].  
**Warning signs:** Spec waits on real streaming; answers lack `"Mock"`.

### Pitfall 5: Windows source-id selector flake

**What goes wrong:** Related/full-ui steps fail on `\\?\` filesystem URIs.  
**Why it happens:** CSS attribute selectors cannot safely embed raw Windows path ids.  
**How to avoid:** Reuse Phase 16 `clickSourceRowByIndex` / exact `data-testid` string match in `browser.execute` (discretion).  
**Warning signs:** `source-row index 0 missing` or timeout selecting neighbor.

### Pitfall 6: Skipping MCP cargo / docs drift

**What goes wrong:** TRUST-02 half incomplete; README still says stub.  
**Why it happens:** “E2E green = done”; Phase 18 docs already updated but wording can drift.  
**How to avoid:** D-05 `cargo test -p mcp`; D-06 confirm `docs/mcp.md` Status = live tools + README `**MCP (stdio):** docs/mcp.md` [VERIFIED: docs/mcp.md, README.md].  
**Warning signs:** docs say “stub only”; allowlist test missing from VERIFICATION.

### Pitfall 7: Over-building full-ui related step

**What goes wrong:** Duplicates focused spec → long flaky primary journey.  
**Why it happens:** Copy-paste from `related-docs.spec.ts`.  
**How to avoid:** D-04 minimum = panel displayed after select; optional single navigate.  
**Warning signs:** full-ui gains deselect/toggle coverage that belongs only in focused spec.

### Pitfall 8: “Looks done” without recording commands

**What goes wrong:** Phase marked complete; VERIFICATION lacks exact exit evidence.  
**Why it happens:** Gate phases feel administrative.  
**How to avoid:** D-08 — document exact wdio/cargo invocations and results in 19-VERIFICATION / SUMMARY (mirror Phase 16 VERIFICATION table).

## Code Examples

### Exact ship-gate command block (PowerShell)

```powershell
# From repo root: D:\Work\99_Code\02_Rust\Jarvis
# 1) MCP offline (TRUST-02)
cargo test -p mcp

# 2) Build E2E binary (prerequisite for any wdio run)
npm run build:e2e

# 3a) Full suite (final bar — D-08)
$env:JARVIS_E2E = "1"
$env:JARVIS_E2E_FIXTURE = "./e2e/fixtures/sample.md"
# Prefer official script (sets cross-env + rebuild):
npm run test:e2e:local

# 3b) Focused iteration (DO NOT use npm run test:e2e:local -- --spec)
$env:JARVIS_E2E = "1"
$env:JARVIS_E2E_FIXTURE = "./e2e/fixtures/sample.md"
npx wdio run e2e/wdio.conf.ts --spec e2e/specs/qa.spec.ts
npx wdio run e2e/wdio.conf.ts --spec e2e/specs/full-ui.spec.ts
npx wdio run e2e/wdio.conf.ts --spec e2e/specs/wiki.spec.ts
npx wdio run e2e/wdio.conf.ts --spec e2e/specs/related-docs.spec.ts
```

### Citation assertions already in tree

```typescript
// qa.spec.ts — citation excerpt regression [VERIFIED]
const excerpts = await $$('[data-testid="citation-excerpt"]');
expect(excerpts.length).toBeGreaterThan(0);

// wiki.spec.ts — URI trust: at least one non-wiki:// citation [VERIFIED]
const uris = await browser.execute(() =>
  [...document.querySelectorAll("[data-source-uri]")].map((e) =>
    e.getAttribute("data-source-uri"),
  ),
);
expect(uris.some((u) => !!u && !u.startsWith("wiki://"))).toBe(true);
```

### Thin full-ui related sketch (planner guidance)

```typescript
// Suggested insertion near "shows library stats after seeding"
// Minimum depth (discretion): panel displayed after select
await openNav("library", '[data-testid="library-stats"]');
// wait for >=2 indexed if needed (dual fixture)
await browser.execute(() => {
  const row = document.querySelector('[data-testid^="source-row-"]') as HTMLElement | null;
  if (!row) throw new Error("no source-row");
  row.click();
});
await expect($('[data-testid="related-docs-panel"]')).toBeDisplayed();
```

### Docs drift checklist

| Check | Expected (Phase 18) | Action if wrong |
|-------|---------------------|-----------------|
| `docs/mcp.md` Status | Live hybrid `search` + Indexed `list_sources` via `kb_readonly` | Docs-only edit |
| Tools allowlist | Exactly `{search, list_sources}` | Fix docs or re-open Phase 18 bug |
| Caps | `final_k=8`, excerpt 200 chars, list cap 200 | Align docs |
| README | `**MCP (stdio):** docs/mcp.md` | Restore pointer |
| e2e-required map | Related-docs → `related-docs.spec.ts` | Update only if renamed |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Skip citation E2E after side features | Mandatory Phase 19 qa/full-ui/wiki gate | v1.11 research | Protects Core Value |
| Plan wrote `npm run test:e2e:local -- --spec` | `npx wdio … --spec` after `build:e2e` | Phase 16-04 | Reliable filtered runs on Windows |
| MCP “verify in IDE” | `cargo test -p mcp` offline | Phases 17–18 | Deterministic CI |
| Single E2E fixture | Dual seed (`sample` + `related-neighbor`) | Phase 16 | Related navigate possible |

**Deprecated/outdated:**
- Treating `readOnlyHint` as MCP security — still true; Phase 19 only re-verifies allowlist via cargo.
- Stub wording in MCP docs — Phase 18 removed; Phase 19 confirms no regression.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | *(none material)* | — | All gate commands and harness facts verified from package.json, wdio.conf, Phase 16 SUMMARY/VERIFICATION, mcp tests list, docs |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

## Open Questions

1. **Final bar: full suite vs focused four specs?**
   - What we know: Discretion allows either; D-08 requires evidence D-02/D-03 green.
   - What's unclear: Wall-clock preference on this machine.
   - Recommendation: Iterate with `--spec`; close phase with `npm run test:e2e:local` once (CI parity) **or** multi-`--spec` of the four gate files if full suite is too slow — record which in VERIFICATION.

2. **full-ui related navigate depth?**
   - What we know: Minimum = panel displayed after select.
   - What's unclear: Whether one navigate click is worth flake risk.
   - Recommendation: Panel display required; add navigate only if helpers stay stable (reuse Phase 16 execute clicks).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | wdio / npm scripts | ✓ | v24.5.0 (local); CI Node 22 | — |
| Rust / cargo | `cargo test -p mcp` | ✓ | cargo 1.97.0 | — |
| `tauri-driver` | E2E | ✓ | `~/.cargo/bin/tauri-driver.exe` | Install via `cargo install tauri-driver` |
| `msedgedriver.exe` | E2E | ✓ | repo root | Set `MSEDGEDRIVER_PATH` |
| `target/release/tauri-app.exe` | E2E | ✓ (present this session) | — | `npm run build:e2e` |
| Live LLM / Ollama | — | N/A | — | **Must not use** — mocks only |
| Cursor/Claude MCP client | — | N/A | — | **Out of scope** — cargo only |

**Missing dependencies with no fallback:** none observed on research machine.

**Missing dependencies with fallback:** none.

## Validation Architecture

> `workflow.nyquist_validation` is **true** in `.planning/config.json` — section required.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | WebdriverIO 9 + Mocha + `@wdio/tauri-service`; Rust `cargo test` for MCP |
| Config file | `e2e/wdio.conf.ts`; mcp crate tests under `crates/mcp/tests/` |
| Quick run command | `npx wdio run e2e/wdio.conf.ts --spec e2e/specs/related-docs.spec.ts` (after `build:e2e` + env) |
| Full suite command | `npm run test:e2e:local` && `cargo test -p mcp` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TRUST-01 | Q&A returns Mock answer from fixture | E2E | `npx wdio run e2e/wdio.conf.ts --spec e2e/specs/qa.spec.ts` | ✅ |
| TRUST-01 | Citation excerpts render (char-boundary regression) | E2E | same qa.spec | ✅ |
| TRUST-01 | full-ui seeded Q&A / library trust paths | E2E | `--spec e2e/specs/full-ui.spec.ts` | ✅ (extend related step ❌ Wave 0) |
| TRUST-01 | Wiki citation URI not only `wiki://` | E2E | `--spec e2e/specs/wiki.spec.ts` | ✅ |
| TRUST-01 | No opportunistic RAG default changes | review + E2E | git diff + suites above | ✅ process |
| TRUST-02 | Related panel + navigate | E2E | `--spec e2e/specs/related-docs.spec.ts` | ✅ |
| TRUST-02 | Related in primary journey | E2E | thin step in `full-ui.spec.ts` | ❌ Wave 0 add |
| TRUST-02 | MCP search/list happy path | integration | `cargo test -p mcp` | ✅ |
| TRUST-02 | MCP allowlist exact | unit | `cargo test -p mcp --test allowlist` | ✅ |
| TRUST-02 | docs/mcp.md + README + e2e-required accurate | manual/docs | file review | ✅ files exist |

### Sampling Rate

- **Per task commit:** focused `npx wdio … --spec` for touched spec **or** `cargo test -p mcp` for docs-only/MCP
- **Per wave merge:** multi-spec citation + related-docs + `cargo test -p mcp`
- **Phase gate:** `npm run test:e2e:local` (or recorded equivalent proving D-02/D-03) **and** `cargo test -p mcp` green; docs/map confirmed

### Wave 0 Gaps

- [ ] `e2e/specs/full-ui.spec.ts` — short related-docs step (D-04) — **only code gap**
- [ ] `19-VERIFICATION.md` — record exact commands/results (created at verify time)
- [ ] None for MCP cargo / related-docs focused / qa / wiki — infrastructure exists

*(No framework install needed.)*

## Security Domain

> Gate phase; no new network surface. Re-confirm prior MCP read-only invariants via cargo.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Local desktop + stdio MCP (client-spawned) |
| V3 Session Management | no | — |
| V4 Access Control | yes (light) | MCP structural allowlist `{search, list_sources}` — re-verify `cargo test -p mcp --test allowlist` |
| V5 Input Validation | yes (light) | Existing empty-query error; E2E does not add new IPC |
| V6 Cryptography | no | — |

### Known Threat Patterns for this gate

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Accidental mutate tools on MCP | Tampering | Allowlist test must stay green |
| Citation trust erosion | Spoofing / repudiation of “from my data” | TRUST-01 E2E gate |
| Live LLM secrets in CI | Information disclosure | `JARVIS_E2E=1` mocks only |
| Docs claiming stub while live / vice versa | Elevation via confusion | D-06 docs drift check |

`security_enforcement`: treat as enabled (config absent false).

## Project Constraints (from .cursor/rules/)

| Rule | Directive for Phase 19 |
|------|------------------------|
| `e2e-required.mdc` | User-facing → specs + `data-testid` + `JARVIS_E2E=1` mocks; Related-docs row already present; extend `full-ui` for primary journey; CI `npm run test:e2e` |
| `tdd-goal-driven.mdc` | Goal + acceptance first; bugfix = regression test first; E2E for user-facing |
| `karpathy-guidelines.mdc` | Surgical changes only; no speculative refactors; gate ≠ feature dump |
| `jarvis-stack.mdc` | No stack churn; Rust stable / Tauri 2 unchanged |
| `frontend-taste.mdc` | D-09 — no new Library chrome; if UI touch forced by flake, keep glass/loading/empty/error intact |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/19-e2e-citation-regression-gate/19-CONTEXT.md` — D-01..D-09 locks
- `.planning/REQUIREMENTS.md` — TRUST-01, TRUST-02
- `.planning/ROADMAP.md` — Phase 19 success criteria
- `.planning/research/SUMMARY.md`, `PITFALLS.md`, `STACK.md` — gate rationale + Pitfall 5
- `.planning/phases/16-related-docs-library-panel/16-VERIFICATION.md` + `16-04-SUMMARY.md` — wdio `--spec` pattern + npm forwarding pitfall
- `.planning/phases/18-mcp-tools-search-list-sources/18-CONTEXT.md` — cargo-only MCP lock
- `package.json` scripts — `build:e2e`, `test:e2e`, `test:e2e:local`
- `e2e/wdio.conf.ts`, `e2e/specs/{qa,full-ui,wiki,related-docs}.spec.ts`, `e2e/helpers.ts`
- `docs/mcp.md`, `README.md` MCP pointer
- `.cursor/rules/e2e-required.mdc`
- `cargo test -p mcp -- --list` — allowlist / tools_kb / paths inventory
- `.github/workflows/ci.yml` — E2E job env + build/test split

### Secondary (MEDIUM confidence)

- WebdriverIO CLI `--spec` help text via `npm run test:e2e -- --help` (array override of config specs)

### Tertiary (LOW confidence)

- Exact wall-clock of full `test:e2e:local` on this machine this session (not re-timed)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — existing harness; no new deps
- Architecture: HIGH — gate = verify + thin full-ui + docs; locked decisions constrain alternatives
- Pitfalls: HIGH — Pitfall 5 + Phase 16 `--spec` failure mode empirically documented

**Research date:** 2026-07-28  
**Valid until:** 2026-08-28 (30 days — harness stable; re-check if wdio major bumps)
