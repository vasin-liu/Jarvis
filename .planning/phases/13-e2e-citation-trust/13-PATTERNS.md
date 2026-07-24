# Phase 13: E2E + citation trust - Pattern Map

**Mapped:** 2026-07-24
**Files analyzed:** 9 (7 locked + 2 discretionary)
**Analogs found:** 9 / 9

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `e2e/specs/wiki.spec.ts` | test | request-response (UI journey) | `e2e/specs/lark.spec.ts` (+ `qa.spec.ts` for ask/citations) | exact |
| `e2e/specs/full-ui.spec.ts` | test | request-response | `e2e/specs/full-ui.spec.ts` library-stats `it` (extend in place) | exact |
| `e2e/helpers.ts` | utility | request-response / file-I/O | `e2e/helpers.ts` `setOrchestrationMode` + `waitForDomText` | exact |
| `src/hooks/useLibrary.ts` | hook | file-I/O (dialog → IPC) | `src/hooks/useLibrary.ts` `exportWiki` / `pickAndIndex` | exact |
| `src/views/ChatView.tsx` | component | request-response | `src/views/ChatView.tsx` citation `<button>` block | exact |
| `src/views/SettingsView.tsx` | component | request-response | `src/views/SettingsView.tsx`「保存配置」+ `lark-sync-submit` testid pattern | exact |
| `src/hooks/useLibrary.test.ts` | test | request-response | `src/hooks/useLibrary.test.ts` existing `exportWiki` cases | exact |
| `.cursor/rules/e2e-required.mdc` | config | — | same file Spec map table | exact |
| `e2e/README.md` | config | — | same file Specs table | exact |

## Pattern Assignments

### `e2e/specs/wiki.spec.ts` (test, request-response)

**Analog (primary):** `e2e/specs/lark.spec.ts` — focused Settings → action → Library list wait  
**Analog (citations):** `e2e/specs/qa.spec.ts` — `askQuestion("What is xyzzy-plugh?")` + citation DOM assert  
**Analog (prefix selector):** `e2e/specs/memory.spec.ts` — first `[data-testid^="…"]` match

**Imports pattern** (from `lark.spec.ts` lines 1–7 + `qa` askQuestion):

```typescript
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {
  askQuestion,
  clickViaDom,
  expandSettingsSection,
  openNav,
  setReactCheckbox,
  // + new helpers: setWikiExportPath / assertWikiZipNonEmpty (Discretion)
} from "../helpers.ts";
```

**Default-off / Library absence** (compose from RESEARCH + `full-ui` library open):

```typescript
await openNav("library", '[data-testid="library-stats"]');
expect(await $$('[data-testid="wiki-export"]')).toHaveLength(0);
expect(await $$('[data-testid^="wiki-compile-"]')).toHaveLength(0);
```

**Hybrid Settings enable** (copy `lark.spec.ts` expand + `full-ui.spec.ts` `setReactCheckbox`):

```typescript
// Source: e2e/specs/lark.spec.ts lines 14–15, 30–31
await openNav("settings", '[data-testid="settings-panel"]');
await expandSettingsSection("settings-section-wiki");
await setReactCheckbox(await $('[data-testid="wiki-enabled-toggle"]'), true);
await clickViaDom('[data-testid="settings-save-config"]'); // after adding testid
```

**First eligible compile + list wait** (copy `memory.spec.ts` prefix + `lark.spec.ts` list wait):

```typescript
// Source: e2e/specs/memory.spec.ts line 23; e2e/specs/lark.spec.ts lines 36–45
await openNav("library", '[data-testid="source-list"]');
const compile = await $('[data-testid^="wiki-compile-"]');
await compile.waitForDisplayed({ timeout: 30_000 });
await clickViaDom(`[data-testid="${await compile.getAttribute("data-testid")}"]`);
await browser.waitUntil(
  async () => (await $('[data-testid="source-list"]').getText()).includes("笔记页"),
  { timeout: 90_000, timeoutMsg: "wiki_page not listed" },
);
```

**Q&A + citation assert** (extend `qa.spec.ts` — do not edit qa):

```typescript
// Source: e2e/specs/qa.spec.ts lines 8–21 — keep citation-excerpt alone in qa;
// wiki.spec adds URI query after same askQuestion text
const answer = await askQuestion("What is xyzzy-plugh?");
expect((await answer.getText()).length).toBeGreaterThan(0);
const uris = await browser.execute(() =>
  [...document.querySelectorAll("[data-source-uri]")].map((e) =>
    e.getAttribute("data-source-uri"),
  ),
);
expect(uris.some((u) => !!u && !u.startsWith("wiki://"))).toBe(true);
```

**File header style:** match focused specs (`lark.spec.ts` / `memory.spec.ts`) — short `describe("Jarvis wiki journey")`, no shared `beforeEach` required.

---

### `e2e/specs/full-ui.spec.ts` (test, request-response)

**Analog:** same file — `"shows library stats after seeding"` `it` (lines 49–53)

**Core pattern — light default-off only (D-12):**

```typescript
// Extend existing library it OR add sibling it immediately after nav/library checks.
// Source placement: e2e/specs/full-ui.spec.ts lines 49–53
it("shows library stats after seeding", async () => {
  await openNav("library");
  const stats = await $('[data-testid="library-stats"]');
  await expect(stats).toHaveText(expect.stringContaining("已索引"));
  // ADD (D-12):
  expect(await $$('[data-testid="wiki-export"]')).toHaveLength(0);
  expect(await $$('[data-testid^="wiki-compile-"]')).toHaveLength(0);
});
```

**Do not copy:** positive wiki enable/compile/export from RESEARCH skeleton into `full-ui` — that lives only in `wiki.spec.ts`.

---

### `e2e/helpers.ts` (utility, request-response + file-I/O)

**Analog (window hook):** `setOrchestrationMode` lines 100–127  
**Analog (DOM text wait):** `waitForDomText` lines 26–44  
**Analog (click):** `clickViaDom` lines 146–151

**Window E2E hook pattern** (copy shape; new name `__JARVIS_E2E_WIKI_EXPORT_PATH__`):

```typescript
// Source: e2e/helpers.ts lines 104–112
await browser.execute((m) => {
  (
    window as Window & {
      __JARVIS_E2E_SET_ORCHESTRATION__?: (
        mode: "pipeline" | "router" | "single",
      ) => void;
    }
  ).__JARVIS_E2E_SET_ORCHESTRATION__?.(m);
}, mode);
```

**Prescribed helpers (Discretion — mirror above):**

```typescript
export async function setWikiExportPath(destPath: string) {
  await browser.execute((p: string) => {
    (
      window as Window & { __JARVIS_E2E_WIKI_EXPORT_PATH__?: string }
    ).__JARVIS_E2E_WIKI_EXPORT_PATH__ = p;
  }, destPath);
}

export function wikiE2eZipPath() {
  return path.join(os.tmpdir(), "jarvis-e2e-wiki.zip");
}

export function assertWikiZipNonEmpty(zipPath: string) {
  const st = fs.statSync(zipPath);
  if (st.size <= 0) throw new Error(`wiki zip empty or missing: ${zipPath}`);
}
```

**Node FS note:** helpers that use `fs`/`os`/`path` run in the WDIO Node process (same as RESEARCH D-06/D-07) — import Node builtins at top of `helpers.ts` only if exporting those helpers there; alternatively keep FS unlink/stat inline in `wiki.spec.ts` (Discretion). Prefer helpers for planner reuse.

---

### `src/hooks/useLibrary.ts` (hook, file-I/O)

**Analog:** same file `exportWiki` (lines 154–175) + `pickAndIndex` early-return when dialog null (lines 130–138)

**Current core** (extend surgically):

```typescript
// Source: src/hooks/useLibrary.ts lines 154–175
const exportWiki = useCallback(
  async ({ onSuccess }: ExportWikiOptions = {}) => {
    try {
      const preflight = await wikiExportPreflight();
      if (!preflight.hasNotes) {
        reportError("还没有可导出的笔记，请先生成笔记");
        return;
      }
      const destPath = await save({
        defaultPath: `jarvis-wiki-${localYmd()}.zip`,
        filters: [{ name: "Zip", extensions: ["zip"] }],
      });
      if (destPath === null) return;
      await exportWikiZipCmd(destPath);
      onSuccess?.(destPath);
      return destPath;
    } catch (error) {
      reportError(error);
    }
  },
  [reportError],
);
```

**Prescribed change** (RESEARCH Pattern 1 — keep try/catch + preflight + onSuccess):

```typescript
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
onSuccess?.(destPath);
```

**Error handling:** keep existing `reportError(error)` catch — do not introduce new error types.  
**Optional gate:** honor only when `e2eMode` if planner prefers (Assumption A3) — other hooks in `App.tsx` gate on `e2eMode` for *registration*; reading a path string may stay ungated like RESEARCH default.

**Related success UI (do not change):** `AppShell.tsx` lines 101–118 — `data-testid="wiki-export-done"` emerald notice already wired via `onSuccess` → `setNotice`.

---

### `src/views/ChatView.tsx` (component, request-response)

**Analog:** same file citation block (lines 196–219)

**Core pattern — surgical attr only (D-09 / UI-SPEC):**

```typescript
// Source: src/views/ChatView.tsx lines 199–217
<button
  type="button"
  className="text-left text-xs text-cyan-300 hover:underline"
  onClick={() =>
    openCitation(c.source_uri).catch((e) =>
      setErr(String(e)),
    )
  }
>
  {c.source_title} · {c.loc}
</button>
<div
  data-testid="citation-excerpt"
  className="mt-1 text-zinc-400"
>
  {c.excerpt}
</div>
```

**Add on the `<button>` only:**

```typescript
data-source-uri={c.source_uri}
```

**Forbidden:** move/rename `citation-excerpt`; add `data-testid="citation-source"` (UI-SPEC: skip); restyle classes.

---

### `src/views/SettingsView.tsx` (component, request-response) — ASSUMED recommended

**Analog:** same file「保存配置」button (lines 1205–1214) + `data-testid` on peer CTA `lark-sync-submit` (lines 1111–1118)

**Current save button:**

```typescript
// Source: src/views/SettingsView.tsx lines 1205–1214
<button
  type="button"
  className="btn-primary w-fit"
  disabled={busy}
  onClick={() => {
    void handleSaveConfig(config).then(() => refreshIndexStatus());
  }}
>
  保存配置
</button>
```

**Copy testid placement from:**

```typescript
// Source: src/views/SettingsView.tsx lines 1111–1118
<button
  type="button"
  className="btn-primary shrink-0"
  data-testid="lark-sync-submit"
  disabled={busy || !larkDocToken.trim()}
  onClick={() => void syncLark("doc", larkDocToken.trim())}
>
```

**Prescribed:** add `data-testid="settings-save-config"` — no class/copy change.

**Wiki toggle already exists** — reuse `wiki-enabled-toggle` / `settings-section-wiki` (no new controls). Expand via `expandSettingsSection("settings-section-wiki")` like `settings-section-lark` in `lark.spec.ts`.

---

### `src/hooks/useLibrary.test.ts` (test, request-response) — recommended Vitest

**Analog:** same file `exportWiki` cases (lines 116–178)

**Pattern to extend:**

```typescript
// Source: src/hooks/useLibrary.test.ts lines 145–159
it("exportWiki saves then exports and notifies onSuccess", async () => {
  const { wikiExportPreflight, exportWikiZip } = await import("../lib/tauri");
  const { save } = await import("@tauri-apps/plugin-dialog");
  const dest = "C:\\exports\\jarvis-wiki-2026-07-23.zip";
  vi.mocked(wikiExportPreflight).mockResolvedValueOnce({ hasNotes: true });
  vi.mocked(save).mockResolvedValueOnce(dest);
  const onSuccess = vi.fn();
  const { result } = renderHook(() => useLibrary());

  await act(async () => {
    await result.current.exportWiki({ onSuccess });
  });

  expect(exportWikiZip).toHaveBeenCalledWith(dest);
  expect(onSuccess).toHaveBeenCalledWith(dest);
});
```

**New case:** set `(window as any).__JARVIS_E2E_WIKI_EXPORT_PATH__ = forcedPath` before `exportWiki`; expect `save` **not** called; `exportWikiZip` called with forced path. Clear window prop in `afterEach` / finally.

---

### `.cursor/rules/e2e-required.mdc` (config)

**Analog:** same file Spec map (lines 26–37)

```markdown
| Area | Spec file |
|------|-----------|
| Full journey | `full-ui.spec.ts` |
| Lark / Feishu | `lark.spec.ts` |
| Memory | `memory.spec.ts` |
| Agent | `agent.spec.ts` |
| Q&A | `qa.spec.ts` |
| Settings / index | `settings.spec.ts` |
| Navigation | `navigation.spec.ts` |
| Smoke | `smoke.spec.ts` |
```

**Add row (D-13):** `| Wiki | \`wiki.spec.ts\` |` — keep alphabetical-ish placement near Q&A or after Settings; name must remain `wiki.spec.ts` so suite order stays last.

---

### `e2e/README.md` (config)

**Analog:** same file Specs table (lines 43–54)

```markdown
| File | Coverage |
|------|----------|
| `full-ui.spec.ts` | Full journey: nav, Q&A, memory, agent, settings, **Lark** |
| `lark.spec.ts` | Feishu connection check + URL sync (E2E mock, no live lark-cli) |
| ...
```

**Add row (D-13):** `| \`wiki.spec.ts\` | Wiki enable → compile →「笔记页」→ export + citation URI trust (WIKI-08/09); default-off Library controls |`  
Optionally note `full-ui` light default-off only in that row or in `full-ui` coverage cell.

---

## Shared Patterns

### Window `__JARVIS_E2E_*` hooks
**Source:** `e2e/helpers.ts` `setOrchestrationMode` (100–127); app registration in `src/App.tsx` (142–170)  
**Apply to:** `__JARVIS_E2E_WIKI_EXPORT_PATH__` consumer in `useLibrary.exportWiki` + setter helper in `helpers.ts`  
**Note:** Orchestration hooks are *functions* registered when `e2eMode`; export path is a *string* set from WDIO — closer to a write-once bypass flag than App.tsx registration. Prefer reading the string in the hook (RESEARCH) rather than adding a new App.tsx registrar unless planner wants symmetry.

### React-controlled inputs in E2E
**Source:** `e2e/helpers.ts` `setReactCheckbox` (47–71), `clickViaDom` (146–151)  
**Apply to:** wiki enable toggle; compile/export/save clicks — never rely on WebDriver `.click()` alone for React handlers.

### Focused spec + light `full-ui`
**Source:** `lark.spec.ts` / `memory.spec.ts` / `agent.spec.ts` vs `full-ui.spec.ts`  
**Apply to:** full positive wiki journey in `wiki.spec.ts`; `full-ui` only asserts absence of wiki Library controls (D-12). Leave `qa.spec.ts` untouched (D-11).

### Hide-when-disabled (not disable)
**Source:** `src/views/LibraryView.tsx` lines 142–152 (`config?.wiki?.enabled === true && …`)  
**Apply to:** E2E default-off asserts use `$$(...).toHaveLength(0)`, not “disabled” checks.

### Soft success notice vs error bar
**Source:** `src/AppShell.tsx` lines 101–118 (`wiki-export-done`) vs red error panel  
**Apply to:** export success wait targets emerald `wiki-export-done` only.

### Citation success label
**Source:** `src/lib/sourceDisplay.ts` `wiki_page` → `笔记页`  
**Apply to:** Library compile success = `source-list` text includes「笔记页」(D-02), not a compile toast.

### Spec map dual update
**Source:** `.cursor/rules/e2e-required.mdc` + `e2e/README.md`  
**Apply to:** both get Wiki → `wiki.spec.ts` (D-13).

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | None — every Phase 13 file has an in-repo exact or same-file analog |

**Closest stretch:** Node `fs.stat` zip size assert has no prior E2E helper — use Node builtins in WDIO process (standard Node, not a project pattern gap). RESEARCH marks this as intentional.

## Metadata

**Analog search scope:** `e2e/specs/`, `e2e/helpers.ts`, `e2e/README.md`, `src/hooks/`, `src/views/`, `src/App.tsx`, `src/AppShell.tsx`, `src/lib/sourceDisplay.ts`, `.cursor/rules/e2e-required.mdc`  
**Files scanned:** ~20 primary (specs + hooks/views + rules)  
**Pattern extraction date:** 2026-07-24  
**Rules skimmed:** `e2e-required.mdc`, `frontend-taste.mdc`, `tdd-goal-driven.mdc`, `karpathy-guidelines.mdc`, `jarvis-stack.mdc`
