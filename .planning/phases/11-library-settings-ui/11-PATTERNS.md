# Phase 11: Library / Settings UI - Pattern Map

**Mapped:** 2026-07-23  
**Files analyzed:** 8  
**Analogs found:** 8 / 8

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/views/SettingsView.tsx` | view | form-state | same file — `AccordionSection` +「自动洞察」checkbox +「保存配置」 | exact |
| `src/views/LibraryView.tsx` | view | props→UI | same file — `summarize-{id}` / `extract-tasks-{id}` action column | exact |
| `src/views/LibraryView.test.tsx` | test | props→UI | `src/views/ChatView.test.tsx` (view props + motion mock) | role-match |
| `src/hooks/useLibrary.ts` | hook | request-response | same file — `summarizeSource` / `extractTasks` | exact |
| `src/hooks/useLibrary.test.ts` | test | request-response | same file — tauri mock + helper typeof asserts | exact |
| `src/lib/tauri.ts` | adapter | request-response | same file — `summarizeSource` / `extractTasks` wrappers | exact |
| `src/App.tsx` | shell | request-response | same file — `handleSummarizeSource` busy wrapper + Library props | exact |
| `src/types/ipc.ts` | model | transform | same file — camelCase IPC result interfaces (e.g. near `InsightsReport`) | role-match |

**Do not modify:** `src-tauri/src/commands/wiki.rs`, `src/types/config.ts` (wiki already round-trips), export UI, `wiki.spec.ts`.

---

## Pattern Assignments

### `src/views/SettingsView.tsx` (view, form-state)

**Analog:** same file — local `AccordionSection` sibling to Lark/Agent; checkbox mutates `setConfig` only; persist via bottom「保存配置」

**Imports pattern** (lines 1–6) — reuse existing; no new imports needed for wiki toggle:
```tsx
import { useState, type ReactNode } from "react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";

import { useJarvisConfig } from "../hooks/useJarvisConfig";
import { formatIndexedAt } from "../lib/sourceDisplay";
import type { AppConfig } from "../types/ipc";
```

**AccordionSection shell** (lines 18–60) — new wiki section uses identical helper (do not invent a second accordion):
```tsx
function AccordionSection({
  testId,
  title,
  defaultExpanded = false,
  children,
}: {
  testId: string;
  title: string;
  defaultExpanded?: boolean;
  children: ReactNode;
}) {
  // ... expanded state + motion.div content
  return (
    <div data-testid={testId} className="rounded-xl border border-white/10">
      {/* title: text-sm font-medium text-zinc-200 */}
      {/* content: space-y-4 border-t border-white/10 p-4 */}
    </div>
  );
}
```

**Sibling placement** (lines 623–625) — insert **after** `settings-section-sync` closes, **before** Lark:
```tsx
      </AccordionSection>

      {/* ADD: AccordionSection testId="settings-section-wiki" title="Wiki 笔记" */}

      <AccordionSection testId="settings-section-lark" title="飞书 / lark-cli">
```

**Checkbox local-state pattern** (lines 595–607) — copy row chrome; nested `wiki` spread (preserve `auto_on_insights`):
```tsx
<label className="flex items-center gap-2 text-zinc-300">
  <input
    type="checkbox"
    checked={config.auto_summarize_on_index}
    onChange={(e) =>
      setConfig({
        ...config,
        auto_summarize_on_index: e.target.checked,
      })
    }
  />
  索引完成后自动生成摘要
</label>
```

**Prescribed wiki section** (CONTEXT D-01..D-03, D-13, D-15, D-16 + UI-SPEC helper):
```tsx
<AccordionSection testId="settings-section-wiki" title="Wiki 笔记">
  <p className="text-xs text-zinc-500">
    开启后可在资料库为已索引来源生成笔记页。关闭后仅隐藏生成入口，已有笔记页仍保留。
  </p>
  <label className="flex items-center gap-2 text-sm text-zinc-300">
    <input
      type="checkbox"
      data-testid="wiki-enabled-toggle"
      checked={config.wiki?.enabled ?? false}
      onChange={(e) =>
        setConfig({
          ...config,
          wiki: {
            enabled: e.target.checked,
            auto_on_insights: config.wiki?.auto_on_insights ?? false,
          },
        })
      }
    />
    启用 Wiki 笔记层
  </label>
</AccordionSection>
```

**Persist pattern** (lines 1182–1191) — do **not** call `handleSaveConfig` from the checkbox:
```tsx
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

**Helper text analog** (Lark section lines 626–632): `text-xs text-zinc-500` paragraph inside Accordion content.

**Do not:** nest under「自动洞察」; show `auto_on_insights`; immediate save; accent/cyan styling on checkbox.

---

### `src/views/LibraryView.tsx` (view, props→UI)

**Analog:** same file — per-source action column for indexed sources (`summarize` / `extract-tasks`)

**Props pattern** (lines 62–76) — add one callback sibling to summarize/extract:
```tsx
export interface LibraryViewProps {
  sources: Source[];
  tasks: TaskItem[];
  busy: boolean;
  indexProgress: IndexProgressView | null;
  config: AppConfig | null;
  cursorCandidates: CursorTranscriptSummary[];
  onRunInsightsAll: () => void;
  onPickAndIndex: () => void;
  onSyncCursorTranscripts: () => void;
  onSummarizeSource: (sourceId: string) => void;
  onExtractTasks: (sourceId: string) => void;
  // ADD: onCompileWiki: (sourceId: string) => void;
  onRetrySource: (sourceId: string) => void;
  onRemoveSource: (sourceId: string) => void;
}
```

**Action column pattern** (lines 202–223) — extend the indexed fragment; keep `btn-ghost text-xs` + `disabled={busy}`:
```tsx
<div className="flex shrink-0 flex-col gap-2">
  {s.status === "indexed" && (
    <>
      <button
        type="button"
        className="btn-ghost text-xs"
        disabled={busy}
        data-testid={`summarize-${s.id}`}
        onClick={() => onSummarizeSource(s.id)}
      >
        生成摘要
      </button>
      <button
        type="button"
        className="btn-ghost text-xs"
        disabled={busy}
        data-testid={`extract-tasks-${s.id}`}
        onClick={() => onExtractTasks(s.id)}
      >
        提取任务
      </button>
      {/* ADD wiki compile button — see gate below */}
    </>
  )}
```

**Visibility gate** (CONTEXT D-05..D-08; RESEARCH §3) — treat missing `wiki` as disabled; never filter `wiki_page` rows out of the list:
```tsx
{config?.wiki?.enabled === true && s.kind !== "wiki_page" && (
  <button
    type="button"
    className="btn-ghost text-xs"
    disabled={busy}
    data-testid={`wiki-compile-${s.id}`}
    onClick={() => onCompileWiki(s.id)}
  >
    生成笔记
  </button>
)}
```

**Existing config usage** (lines 146–157) — `config?.cursor_projects_root` optional chaining is the same null-safe style.

**Do not:** hide `wiki_page` list rows when disabled (D-04); put busy inside the view; invent toast/inline error; use `btn-primary` for compile.

---

### `src/views/LibraryView.test.tsx` (test, props→UI) — **CREATE**

**Analog:** `src/views/ChatView.test.tsx` — jsdom Vitest, mock `motion/react`, render with `baseProps`, assert `data-testid`

**File header + motion mock** (ChatView.test.tsx lines 1–19):
```tsx
/**
 * @vitest-environment jsdom
 */
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { LibraryView } from "./LibraryView";

vi.mock("motion/react", () => ({
  motion: {
    section: ({
      children,
      ...props
    }: React.PropsWithChildren<Record<string, unknown>>) => (
      <section {...props}>{children}</section>
    ),
  },
  useReducedMotion: () => true,
}));
```

**baseProps pattern** — minimal LibraryView props; vary `config.wiki` + source `kind`/`status` per case:
```tsx
const indexedLocal = {
  id: "src-1",
  kind: "local_file",
  uri: "file:///tmp/a.md",
  title: "a.md",
  status: "indexed",
  indexed_at: 1,
  error: null,
  summary: null,
};

const baseProps = {
  sources: [indexedLocal],
  tasks: [],
  busy: false,
  indexProgress: null,
  config: { wiki: { enabled: true, auto_on_insights: false } } as AppConfig,
  cursorCandidates: [],
  onRunInsightsAll: vi.fn(),
  onPickAndIndex: vi.fn(),
  onSyncCursorTranscripts: vi.fn(),
  onSummarizeSource: vi.fn(),
  onExtractTasks: vi.fn(),
  onCompileWiki: vi.fn(),
  onRetrySource: vi.fn(),
  onRemoveSource: vi.fn(),
};
```

**Cases to assert** (RESEARCH Validation Architecture):
| Case | Expect |
|------|--------|
| enabled + indexed local_file | `wiki-compile-src-1` present, text「生成笔记」 |
| `wiki.enabled: false` or missing `wiki` | no `wiki-compile-*` |
| enabled + indexed `wiki_page` | no compile button |
| disabled wiki | `wiki_page` row still renders (kind label path unchanged) |

**Do not:** require full AppConfig fields beyond what LibraryView reads; call real Tauri.

---

### `src/hooks/useLibrary.ts` (hook, request-response)

**Analog:** same file — `summarizeSource` / `extractTasks` (invoke → refreshSources → reportError; **no busy**)

**Imports pattern** (lines 3–14) — add `compileWiki as compileWikiCmd` from `../lib/tauri`:
```ts
import {
  extractTasks as extractTasksCmd,
  indexFile,
  listCursorTranscripts,
  listSources,
  removeSource as removeSourceCmd,
  retrySource as retrySourceCmd,
  runInsightsAll as runInsightsAllCmd,
  summarizeSource as summarizeSourceCmd,
  syncCursorTranscripts as syncCursorTranscriptsCmd,
  // ADD: compileWiki as compileWikiCmd,
} from "../lib/tauri";
```

**Error + refresh pattern** (lines 27–38, 70–80):
```ts
const reportError = useCallback(
  (error: unknown) => {
    onError?.(String(error));
  },
  [onError],
);

const summarizeSource = useCallback(
  async (sourceId: string) => {
    try {
      await summarizeSourceCmd(sourceId);
      await refreshSources();
    } catch (error) {
      reportError(error);
    }
  },
  [refreshSources, reportError],
);
```

**Prescribed `compileWiki`** — discard return value (D-11); same try/catch/refresh:
```ts
const compileWiki = useCallback(
  async (sourceId: string) => {
    try {
      await compileWikiCmd(sourceId);
      await refreshSources();
    } catch (error) {
      reportError(error);
    }
  },
  [refreshSources, reportError],
);
```

**Return object** (lines 127–139) — export `compileWiki` next to `summarizeSource` / `extractTasks`.

**Do not:** expose `busy` / `err` / `indexProgress` from the hook (asserted in tests).

---

### `src/hooks/useLibrary.test.ts` (test, request-response)

**Analog:** same file — `vi.mock("../lib/tauri")` + typeof helper asserts + “does not expose busy”

**Extend mock factory** (lines 10–35):
```ts
vi.mock("../lib/tauri", () => ({
  // ...existing mocks...
  summarizeSource: vi.fn().mockResolvedValue("summary"),
  extractTasks: vi.fn().mockResolvedValue([]),
  compileWiki: vi.fn().mockResolvedValue({
    wikiRoot: "/tmp/wiki",
    pagesWritten: 1,
    created: 1,
    updated: 0,
    skippedUserEdit: 0,
    cleaned: 0,
  }),
  // ...
}));
```

**Extend “returns library domain helpers”** (lines 60–72):
```ts
expect(typeof result.current.compileWiki).toBe("function");
```

**Add invoke path test** (mirror summarize behavior):
```ts
it("compileWiki calls tauri and refreshSources", async () => {
  const { compileWiki: compileWikiCmd } = await import("../lib/tauri");
  const { result } = renderHook(() => useLibrary());
  await act(async () => {
    await result.current.compileWiki("source-1");
  });
  expect(compileWikiCmd).toHaveBeenCalledWith("source-1");
  expect(listSources).toHaveBeenCalled();
});

it("compileWiki reports errors via onError", async () => {
  const { compileWiki: compileWikiCmd } = await import("../lib/tauri");
  vi.mocked(compileWikiCmd).mockRejectedValueOnce(new Error("wiki off"));
  const onError = vi.fn();
  const { result } = renderHook(() => useLibrary({ onError }));
  await act(async () => {
    await result.current.compileWiki("source-1");
  });
  expect(onError).toHaveBeenCalled();
});
```

**Keep** existing `does not expose indexProgress or busy` assertion green after adding `compileWiki`.

---

### `src/lib/tauri.ts` (adapter, request-response)

**Analog:** same file — thin `invoke` wrappers with camelCase args (lines 100–106)

```ts
export function summarizeSource(sourceId: string) {
  return invoke<string>("summarize_source_cmd", { sourceId });
}

export function extractTasks(sourceId: string) {
  return invoke<TaskItem[]>("extract_tasks_cmd", { sourceId });
}
```

**Prescribed wrapper** (RESEARCH §4; Rust `compile_wiki_cmd(source_id)` → FE `{ sourceId }`):
```ts
export function compileWiki(sourceId: string) {
  return invoke<WikiCompileSummary>("compile_wiki_cmd", { sourceId });
}
```

**Import / type:** add `WikiCompileSummary` from `../types/ipc` (or define adjacent if planner colocates — prefer `ipc.ts` for consistency with other IPC DTOs).

**Do not:** reimplement compile pipeline; add new Tauri command registration.

---

### `src/App.tsx` (shell, request-response)

**Analog:** same file — destructure hook helpers, busy wrapper, pass props into `LibraryView`

**Hook destructure** (lines 25–37) — add `compileWiki: libCompileWiki`:
```ts
const {
  sources,
  cursorCandidates,
  refreshSources,
  refreshCursorCandidates,
  retrySource: libRetrySource,
  removeSource: libRemoveSource,
  summarizeSource: libSummarizeSource,
  extractTasks: libExtractTasks,
  // ADD: compileWiki: libCompileWiki,
  runInsightsAll: libRunInsightsAll,
  pickAndIndex: libPickAndIndex,
  syncCursorTranscripts: libSyncCursorTranscripts,
} = useLibrary({ onError: setErr });
```

**Busy wrapper** (lines 232–239) — copy exactly for compile (no task refresh needed unless extract-style side effects appear — wiki only needs sources):
```ts
async function handleSummarizeSource(sourceId: string) {
  setErr(null);
  setBusy(true);
  try {
    await libSummarizeSource(sourceId);
  } finally {
    setBusy(false);
  }
}
```

**Prescribed:**
```ts
async function handleCompileWiki(sourceId: string) {
  setErr(null);
  setBusy(true);
  try {
    await libCompileWiki(sourceId);
  } finally {
    setBusy(false);
  }
}
```

**LibraryView wiring** (lines 351–368):
```tsx
<LibraryView
  // ...existing props...
  onSummarizeSource={(sourceId) =>
    void handleSummarizeSource(sourceId)
  }
  onExtractTasks={(sourceId) => void handleExtractTasks(sourceId)}
  onCompileWiki={(sourceId) => void handleCompileWiki(sourceId)}
  onRetrySource={(id) => void handleRetrySource(id)}
  onRemoveSource={(id) => void handleRemoveSource(id)}
/>
```

**Error path:** `useLibrary({ onError: setErr })` already feeds shared App error bar — no new UI.

**Do not:** put `setBusy` inside the hook; toast on success; ignore `finally` busy clear.

---

### `src/types/ipc.ts` (model, transform) — **optional additive**

**Analog:** same file — camelCase IPC shapes; nested `wiki?: { enabled; auto_on_insights }` already present (line 90)

**Rust source of truth** (`crates/insights/src/wiki.rs` lines 67–76) — `#[serde(rename_all = "camelCase")]`:
```rust
pub struct WikiCompileSummary {
    pub wiki_root: PathBuf,
    pub pages_written: usize,
    pub created: usize,
    pub updated: usize,
    pub skipped_user_edit: usize,
    pub cleaned: usize,
}
```

**Prescribed FE type** (typing only; UI discards value per D-11):
```ts
export interface WikiCompileSummary {
  wikiRoot: string;
  pagesWritten: number;
  created: number;
  updated: number;
  skippedUserEdit: number;
  cleaned: number;
}
```

**Do not:** change `AppConfig.wiki` shape; flatten wiki into top-level flags; invent a second config type in `config.ts`.

---

## Shared Patterns Summary

### Thin view + App busy wrapper (summarize/extract)
**Source:** `useLibrary` → `App.handleSummarizeSource` → `LibraryView` props  
**Apply to:** `compileWiki` / `handleCompileWiki` / `onCompileWiki` /「生成笔记」  
Hook: invoke + `refreshSources` + `reportError`. App: `setErr(null); setBusy(true); try/finally`. View: gated button, `disabled={busy}`.

### Settings local setConfig + bottom save
**Source:** `SettingsView`「自动洞察」checkboxes +「保存配置」  
**Apply to:** `wiki.enabled` toggle only  
Critical: preserve `auto_on_insights` when spreading `wiki`.

### AccordionSection sibling
**Source:** `settings-section-lark` / `settings-section-agent`  
**Apply to:** `settings-section-wiki` titled「Wiki 笔记」 after sync, before Lark.

### Feature-flag visibility
**Source:** RESEARCH §3 + Library `config?.…` optional chaining  
**Apply to:** `config?.wiki?.enabled === true && status === "indexed" && kind !== "wiki_page"`  
Missing `wiki` ≡ disabled. List rows for `wiki_page` stay visible when disabled.

### Tauri invoke wrapper
**Source:** `src/lib/tauri.ts` `summarizeSource`  
**Apply to:** `compileWiki(sourceId)` → `"compile_wiki_cmd"` + `{ sourceId }`

### Vitest view + hook split
**Source:** `ChatView.test.tsx` (visibility) + `useLibrary.test.ts` (invoke)  
**Apply to:** `LibraryView.test.tsx` for D-05..D-08; extend hook test for compile path  
Phase E2E deferred to Phase 13 (document in PLAN).

### Error handling
**Source:** `reportError` → `onError?.(String(error))` → App `setErr`  
**Apply to:** compile failures — shared error bar only (D-10).

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | All Phase 11 files have same-file or adjacent FE analogs. Closest gap: **no existing `LibraryView.test.tsx`** — create from `ChatView.test.tsx` props-render style. Settings Vitest (if planned) similarly has no `SettingsView.test.tsx` yet — prefer thin render test modeled on ChatView, or cover toggle via a focused extract if planner splits component. |

---

## Out of Scope (do not pattern-map into this phase)

| Item | Why |
|------|-----|
| Obsidian zip / export UI | Phase 12 |
| `wiki.spec.ts` full journey | Phase 13 |
| `auto_on_insights` checkbox / auto-compile | CONTEXT deferred |
| Bulk「全部生成笔记」 | Out of scope |
| Success toast / `WikiCompileSummary` UX display | D-11 reject |
| Rust `compile_wiki_cmd` changes | Already complete (Phase 10) |

---

## Metadata

**Analog search scope:** `src/views/{SettingsView,LibraryView,ChatView}.tsx`, `src/hooks/useLibrary.ts` + `.test.ts`, `src/lib/tauri.ts`, `src/App.tsx`, `src/types/{ipc,config}.ts`, `src-tauri/src/commands/wiki.rs`, `crates/insights/src/wiki.rs` (summary shape only)  
**Files scanned:** ~12 primary + CONTEXT/RESEARCH/UI-SPEC  
**Pattern extraction date:** 2026-07-23
