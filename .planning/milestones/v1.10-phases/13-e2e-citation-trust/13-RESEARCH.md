# Phase 13: E2E + citation trust - Research

**Researched:** 2026-07-24
**Domain:** WebdriverIO / tauri-driver E2E harness + citation DOM attributes (WIKI-08, WIKI-09)
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Wiki E2E journey (`wiki.spec.ts`)
- **D-01:** Cover **happy-path + default-off**: assert wiki compile/export controls absent when disabled, then enable and run compile → list → export.
- **D-02:** After compile, identify success by **at least one Library row labeled「笔记页」**.
- **D-03:** **Hybrid enable:** assert default-off first → Settings UI toggle「启用 Wiki 笔记层」→「保存配置」→ Library compile/export (do not seed `wiki.enabled=true` at process start for the positive path).
- **D-04:** Compile the **first eligible** source: first non-`wiki_page` indexed row that exposes a `wiki-compile-{id}` button (do not hard-bind fixture display name).

#### Export under WebDriver
- **D-05:** In E2E, **bypass the native Save dialog**. Prefer a FE hook `__JARVIS_E2E_WIKI_EXPORT_PATH__`: when set, `exportWiki` skips `save()` and calls real `export_wiki_zip(destPath)`.
- **D-06:** Assert **success notice** (`wiki-export-done` or equivalent) **and** that the zip file **exists with size > 0**. Do **not** require unzipping / asserting `index.md` / `.obsidian` contents in E2E (Phase 12 unit coverage owns stub/path safety).
- **D-07:** Fixed dest path: **OS temp + `jarvis-e2e-wiki.zip`** (e.g. `%TEMP%/jarvis-e2e-wiki.zip` on Windows). Spec may unlink beforehand to avoid stale size assertions.

#### Citation trust (WIKI-08)
- **D-08:** Prove citation trust **primarily in E2E** inside `wiki.spec` after wiki is enabled and compiled. Do **not** require a new Rust RAG integration test for this phase (planner may add one optionally; not locked).
- **D-09:** Add **`data-source-uri={c.source_uri}`** (or equivalent) on citation buttons in Chat so E2E can read URIs.
- **D-10:** After asking **`What is xyzzy-plugh?`**, require **at least one citation whose `data-source-uri` does not start with `wiki://`**. Wiki citations may appear alongside; forbid only “all citations are wiki” / sole replacement.
- **D-11:** Leave **`qa.spec.ts` unchanged** (default-off Mock answer + `citation-excerpt` regression stays as-is).

#### Regression placement & docs
- **D-12:** **`wiki.spec`** owns the full positive journey + its opening default-off assert. **`full-ui.spec.ts`** gets **light default-off only** (e.g. Library has no `wiki-export` / no `wiki-compile-*`; no positive enable/compile/export in `full-ui`).
- **D-13:** Update **both** `.cursor/rules/e2e-required.mdc` and `e2e/README.md` spec maps with **Wiki → `wiki.spec.ts`**.

### Claude's Discretion
- Exact helper placement for setting `__JARVIS_E2E_WIKI_EXPORT_PATH__` / reading zip size in `e2e/helpers.ts`.
- Whether Settings default-off assert checks unchecked `wiki-enabled-toggle` vs absence of Library controls only (Library absence is mandatory per D-12).
- Mock chat / `e2e.rs` tweaks if wiki compile JSON collides with other Mock branches — keep deterministic; extend Mock only as needed.
- Optional Vitest for `data-source-uri` rendering — not required if E2E covers it.
- Optional Rust citation integration with wiki indexed — not required (D-08).

### Deferred Ideas (OUT OF SCOPE)
- Empty-export E2E assertion (「还没有可导出的笔记…」) — covered by Phase 12 unit/Vitest; optional later
- Unzip / Obsidian stub contents in E2E — Phase 12 unit ownership
- Mandatory Rust RAG+wiki citation integration test — optional, not locked
- Bidirectional Obsidian sync / bulk compile / `auto_on_insights` UX — post-v1.10 backlog

None else — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WIKI-08 | With wiki off or on, RAG Q&A still cites original sources when relevant — wiki pages are additive, not a replacement for citations | After compile, E2E asks `What is xyzzy-plugh?` and asserts ≥1 citation `data-source-uri` not starting with `wiki://`; leave `qa.spec.ts` alone for default-off regression; Mock wiki JSON lacks `xyzzy-plugh` so FTS still prefers fixture |
| WIKI-09 | E2E journey covers enable → compile → list wiki page → export under `JARVIS_E2E=1` mocks; default-off does not break `full-ui` | New `wiki.spec.ts` (hybrid Settings enable + first `wiki-compile-*` +「笔记页」+ export path hook); light Library default-off in `full-ui`; docs map updates |
</phase_requirements>

## Summary

Phase 13 is an **E2E harness + tiny UI testability** phase — not a new product feature. Product surfaces for wiki (Settings toggle, Library「生成笔记」/「导出 Wiki」, Obsidian zip, `MockChatModel`「笔记编译」branch,「笔记页」label) already shipped in Phases 07–12. What is missing is: (1) `e2e/specs/wiki.spec.ts`, (2) a Save-dialog bypass so WebDriver can export without native OS UI, (3) exposing `source_uri` on citation buttons for URI assertions, (4) light default-off asserts in `full-ui`, and (5) spec-map docs.

E2E already wipes `jarvis-e2e` and starts from `AppConfig::default()` (`wiki.enabled=false`) on each app launch. Specs share **one** Tauri session (`maxInstances: 1`). Alphabetical glob order puts a future `wiki.spec.ts` **last**, so enabling wiki mid-suite does not break earlier `full-ui` / `qa` default-off checks — document this and optionally disable wiki at end of `wiki.spec` for safety. [VERIFIED: `src-tauri/src/state.rs`, `e2e/wdio.conf.ts`]

**Primary recommendation:** Implement TDD as RED `wiki.spec.ts` + light `full-ui` assert → GREEN with `__JARVIS_E2E_WIKI_EXPORT_PATH__` in `useLibrary.exportWiki`, `data-source-uri` on Chat citation buttons, helpers in `e2e/helpers.ts`, then docs — do **not** change `qa.spec.ts` or Mock wiki JSON unless compile/Q&A collide (unlikely today).

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Wiki enable → save → compile → list → export journey | Browser / Client (WDIO E2E) | API (existing wiki cmds) | WIKI-09 is a UI journey under mocks; backend already exists |
| Default-off regression (`full-ui`) | Browser / Client (WDIO) | — | Assert absence of Library wiki controls; no enable path |
| Save-dialog bypass | Browser / Client (`useLibrary`) | OS temp FS | Native `save()` cannot be driven reliably; window hook → real `export_wiki_zip` |
| Zip exists / size > 0 | WDIO Node process (`fs` + `os.tmpdir`) | Tauri FS write | Assert in helper after `wiki-export-done`; no unzip |
| Citation URI trust (WIKI-08) | Browser / Client (`ChatView` attr + E2E) | RAG / Store (existing) | Citations already carry `source_uri`; UI must expose for selectors |
| Mock wiki compile JSON | API / Backend (`MockChatModel`) | — | Already returns valid JSON when system contains「笔记编译」— extend only if collision |
| Spec map docs | Repo docs / rules | — | D-13: `e2e-required.mdc` + `e2e/README.md` |

## Standard Stack

### Core

| Library / Tool | Version | Purpose | Why Standard |
|----------------|---------|---------|--------------|
| WebdriverIO | **^9.20.0** (repo) | E2E runner | Existing harness; CI `e2e` job [VERIFIED: `package.json`] |
| `@wdio/mocha-framework` | **^9.20.0** | Spec style | Matches all `e2e/specs/*.spec.ts` [VERIFIED: `package.json`] |
| `@wdio/tauri-service` | **^1.1.0** | Tauri capability | Existing WDIO config [VERIFIED: `package.json`] |
| tauri-driver + msedgedriver | installed locally | Drive packaged app | Existing `e2e/wdio.conf.ts` [VERIFIED: env probe — both present] |
| Vitest | **^3.2.4** | Optional FE unit for export hook / attr | Discretion only; E2E is the gate [VERIFIED: `package.json`] |

### Supporting

| Library / Tool | Version | Purpose | When to Use |
|----------------|---------|---------|-------------|
| `node:fs` / `node:os` / `node:path` | Node 22+ (local 24.5) | Unlink + `stat` zip under `%TEMP%` | Export size assertion (D-06/D-07) — in WDIO process, not browser |
| Existing `e2e/helpers.ts` | — | `openNav`, `expandSettingsSection`, `setReactCheckbox`, `askQuestion`, `clickViaDom` | Compose wiki journey; add export-path + zip helpers here (Discretion) |
| `JARVIS_E2E=1` + `JARVIS_E2E_FIXTURE` | — | Mock providers + seed `sample.md` | Already set by `test:e2e:local` / wdio `onPrepare` |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `__JARVIS_E2E_WIKI_EXPORT_PATH__` FE hook | Auto-approve Save dialog via OS automation | Fragile on Windows CI; locked out by D-05 |
| E2E citation URI assert | New Rust RAG+wiki integration test only | D-08 locks E2E as primary; Rust optional only |
| Change `qa.spec.ts` for URI assert | Leave unchanged + assert in `wiki.spec` | D-11 forbids qa changes |

**Installation:** None — no new npm/crates packages for this phase.

**Package Legitimacy Audit:** N/A — zero new dependencies. [VERIFIED: phase scope]

## Architecture Patterns

### System Architecture Diagram

```text
WDIO (Node)                         Tauri app (JARVIS_E2E=1)
───────────                         ─────────────────────────
wiki.spec                           AppConfig.default → wiki.enabled=false
  │                                 MockEmbedder + MockChatModel
  ├─ openNav(library) ────────────► Library: no wiki-export / wiki-compile-*
  ├─ Settings: expand wiki ───────► wiki-enabled-toggle → 保存配置
  │                                 set_config → nested config updates Library
  ├─ click first wiki-compile-* ──► compile_wiki_cmd → Mock「笔记编译」JSON
  │                                 index wiki:// pages → Library「笔记页」
  ├─ set __JARVIS_E2E_WIKI_EXPORT_PATH__ = %TEMP%/jarvis-e2e-wiki.zip
  ├─ click wiki-export ───────────► preflight → skip save() → export_wiki_zip
  │                                 AppShell wiki-export-done
  ├─ fs.stat(zip).size > 0
  └─ askQuestion("What is xyzzy-plugh?")
        retrieve (FTS+vector+RRF) → citations with source_uri
        ChatView button[data-source-uri] ──► assert ≥1 !startsWith("wiki://")
```

### Recommended Project Structure

```text
e2e/
├── helpers.ts              # + setWikiExportPath / assertWikiZipExists (Discretion)
├── fixtures/sample.md      # xyzzy-plugh (unchanged)
└── specs/
    ├── wiki.spec.ts        # NEW — WIKI-08 + WIKI-09 journey
    ├── full-ui.spec.ts     # light Library default-off only
    └── qa.spec.ts          # UNCHANGED (D-11)
src/
├── hooks/useLibrary.ts     # exportWiki honors __JARVIS_E2E_WIKI_EXPORT_PATH__
├── views/ChatView.tsx      # data-source-uri on citation <button>
└── views/SettingsView.tsx  # optional data-testid on 保存配置 (Discretion)
.cursor/rules/e2e-required.mdc   # Wiki → wiki.spec.ts
e2e/README.md                    # same row
```

### Pattern 1: Window E2E hooks (existing)

**What:** App exposes `window.__JARVIS_E2E_*` only in E2E / when needed; specs call via `browser.execute`.
**When to use:** Bypass UI that WebDriver cannot drive (orchestration, ask, Save dialog).
**Example (export path — implement analog):**

```typescript
// Source: e2e/helpers.ts setOrchestrationMode + App.tsx __JARVIS_E2E_* [VERIFIED]
await browser.execute((dest: string) => {
  (window as Window & { __JARVIS_E2E_WIKI_EXPORT_PATH__?: string })
    .__JARVIS_E2E_WIKI_EXPORT_PATH__ = dest;
}, zipPath);

// useLibrary.exportWiki — when set, skip save():
const forced =
  (window as Window & { __JARVIS_E2E_WIKI_EXPORT_PATH__?: string })
    .__JARVIS_E2E_WIKI_EXPORT_PATH__;
const destPath = forced ?? (await save({ /* ... */ }));
if (!forced && destPath === null) return;
await exportWikiZipCmd(destPath);
```

### Pattern 2: Hybrid Settings enable (D-03)

**What:** Expand `settings-section-wiki` (default collapsed), `setReactCheckbox` on `wiki-enabled-toggle`, click「保存配置」, then Library.
**When to use:** Positive wiki path only — never seed `wiki.enabled=true` in `apply_e2e_config`.
**Note:** AccordionSection defaults `defaultExpanded=false`; wiki section must use `expandSettingsSection("settings-section-wiki")`. [VERIFIED: `SettingsView.tsx`]

### Pattern 3: First eligible compile button (D-04)

**What:** After enable + save, in Library:

```typescript
const btn = await $('[data-testid^="wiki-compile-"]');
await btn.waitForDisplayed({ timeout: 30_000 });
await clickViaDom(`[data-testid="${await btn.getAttribute("data-testid")}"]`);
```

Do **not** hardcode fixture title. Wait for busy clear + `source-list` text includes「笔记页」.

### Pattern 4: Citation URI attribute (D-09 / D-10)

**What:** Put `data-source-uri={c.source_uri}` on the citation **button** (same element that already calls `openCitation(c.source_uri)`). Keep `data-testid="citation-excerpt"` on the excerpt div unchanged so `qa.spec` stays green.

```typescript
// ChatView — surgical add [VERIFIED shape: ChatView.tsx ~201-217]
<button
  type="button"
  data-source-uri={c.source_uri}
  data-testid="citation-source"  // optional; not locked — URI attr is mandatory
  ...
>
```

E2E assert:

```typescript
const uris = await browser.execute(() =>
  [...document.querySelectorAll("[data-source-uri]")].map((el) =>
    el.getAttribute("data-source-uri"),
  ),
);
expect(uris.some((u) => u && !u.startsWith("wiki://"))).toBe(true);
```

### Anti-Patterns to Avoid

- **Driving native Save dialog:** Locked out — use path hook (D-05).
- **Seeding `wiki.enabled=true` in `apply_e2e_config`:** Breaks D-03 hybrid enable and default-off proofs.
- **Asserting only `citation-excerpt` for WIKI-08:** Excerpt alone cannot prove non-wiki URI — need `data-source-uri`.
- **Editing `qa.spec.ts`:** Forbidden by D-11.
- **Unzip / Obsidian stub checks in E2E:** Deferred; Phase 12 owns them.
- **Hard-binding fixture display name for compile:** Use first `wiki-compile-*` (D-04).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| OS Save dialog in CI | Win32 dialog automation | `__JARVIS_E2E_WIKI_EXPORT_PATH__` | Fragile; D-05 locks FE hook |
| Custom E2E chat submit | New submit helpers | `askQuestion` / `setReactInputValue` / `clickViaDom` | Already battle-tested |
| Custom checkbox setter | `.click()` alone | `setReactCheckbox` | React controlled inputs ignore plain clicks |
| New Mock chat provider | Separate E2E chat double | Existing `MockChatModel`「笔记编译」branch | Already ordered before 记忆/任务/摘要 |
| New citation parser for E2E | Re-parse JSON in browser | `data-source-uri` DOM attr | Matches D-09; `parseCitations` already feeds UI |
| Fresh zip assert via unzip | AdmZip / manual inflate | `fs.stat` size > 0 | D-06 |

**Key insight:** Phase 13 succeeds by composing existing IPC/UI with two surgical FE hooks (export path + URI attr) and one new focused spec — not by extending RAG or Mock semantics.

## Common Pitfalls

### Pitfall 1: Save dialog hangs the suite
**What goes wrong:** Click `wiki-export` → native Save dialog → WDIO timeout.
**Why it happens:** `exportWiki` always calls `save()` today. [VERIFIED: `useLibrary.ts`]
**How to avoid:** Implement D-05 before / with the export step; set path **before** click; Vitest case: forced path skips `save`, still calls `exportWikiZip`.
**Warning signs:** Spec stuck after export click; no `wiki-export-done`.

### Pitfall 2: Wiki accordion collapsed / 保存配置 has no testid
**What goes wrong:** Toggle not visible; save click misses.
**Why it happens:** `settings-section-wiki` uses default collapsed accordion;「保存配置」button has **no** `data-testid`. [VERIFIED: `SettingsView.tsx`]
**How to avoid:** Always `expandSettingsSection("settings-section-wiki")`. Discretion: add `data-testid="settings-save-config"` **or** `clickViaDom` via `button` containing text「保存配置」in settings panel.
**Warning signs:** Compile buttons never appear after “enable”.

### Pitfall 3: Shared session enables wiki before `full-ui`
**What goes wrong:** Library shows `wiki-export` during default-off assert.
**Why it happens:** One app process for all specs; config persists after `set_config` until process exit. E2E dir wipe only on **app start**. [VERIFIED: `state.rs`, `wdio.conf.ts`]
**How to avoid:** Keep `wiki.spec.ts` last alphabetically (name `wiki.spec.ts`); put `full-ui` default-off early; optionally turn wiki off + save at end of `wiki.spec`. Do **not** rely on cross-run persistence wipe for mid-suite isolation.
**Warning signs:** Flaky default-off only when suite order changes.

### Pitfall 4: Stale zip size assertion
**What goes wrong:** `size > 0` passes from a previous run’s file.
**Why it happens:** Fixed `%TEMP%/jarvis-e2e-wiki.zip` (D-07).
**How to avoid:** `fs.unlinkSync` if exists at start of export step; then export; then `stat`.
**Warning signs:** Export failed but assert still green.

### Pitfall 5: Citation attr on wrong element / qa regression
**What goes wrong:** Moving `citation-excerpt` or changing qa expectations.
**Why it happens:** Temptation to put URI on excerpt div or rename testids.
**How to avoid:** Add `data-source-uri` on the **button**; leave `citation-excerpt` untouched (D-11).
**Warning signs:** `qa.spec` Chinese citation test fails.

### Pitfall 6: Compile success detected too early
**What goes wrong:** Assert「笔记页」before index refresh finishes.
**Why it happens:** `compileWiki` awaits IPC then `refreshSources`, but busy overlay timing varies.
**How to avoid:** Wait until `busy-overlay` gone **and** `source-list` contains「笔记页」with 30–90s timeout (Mock is fast; still allow headroom).
**Warning signs:** Intermittent missing「笔记页」.

### Pitfall 7: Assuming Mock must change for citation trust
**What goes wrong:** Unnecessary Mock / e2e.rs edits.
**Why it happens:** Fear wiki chunks replace fixture.
**How to avoid:** Fixture `sample.md` contains **xyzzy-plugh**; Mock wiki JSON summary/entities do **not**. FTS should still hit the original; assert is “≥1 non-wiki URI”, not “zero wiki URIs”. Only extend Mock if compile fails or answers become empty. [VERIFIED: `e2e/fixtures/sample.md`, `crates/llm/src/mock.rs`]

### Pitfall 8: `wiki-export-done` vs error bar confusion
**What goes wrong:** Asserting success on red `err`.
**Why it happens:** Phase 12 soft notice is separate emerald panel.
**How to avoid:** Wait for `[data-testid="wiki-export-done"]`; failures stay on existing error UI. [VERIFIED: `AppShell.tsx`]

## Code Examples

### Wiki spec skeleton (prescriptive)

```typescript
// e2e/specs/wiki.spec.ts — Source: CONTEXT D-01..D-10 + helpers patterns [VERIFIED]
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {
  askQuestion,
  clickViaDom,
  expandSettingsSection,
  openNav,
  setReactCheckbox,
} from "../helpers.ts";

const zipPath = path.join(os.tmpdir(), "jarvis-e2e-wiki.zip");

describe("Jarvis wiki journey", () => {
  it("hides wiki Library controls when disabled (default-off)", async () => {
    await openNav("library", '[data-testid="library-stats"]');
    expect(await $$('[data-testid="wiki-export"]')).toHaveLength(0);
    expect(await $$('[data-testid^="wiki-compile-"]')).toHaveLength(0);
  });

  it("enable → compile → 笔记页 → export → citation trust", async () => {
    await openNav("settings", '[data-testid="settings-panel"]');
    await expandSettingsSection("settings-section-wiki");
    await setReactCheckbox(await $('[data-testid="wiki-enabled-toggle"]'), true);
    await clickViaDom('[data-testid="settings-save-config"]'); // if added; else text selector
    // wait busy clear

    await openNav("library", '[data-testid="source-list"]');
    const compile = await $('[data-testid^="wiki-compile-"]');
    await compile.waitForDisplayed({ timeout: 30_000 });
    await clickViaDom(`[data-testid="${await compile.getAttribute("data-testid")}"]`);
    await browser.waitUntil(
      async () => (await $('[data-testid="source-list"]').getText()).includes("笔记页"),
      { timeout: 90_000, timeoutMsg: "wiki_page not listed" },
    );

    if (fs.existsSync(zipPath)) fs.unlinkSync(zipPath);
    await browser.execute((p) => {
      (window as any).__JARVIS_E2E_WIKI_EXPORT_PATH__ = p;
    }, zipPath);
    await clickViaDom('[data-testid="wiki-export"]');
    await $('[data-testid="wiki-export-done"]').waitForDisplayed({ timeout: 60_000 });
    const st = fs.statSync(zipPath);
    expect(st.size).toBeGreaterThan(0);

    await askQuestion("What is xyzzy-plugh?");
    const uris = await browser.execute(() =>
      [...document.querySelectorAll("[data-source-uri]")].map((e) =>
        e.getAttribute("data-source-uri"),
      ),
    );
    expect(uris.some((u) => !!u && !u.startsWith("wiki://"))).toBe(true);
  });
});
```

### `exportWiki` bypass (prescriptive)

```typescript
// src/hooks/useLibrary.ts — extend existing exportWiki [VERIFIED base]
const preflight = await wikiExportPreflight();
if (!preflight.hasNotes) {
  reportError("还没有可导出的笔记，请先生成笔记");
  return;
}
const e2ePath =
  typeof window !== "undefined"
    ? (window as Window & { __JARVIS_E2E_WIKI_EXPORT_PATH__?: string })
        .__JARVIS_E2E_WIKI_EXPORT_PATH__
    : undefined;
const destPath =
  e2ePath ??
  (await save({
    defaultPath: `jarvis-wiki-${localYmd()}.zip`,
    filters: [{ name: "Zip", extensions: ["zip"] }],
  }));
if (destPath === null || destPath === undefined) return;
await exportWikiZipCmd(destPath);
```

### `full-ui` light default-off (prescriptive)

```typescript
// Inside existing full-ui library/stats area — D-12 only
await openNav("library");
expect(await $$('[data-testid="wiki-export"]')).toHaveLength(0);
expect(await $$('[data-testid^="wiki-compile-"]')).toHaveLength(0);
// Do NOT enable wiki / compile / export here
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Monolith `full-ui` only | Focused specs (`qa`, `lark`, `memory`, …) + light `full-ui` | v1.x E2E expansion | Put wiki in `wiki.spec.ts`, not a full positive path in `full-ui` (D-12) |
| Native dialog for file pick | E2E window hooks for non-automatable OS UI | Existing orch/ask hooks | Extend same pattern for export path |
| Citation trust via excerpt text | URI attribute on citation control | Phase 13 | Distinguishes `wiki://` vs fixture file URI |

**Deprecated/outdated:**
- Plan draft Task 7 mention of `wiki-compile-done` toast — Phase 11 locked **silent refresh +「笔记页」rows** as success signal (D-02 / Phase 11 D-11). Prefer「笔记页」, not a compile-done toast.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Alphabetical WDIO spec order keeps `wiki.spec.ts` after `full-ui` / `qa` | Pitfall 3 | Default-off flakes if order changes — mitigate by end-of-spec wiki disable |
| A2 | Mock wiki pages without `xyzzy-plugh` will not sole-replace fixture citations under hybrid retrieve | WIKI-08 / Pitfall 7 | If assert fails, inspect top hits; optionally narrow retrieve or ask again — do not weaken D-10 |
| A3 | Honoring `__JARVIS_E2E_WIKI_EXPORT_PATH__` whenever set (even outside e2eMode) is acceptable | Pattern 1 | Low: unset in prod; Discretion may gate on `e2eMode` if planner prefers |

**If wrong:** A1/A3 are process mitigations; A2 is the only product-risk assumption — validate by running the wiki citation step once green.

## Open Questions (RESOLVED)

1. **Save-button selector** — **RESOLVED** (13-01-PLAN)
   - What we know:「保存配置」has no `data-testid` today.
   - Decision (locked in Plan 01): Add `data-testid="settings-save-config"` on Settings「保存配置」button; wiki.spec clicks via that testid (not text-only DOM click).

2. **Settings default-off depth in `wiki.spec` / `full-ui`** — **RESOLVED** (13-02-PLAN)
   - What we know: Library absence is mandatory (D-12).
   - Decision (locked in Plan 02): Library absence only (`wiki-export` / `wiki-compile-*` length 0) in both `wiki.spec` opening it and `full-ui`; do not require unchecked `wiki-enabled-toggle` assert.

3. **End-of-spec wiki disable** — **RESOLVED** (13-02-PLAN)
   - What we know: Shared session pollution risk if order changes.
   - Decision (locked in Plan 02): Optional disable (toggle off + save) at end of positive `wiki.spec` it — Discretion insurance, not mandatory.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Node | WDIO | ✓ | v24.5.0 | — |
| npm | scripts | ✓ | 12.0.1 | — |
| Rust stable | `build:e2e` | ✓ | 1.97.0 | — |
| `tauri-driver` | E2E | ✓ | on PATH (no `--version` flag) | `cargo install tauri-driver` |
| `msedgedriver.exe` | E2E | ✓ | repo root | CI installs via tool |
| `target/release/tauri-app.exe` | E2E | ✓ | present | `npm run build:e2e` |
| Live LLM / Feishu | — | N/A | — | Forbidden; Mock only |

**Missing dependencies with no fallback:** none for this machine snapshot.

**Missing dependencies with fallback:** none.

Step 2.6 note: Phase depends on existing E2E toolchain (not new installs). Re-run `npm run build:e2e` after FE hook changes before `test:e2e`.

## Validation Architecture

> `workflow.nyquist_validation: true` in `.planning/config.json` — section required.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | WebdriverIO 9 + Mocha (`@wdio/mocha-framework`) + Vitest 3 (optional FE units) |
| Config file | `e2e/wdio.conf.ts`; Vitest via `vite.config.ts` |
| Quick run command | `cross-env JARVIS_E2E=1 JARVIS_E2E_FIXTURE=./e2e/fixtures/sample.md npx wdio run e2e/wdio.conf.ts --spec e2e/specs/wiki.spec.ts` |
| Full suite command | `npm run test:e2e:local` (build + all `e2e/specs/**/*.spec.ts`) |
| Companion unit (Discretion) | `npx vitest run src/hooks/useLibrary.test.ts` (+ ChatView test if added) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| WIKI-09 | default-off: no Library wiki controls | E2E | `--spec e2e/specs/wiki.spec.ts` (opening it) + `full-ui` light assert | ❌ Wave 0 — create `wiki.spec.ts`; extend `full-ui.spec.ts` |
| WIKI-09 | enable → compile →「笔记页」→ export notice + zip size>0 | E2E | `--spec e2e/specs/wiki.spec.ts` | ❌ Wave 0 |
| WIKI-09 | docs map Wiki → wiki.spec | Docs | Manual/rg check of `e2e-required.mdc` + `e2e/README.md` | ❌ Wave 0 — rows missing |
| WIKI-08 | after wiki on+compiled, ≥1 citation URI not `wiki://` for `What is xyzzy-plugh?` | E2E | same wiki.spec | ❌ Wave 0 |
| WIKI-08 | default-off qa citation excerpt regression | E2E | `--spec e2e/specs/qa.spec.ts` (unchanged) | ✅ `qa.spec.ts` |
| WIKI-09 harness | export path bypass skips `save` | Unit | `npx vitest run src/hooks/useLibrary.test.ts` | ✅ file exists — ❌ case for forced path (Wave 0 gap) |
| WIKI-08 harness | `data-source-uri` rendered | E2E (required) / Vitest optional | wiki.spec URI query | ❌ attr not in ChatView yet |

### Sampling Rate

- **Per task commit:** focused `--spec e2e/specs/wiki.spec.ts` after `build:e2e` when FE changed; Vitest for touched hook/view
- **Per wave merge:** `npm run test:e2e:local` (or at least `wiki` + `qa` + `full-ui` specs)
- **Phase gate:** Full `npm run test:e2e:local` green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `e2e/specs/wiki.spec.ts` — WIKI-08 + WIKI-09 (does not exist)
- [ ] `e2e/helpers.ts` — `setWikiExportPath` + `assertWikiZipNonEmpty` (or inline; Discretion)
- [ ] `src/hooks/useLibrary.ts` — honor `__JARVIS_E2E_WIKI_EXPORT_PATH__`
- [ ] `src/views/ChatView.tsx` — `data-source-uri` on citation buttons
- [ ] `e2e/specs/full-ui.spec.ts` — light Library default-off asserts
- [ ] `.cursor/rules/e2e-required.mdc` + `e2e/README.md` — Wiki row (D-13)
- [ ] (Recommended) Vitest: forced export path skips `save`
- [ ] (Recommended) `data-testid="settings-save-config"` on Settings save button

Framework install: none — already present.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | Local desktop; no new auth |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes (light) | Export dest path from E2E hook / Save dialog; backend `export_wiki_zip` already path-safe for zip **entries**; dest is caller-chosen write path (existing Phase 12 trust model) |
| V6 Cryptography | no | — |

### Known Threat Patterns for this phase

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Prod user sets malicious `__JARVIS_E2E_WIKI_EXPORT_PATH__` via injected script | Tampering | Hook only useful if script runs in webview; mirror other `__JARVIS_E2E_*` hooks; optional honor only when `e2eMode` / `JARVIS_E2E` (Discretion) |
| E2E zip write outside temp | Tampering | Spec uses fixed `os.tmpdir()/jarvis-e2e-wiki.zip` only |
| Citation URI XSS via attribute | XSS | `source_uri` is app-controlled store URI (`file://`, `wiki://`, …); React text/attr escaping; no `dangerouslySetInnerHTML` |

`security_enforcement` not disabled in config — threats above are sufficient for harness-only phase.

## Project Constraints (from .cursor/rules/)

| Rule | Directive for Phase 13 |
|------|------------------------|
| `e2e-required.mdc` | User-facing wiki journey must have E2E; add Wiki → `wiki.spec.ts` to spec map; deterministic mocks (`JARVIS_E2E=1`); stable `data-testid`s |
| `tdd-goal-driven.mdc` | Write failing `wiki.spec` / Vitest first; then minimal FE hooks; run suite before done |
| `karpathy-guidelines.mdc` | Surgical diffs only — no RAG redesign, no qa.spec edits, no unzip scope creep |
| `jarvis-stack.mdc` | No stack changes; keep Tauri 2 / WDIO 9 / Vitest 3 |
| `frontend-taste.mdc` | Citation attr / optional save testid only — no visual redesign |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/13-e2e-citation-trust/13-CONTEXT.md` — locked D-01..D-13
- `e2e/helpers.ts`, `e2e/specs/{qa,full-ui,settings}.spec.ts`, `e2e/wdio.conf.ts`, `e2e/fixtures/sample.md`, `e2e/README.md`
- `src/hooks/useLibrary.ts`, `src/views/{ChatView,LibraryView,SettingsView,AppShell}.tsx`
- `src-tauri/src/{e2e.rs,state.rs}`, `crates/llm/src/mock.rs`, `crates/rag/src/ask.rs`
- `.cursor/rules/e2e-required.mdc`, `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` Task 7
- Phase 11/12 CONTEXT — UI/export decisions deferred E2E to 13

### Secondary (MEDIUM confidence)

- Task 7 draft still mentions `wiki-compile-done` — superseded by Phase 11 silent「笔记页」success (treat CONTEXT as normative)

### Tertiary (LOW confidence)

- Assumptions A1–A3 (suite order, retrieval mix, hook gating)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — existing WDIO/Tauri E2E; no new packages
- Architecture: HIGH — verified hooks, testids, Mock, state wipe, citation pipeline
- Pitfalls: HIGH — Save dialog, shared session, accordion/save testid, stale zip all grounded in code

**Research date:** 2026-07-24
**Valid until:** 2026-08-24 (stable harness; re-check if WDIO/Tauri E2E bootstrap changes)
