# Phase 6: Settings + Architecture Review - Research

**Researched:** 2026-06-30
**Domain:** React Settings extraction, Rust shell slim, sync error surfacing, architecture sign-off
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### App.tsx & AppShell (FE-01 completion)
- **D-01:** Create **`AppShell.tsx`**: sidebar navigation + global **err banner** + **busy** overlay.
- **D-02:** **`App.tsx` retains view switch only** (`chat` | `library` | `tasks` | `memory` | `settings`) + minimal cross-view coordination — not layout chrome.
- **D-03:** **Line counts are soft targets** (`App.tsx` <300, `lib.rs` <200): record in VERIFICATION with notes if missed; **not a hard gate**.
- **D-04:** Global event listeners → **`useAppEvents.ts`**; App subscribes and passes props (not per-view duplication).
- **D-05:** **Migrate Chat/agent handlers** (`handleAgentAsk`, tool-call state, etc.) from `App.tsx` into **`useChat` / `ChatView`** this phase.
- **D-06:** **No barrel** `src/views/index.ts` — direct imports per Phase 1 D-20.
- **D-07:** **Busy state**: `AppShell` receives `busy` + `setBusy` via React context or props drilled from `App.tsx` (App still coordinates cross-view busy).

#### Settings Extraction (FE-05)
- **D-08:** **`useJarvisConfig` hook-owned state**: config + Lark tokens + api key draft + skills/hooks/plugins + agent CRUD drafts + sync status; App passes **`busy`/`err` only**.
- **D-09:** Settings layout = **accordion sections**; default **expanded: Providers + Index status**; rest collapsed.
- **D-10:** Section order: **Providers → Index → Sync (watch folders + scheduled sync) → Lark → Agent basics → Advanced (Agent CRUD + skills/hooks/plugins)**.
- **D-11:** **Index rebuild / reinit buttons stay in Settings** (`rebuild-index`, `reinit-rebuild-index` testids unchanged).
- **D-12:** **Watch folders** UI lives inside **「同步与监视」** accordion section.
- **D-13:** Agent: **basic agent selection** in main Settings; **CRUD + skills/hooks/plugins** under **「高级」** accordion.
- **D-14:** Config save = **immediate per-field** `set_config` (current behavior); no section/global Save button.
- **D-15:** Add **`settings-section-*`** testids for accordion; **preserve all existing** settings testids (`settings-panel`, `cloud-api-key-input`, etc.).

#### Scheduled Sync Errors (QA-02 / S8)
- **D-16:** Persist **`last_scheduled_sync_error`** (string) + **`last_scheduled_sync_error_at`** (unix timestamp) in `store.meta` on scheduler failure.
- **D-17:** **Partial failure counts**: if `failed > 0` in scheduled run (even when `indexed > 0`), write error summary to meta.
- **D-18:** **Clear meta on next successful** scheduled sync.
- **D-19:** Display error **inline in 「定时同步」accordion** (not global Settings banner).
- **D-20:** **Keep「立即同步」** button; failures show in same section.
- **D-21:** Extend **`SyncStatusView`** with `lastScheduledSyncError` + `lastScheduledSyncErrorAt` (camelCase serde); frontend reads via existing `get_sync_status`.

#### Lark Manual Sync (Phase 3 D-16 follow-through)
- **D-22:** Manual doc/sheet/mail/IM URL inputs → **collapsed「高级 · 手动同步」** with hint: *推荐使用定时同步 / lark-cli*.
- **D-23:** **Primary Lark UX**: detect connection + **立即同步** + scheduled sync toggles.
- **D-24:** **Preserve** `lark-sync-url-input` etc. testids (DOM may be hidden until accordion expanded).
- **D-25:** **Error separation**: manual Lark errors (`larkSyncError`) in Lark section; scheduled errors in sync section.
- **D-26:** **lark-cli path + identity** config stays in main Lark section (not advanced).

#### TypeScript Config (deferred Phase 3)
- **D-27:** **`src/types/config.ts`**: nested `EmbeddingConfig` / `SyncConfig` / `AgentConfig` mirroring Rust; flat IPC compat via serde on Rust side.
- **D-28:** `useJarvisConfig` uses **nested config shape** internally; saves whole config via `set_config`.
- **D-29:** **Vitest roundtrip** test: nested TS types ↔ camelCase IPC payload consistency.

#### Tauri Shell (SHELL-01 completion)
- **D-30:** **`bootstrap.rs`**: `seed_skills_dir`, `seed_hooks_dir`, `seed_plugins_dir`.
- **D-31:** **`events.rs`**: `emit_index_progress`, `emit_index_complete`.
- **D-32:** **`init_state()` → `state.rs`**; `lib.rs` = `run()` + `generate_handler!` registration.
- **D-33:** **`start_ask_e2e` + `is_e2e_mode_cmd` → `commands/chat.rs`**.
- **D-34:** Startup order: **Store → config+migration → keychain → memory URI migration → providers → watcher → scheduler**.

#### Global Error UX
- **D-35:** Global **`err` banner in AppShell** (top); Chat may retain `chat-error` testid for E2E if needed for agent/RAG errors.

#### Architecture Review (QA-02, QA-04)
- **D-36:** Checklist source: **CONCERNS.md + ROADMAP Phase 6 success criteria + REQUIREMENTS** traceability.
- **D-37:** **`06-VERIFICATION.md`**: Phase 5 table format + automated check boxes; mark **FE-01** and **SHELL-01** **Complete**.
- **D-38:** Structural CONCERNS **PASS**; non-structural items → **Known Remaining** table in VERIFICATION (do not block v1.9.6).
- **D-39:** **QA-04 strict**: only refactor, bugfix, tests, docs — accordion/sync-error/Lark-collapse are **in-scope UX**, not new features.
- **D-40:** Phase pass → **v1.9.6** tag target.

#### E2E & Playwright
- **D-41:** **CI gate unchanged**: WebdriverIO + `tauri-driver`; extend **`settings.spec.ts`** (accordion, sync error panel, scheduled sync) + **`full-ui.spec.ts`** Settings journey.
- **D-42:** Introduce **`e2e/playwright/`** with playwright-cli **full-nav smoke + Settings focus**; **not required in CI** this phase.
- **D-43:** Playwright lands **in Phase 6** as supplemental tooling; **WDIO remains authoritative** for merge gate.

#### i18n / Copy
- **D-44:** Settings copy: **concise 中文 technical tone** (consistent with existing UI).

### Claude's Discretion
- Exact accordion section IDs and `settings-section-*` testid suffixes.
- React context vs prop-drill for busy/err through AppShell.
- `SyncStatusView` error string formatting for partial failures.
- `e2e/playwright/` script layout and playwright-cli command wrappers.
- Whether `chat-error` duplicates AppShell err or only agent-specific errors.
- Minor hook internal organization within `useJarvisConfig`.

### Deferred Ideas (OUT OF SCOPE)
- **Playwright as CI gate** — WDIO remains merge authority; revisit post-v1.9.6.
- **react-router** for view navigation — rejected this phase.
- **Complete removal of Lark manual URL inputs** — folded to advanced; removal later if unused.
- **Section-level Save button for Settings** — user chose immediate save.
- **Global Settings banner for sync errors** — user chose inline sync section only.
- **CONCERNS.md full closure** — IM pagination, PDF ingest, FastEmbed CI ignore, etc. → Known Remaining.
- **v2.0.0 / ARCH-01** — after refactor review complete.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| **FE-05** | Settings view extracted with `useJarvisConfig` hook; all config fields editable | `useJarvisConfig` owns config + Lark tokens + API key draft + skills/hooks/plugins + agent CRUD + sync status; `SettingsView.tsx` receives only `busy`/`err`; accordion layout per D-09/D-10; all existing testids preserved; D-27–D-29 `src/types/config.ts` nested types with Vitest roundtrip |
| **QA-02** | Architecture review checklist passes (module boundaries match crate layout, single DB owner, thin shell) | `06-VERIFICATION.md` documents FE-01 + SHELL-01 Complete; structural CONCERNS closed; non-structural items → Known Remaining; traceability matrix against REQUIREMENTS.md |
| **QA-04** | No new user-facing features except small fixes in files already being refactored | Accordion UI, sync error surface, Lark collapse are locked refactor-scope UX; no new Tauri commands, no new crates, no new routes |
</phase_requirements>

---

## Summary

Phase 6 completes the v1.9.x structural refactor by closing the two remaining requirements (FE-01 and SHELL-01) plus delivering FE-05, QA-02, and QA-04. The work is **entirely refactor-scope**: no new crates, no SQLite schema changes (only two new `meta` keys), and no new Tauri IPC commands.

**Codebase state entering Phase 6 [VERIFIED: codebase]:**

- `src/App.tsx` — **2,034 lines** (CONTEXT.md noted ~1,961; now 2,034 after Phase 5 additions). Settings JSX is inline from ~line 1,051 to ~2,034; `handleAgentAsk`, `handleRagAsk`, e2e ask wrappers, tool-call state all still in App.tsx.
- `src-tauri/src/lib.rs` — **315 lines** (CONTEXT noted ~292; Phase 5 added imports). Contains: `emit_index_progress`, `emit_index_complete`, `is_e2e_mode_cmd`, `start_ask_e2e`, seed functions (`seed_skills_dir`, `seed_hooks_dir`, `seed_plugins_dir`), `init_state()`, `ask_question`, and `run()` + handler registration.
- `src-tauri/src/state.rs` — already exists and owns `AppState` — SHELL-01 is **partially complete** (`AppState` is already in `state.rs`; remaining: move `init_state()` function body there and extract seed/emit functions).
- `src-tauri/src/sync_scheduler.rs` — **182 lines**: `run_scheduled_sync` already writes `last_scheduled_sync_at` to meta; partial failures (`failed > 0`) are **silently ignored** — only the `Err(e)` case is swallowed in the scheduler loop via `let _ = rt.block_on(...)`. `SyncStatusView` has no error fields.
- `src/views/SettingsView.tsx` — stub passthrough (5 lines, renders children).
- `src/hooks/` — `useChat`, `useLibrary`, `useMemory`, `useTasks` exist; **no `useJarvisConfig`**.
- `src/types/ipc.ts` — flat `AppConfig` (not nested); `SyncStatusView` has no error fields.
- `e2e/specs/settings.spec.ts` — 3 tests: index health, rebuild button, API key UI.

**Three vertical slices for planning:**
1. **Settings extraction** — `useJarvisConfig`, `SettingsView` accordion, `src/types/config.ts`, `useAppEvents`, `AppShell`, Chat handler migration to `useChat`/`ChatView`.
2. **Sync error surface** — `sync_scheduler` meta writes, `SyncStatusView` extension, `SyncStatusView` IPC types, Settings accordion inline display.
3. **Shell slim + architecture sign-off** — `bootstrap.rs`, `events.rs`, move `init_state` body, `start_ask_e2e` + `is_e2e_mode_cmd` → `commands/chat.rs`, E2E extension, `06-VERIFICATION.md`.

**Primary recommendation:** Implement as three sequential waves matching the three slices above; test after each wave; architecture sign-off wave happens last when structural metrics are measurable.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Settings UI (accordion, form fields) | **Browser / Client** (`SettingsView.tsx`) | `useJarvisConfig` hook | Pure presentational; hook owns all state |
| Config state + all settings actions | **Browser / Client** (`useJarvisConfig.ts`) | App.tsx (busy/err only) | Hook pattern mirrors `useChat`, `useLibrary` |
| AppShell layout + nav + err banner | **Browser / Client** (`AppShell.tsx`) | App.tsx (busy/err props) | Layout chrome extracted from App |
| Global event listeners (index-progress) | **Browser / Client** (`useAppEvents.ts`) | App.tsx consumer | One listener, passed as props into views |
| Sync error persistence | **API / Backend** (`sync_scheduler.rs`) | `crates/store` meta | Background thread writes; frontend polls |
| SyncStatusView IPC payload | **API / Backend** (`commands/sync.rs`) | `sync_scheduler.rs` | Thin command reads meta + config |
| TypeScript nested config types | **Browser / Client** (`src/types/config.ts`) | `src/types/ipc.ts` | TS-side mirroring of Rust nested structs |
| Shell bootstrap (seed dirs) | **API / Backend** (`bootstrap.rs`) | `lib.rs` caller | Called once from `init_state` startup |
| Shell event emission | **API / Backend** (`events.rs`) | `lib.rs`, `sync_scheduler.rs`, etc. | Single emit utility |
| AppState init | **API / Backend** (`state.rs` new fn) | `lib.rs` caller | `init_state()` body moves into state.rs |
| E2E commands | **API / Backend** (`commands/chat.rs`) | `lib.rs` registration only | `start_ask_e2e` + `is_e2e_mode_cmd` |
| Architecture sign-off | — | `06-VERIFICATION.md` | Document artifact; no code tier |

---

## Standard Stack

### Core (no new packages required)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React 19 | `^19.1.0` | `useJarvisConfig` hook, `AppShell`, `SettingsView` accordion | Project standard |
| `motion/react` | `^12` | Accordion height animation (300ms ease-out) | Already used in App.tsx |
| `@tabler/icons-react` | `^3.44` | Accordion chevron / section icons | Project icon library |
| Vitest 3 | `^3` | Roundtrip TS config type test (D-29) | Project test standard |
| WebdriverIO 9 | `^9.20.0` | E2E settings.spec.ts accordion + sync error tests | CI gate |
| `rusqlite` | `0.32` (workspace) | `store.set_meta` / `get_meta` for sync error keys | Already used in store |
| `serde` / `serde_json` | `1` (workspace) | `SyncStatusView` extension; camelCase IPC | Already used |
| `thiserror` | `1` (workspace) | Error handling in bootstrap/events if needed | Project pattern |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `playwright` (CLI via npm) | `^1.x` | Supplemental nav smoke in `e2e/playwright/` | Dev/QA only — NOT CI gate |
| `tempfile` | dev-dep | Isolated DB for integration tests | Existing pattern |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Prop-drill for `busy`/`err` | React Context | Context adds abstraction; prop-drill simpler for 1 level (App → AppShell); Claude's discretion |
| Accordion from scratch | headlessui or radix | Added dependency; `motion/react` AnimatePresence + useState sufficient per existing pattern |
| New Tauri command for sync errors | Extend existing `get_sync_status` | **Locked D-21**: use existing command; avoid IPC rename |

**Installation:** None — no new packages for this phase. [VERIFIED: codebase + CONTEXT]

---

## Package Legitimacy Audit

> Phase installs **no new external packages**. Refactor uses existing workspace dependencies only. The `playwright` CLI mentioned in D-42 is installed as a dev tool only if not already present — no new npm package needed in `package.json` dependencies if `@playwright/test` is already in devDependencies or added as a one-time CLI install.

| Package | Registry | Verdict | Disposition |
|---------|----------|---------|-------------|
| *(none new)* | — | — | N/A |

**Packages removed due to [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

---

## Architecture Patterns

### System Architecture Diagram

```
User navigates to Settings
        │
        ▼
┌─────────────────────────────────────────┐
│  App.tsx (view router)                  │
│  view === "settings" → <SettingsView>   │
│  busy / err / setBusy / setErr passed   │
└─────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────┐
│  AppShell.tsx                           │
│  sidebar nav + err banner + busy overlay│
└─────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────┐
│  SettingsView.tsx                       │
│  useJarvisConfig() →                   │
│    config, syncStatus, skills, hooks,  │
│    plugins, larkTokens, apiKeyDraft,   │
│    agentDrafts                          │
│  Accordion sections (motion/react)      │
│  • Providers   [expanded default]       │
│  • Index       [expanded default]       │
│  • Sync        [collapsed] ─────────┐  │
│  • Lark        [collapsed]          │  │
│  • Agent       [collapsed]          │  │
│  • Advanced    [collapsed]          │  │
└─────────────────────────────────────────┘
        │ Sync section                 │
        ▼                              │
┌─────────────────┐   (on error)       │
│ sync_scheduler  │──────────────────► │
│ (Rust thread)   │  set_meta(         │
│                 │    last_scheduled_ │
│                 │    sync_error)     │
│                 │    last_scheduled_ │
│                 │    sync_error_at)  │
└─────────────────┘                    │
        │ get_sync_status              │
        ▼                              ▼
┌─────────────────────────────────────────┐
│  commands/sync.rs                       │
│  SyncStatusView { ..., lastScheduled   │
│    SyncError, lastScheduledSyncErrorAt }│
└─────────────────────────────────────────┘
```

### Recommended Project Structure (after Phase 6)

```
src/
├── App.tsx                    # <300 lines: view router + minimal cross-view coordination
├── AppShell.tsx               # NEW: sidebar nav + err banner + busy overlay
├── hooks/
│   ├── useAppEvents.ts        # NEW: index-progress / jarvis-ask-done listeners
│   ├── useChat.ts             # EXTENDED: handleAgentAsk, handleRagAsk migrated in
│   ├── useJarvisConfig.ts     # NEW: all settings state + actions
│   ├── useLibrary.ts
│   ├── useMemory.ts
│   └── useTasks.ts
├── types/
│   ├── config.ts              # NEW: nested EmbeddingConfig / SyncConfig / AgentConfig
│   ├── ipc.ts                 # EXTENDED: SyncStatusView + error fields
│   ├── library.ts
│   ├── tasks.ts
│   └── view.ts
└── views/
    ├── ChatView.tsx           # EXTENDED: receives handleAgentAsk props from useChat
    ├── LibraryView.tsx
    ├── MemoryView.tsx
    ├── SettingsView.tsx       # FULL EXTRACTION: accordion layout with 6 sections
    └── TasksView.tsx

src-tauri/src/
├── lib.rs                     # ~100 lines: run() + generate_handler! only
├── bootstrap.rs               # NEW: seed_skills_dir, seed_hooks_dir, seed_plugins_dir
├── events.rs                  # NEW: emit_index_progress, emit_index_complete
├── state.rs                   # EXTENDED: init_state() body moved here
├── e2e.rs                     # unchanged
├── index_ops.rs               # unchanged
├── insights_ops.rs            # unchanged
├── sync_scheduler.rs          # EXTENDED: error meta writes, SyncStatusView fields
└── commands/
    ├── agent.rs               # unchanged (Phase 5)
    ├── chat.rs                # EXTENDED: start_ask_e2e + is_e2e_mode_cmd moved here
    ├── config.rs              # unchanged
    ├── index.rs               # unchanged
    ├── lark.rs                # unchanged
    ├── library.rs             # unchanged
    ├── memory.rs              # unchanged
    ├── mod.rs                 # updated re-exports
    └── sync.rs                # unchanged (SyncStatusView updated in sync_scheduler.rs)

e2e/
├── specs/settings.spec.ts     # EXTENDED: accordion, sync error, scheduled sync
├── specs/full-ui.spec.ts      # EXTENDED: Settings journey
└── playwright/                # NEW (supplemental, NOT CI): nav smoke + settings focus
```

### Pattern 1: Controlled settings hook (mirrors useChat/useLibrary)

**What:** `useJarvisConfig` owns all settings state; SettingsView is purely presentational.
**When to use:** Every settings-related piece of state and action.

**Example:**

```typescript
// Pattern from src/hooks/useChat.ts [VERIFIED: codebase] — mirror exactly
export interface UseJarvisConfigOptions {
  onError?: (message: string) => void;
  busy: boolean;
  setBusy: (v: boolean) => void;
}

export function useJarvisConfig({ onError, busy, setBusy }: UseJarvisConfigOptions) {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [syncStatus, setSyncStatus] = useState<SyncStatusView | null>(null);
  const [larkDocToken, setLarkDocToken] = useState("");
  // ... all settings-local state

  const refreshConfig = useCallback(async () => {
    const cfg = await invoke<AppConfig>("get_config");
    setConfig(cfg);
  }, []);

  const handleSaveConfig = useCallback(async (next: AppConfig) => {
    setBusy(true);
    try {
      await invoke("set_config", { config: { ...next, cloud_api_key: "" } });
      setConfig({ ...next, cloud_api_key: "" });
    } catch (e) { onError?.(String(e)); }
    finally { setBusy(false); }
  }, [setBusy, onError]);

  return { config, syncStatus, refreshConfig, handleSaveConfig, ... };
}
```

### Pattern 2: Accordion section with motion/react

**What:** Collapsible settings section with AnimatePresence + useReducedMotion.
**When to use:** Each of the 6 accordion sections in SettingsView.

**Example:**

```typescript
// Based on existing motion usage in App.tsx [VERIFIED: codebase]
// src/views/SettingsView.tsx — AccordionSection internal component
function AccordionSection({
  id, title, defaultExpanded = false, children
}: { id: string; title: string; defaultExpanded?: boolean; children: React.ReactNode }) {
  const [open, setOpen] = useState(defaultExpanded);
  const reduceMotion = useReducedMotion();

  return (
    <div data-testid={`settings-section-${id}`} className="rounded-xl border border-white/10">
      <button
        type="button"
        className="flex w-full items-center justify-between px-4 py-3 text-sm font-medium"
        onClick={() => setOpen(o => !o)}
        aria-expanded={open}
      >
        {title}
        <IconChevronDown className={`size-4 transition-transform ${open ? "rotate-180" : ""}`} />
      </button>
      <AnimatePresence initial={false}>
        {open && (
          <motion.div
            key="content"
            initial={reduceMotion ? false : { height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={reduceMotion ? undefined : { height: 0, opacity: 0 }}
            transition={{ duration: 0.3, ease: "easeOut" }}
            style={{ overflow: "hidden" }}
          >
            <div className="px-4 pb-4">{children}</div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
```

### Pattern 3: sync_scheduler error meta write

**What:** On partial or total failure in `run_scheduled_sync`, write error string + timestamp to `store.meta`.
**When to use:** Any `failed > 0` condition OR returned `Err`.

**Example:**

```rust
// Extension to sync_scheduler.rs [VERIFIED: codebase — currently only writes success]
// After combined report computation:
if combined.failed > 0 || result.is_err() {
    let summary = format!("indexed {}, failed {}", combined.indexed, combined.failed);
    let _ = state.store.set_meta("last_scheduled_sync_error", &summary);
    let _ = state.store.set_meta("last_scheduled_sync_error_at", &now.to_string());
} else {
    // Clear on success (D-18)
    let _ = state.store.set_meta("last_scheduled_sync_error", "");
    let _ = state.store.set_meta("last_scheduled_sync_error_at", "");
}
```

### Pattern 4: SyncStatusView extension (camelCase serde)

**What:** Add two optional fields to `SyncStatusView`; read from meta in `sync_status_view`.
**When to use:** Extending the existing IPC DTO.

**Example:**

```rust
// sync_scheduler.rs [VERIFIED: codebase — current struct has 3 fields]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatusView {
    pub last_scheduled_sync_at: Option<i64>,
    pub scheduled_sync_enabled: bool,
    pub scheduled_sync_interval_minutes: u32,
    // NEW: D-21
    pub last_scheduled_sync_error: Option<String>,
    pub last_scheduled_sync_error_at: Option<i64>,
}
```

### Pattern 5: Thin command registration (lib.rs after slim)

**What:** `lib.rs` = `run()` + handler macro only; all logic in modules.
**When to use:** Final `lib.rs` state after bootstrap.rs / events.rs / state.rs extraction.

**Example:**

```rust
// lib.rs after phase — mirrors Phase 2/3/4/5 approach [VERIFIED: codebase pattern]
mod commands;
mod e2e;
mod index_ops;
mod insights_ops;
mod state;
mod sync_scheduler;
mod bootstrap;    // NEW
mod events;       // NEW

pub(crate) use state::AppState;
pub(crate) use events::{emit_index_progress, emit_index_complete};

use commands::{ ... }; // all registered commands

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let state = state::init_state(app)?;  // moved to state.rs
            // ... watcher, scheduler
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![ ... ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Pattern 6: Nested TypeScript config types

**What:** Mirror Rust nested `AppConfig` (`EmbeddingConfig`, `SyncConfig`, `AgentConfig`) in TS.
**When to use:** Internal `useJarvisConfig` state; IPC boundary uses flat `AppConfig` for backward compat.

**Example:**

```typescript
// src/types/config.ts — NEW file
export interface EmbeddingConfig {
  embedder: "mock" | "ollama" | "fast_embed" | "cloud";
  mockEmbedDim: number;
  ollamaEmbedModel: string;
  ollamaEmbedDim: number;
  fastembedModel: string;
  fastembedDim: number;
  cloudEmbedModel: string;
  cloudEmbedDim: number;
}

export interface SyncConfig {
  watchFolders: string[];
  scheduledSyncEnabled: boolean;
  scheduledSyncIntervalMinutes: number;
  scheduledSyncWatchFolders: boolean;
  scheduledSyncCursor: boolean;
  scheduledSyncLark: boolean;
  cursorProjectsRoot: string;
}

// Vitest roundtrip test (D-29): verify camelCase field names match IPC payload
```

### Anti-Patterns to Avoid

- **Duplicating event listeners per view:** `useAppEvents.ts` owns `index-progress` + `jarvis-ask-done`; don't put these in SettingsView or ChatView.
- **Renaming existing Tauri IPC commands during refactor:** `get_sync_status` stays; fields are added, not renamed.
- **Creating a barrel `src/views/index.ts`:** Locked decision D-06.
- **Putting `init_state` in a new file rather than appending to `state.rs`:** D-32 specifies `state.rs`; `AppState` is already there.
- **Missing `#[serde(default)]` on new SyncStatusView fields:** Frontend deserializes from existing endpoint; old binary has null fields → requires serde default.
- **Global err banner for scheduled sync errors:** D-19 specifies inline in sync accordion only.
- **Hard-deleting Lark manual sync inputs:** D-22 folds them into collapsed accordion; testids preserved per D-24.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Accordion animation | CSS height hack or `max-height` hack | `motion/react` `AnimatePresence` with `height: "auto"` | Already in project; handles height: auto correctly which CSS can't |
| Accordion state management | Redux/Zustand | `useState` per section | Simplest; no cross-section sync needed |
| Sync error persistence | Custom error table | `store.set_meta` / `get_meta` | Already used for `last_scheduled_sync_at`; consistent pattern |
| Config type mapping TS↔Rust | Code generator | Manual mirroring + Vitest roundtrip test | D-27 explicit; project has no codegen tooling |
| Settings form save button | Global save | Immediate per-field `set_config` invoke | D-14 locked; matches existing behavior |

**Key insight:** The complexity in this phase is **structural surgery on a 2,034-line file** — the risk is regression, not algorithm design. The pattern (extract to hook, pass props to view) is proven from 4 prior phases.

---

## Runtime State Inventory

> This is a structural refactor phase. Required section.

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| **Stored data** | `store.meta` currently has: `last_scheduled_sync_at` (timestamp string), `embedder_id`, `vector_dim` | **Code edit only**: add two new keys `last_scheduled_sync_error` (string) + `last_scheduled_sync_error_at` (timestamp string); no migration needed for new keys (will be null until first scheduler error) |
| **Stored data** | `config.json` — flat `AppConfig`; nested Rust structs already serialize flat via serde flatten | **No migration**: Rust side already has nested structs with `#[serde(flatten)]`; TS types are the only change |
| **Stored data** | `chat_messages`, `sources`, `chunks`, `tasks` — no changes | **None** |
| **Live service config** | None — refactor is in-app only | None — verified by codebase audit |
| **OS-registered state** | None | None — verified |
| **Secrets/env vars** | `JARVIS_E2E=1`, `JARVIS_E2E_FIXTURE`, `JARVIS_E2E_APP` — unchanged | **Do not change** E2E env contract; `start_ask_e2e` command moves to `commands/chat.rs` but invoke name stays identical |
| **Build artifacts** | `target/release/tauri-app.exe` for E2E | Rebuild after Rust changes — standard CI path via `npm run build:e2e` |

**Canonical post-refactor question:** After all files updated, what runtime systems still have old state? Answer: `store.meta` gains two new keys (additive, no migration). `config.json` schema unchanged. E2E invoke names unchanged. All testids preserved.

---

## Common Pitfalls

### Pitfall 1: Breaking existing settings testids during accordion extraction

**What goes wrong:** Moving Settings JSX into accordion sections changes DOM structure; `$('[data-testid="cloud-api-key-input"]`)` can't find element if it's inside a collapsed accordion in E2E.
**Why it happens:** WdIO `toBeDisplayed()` fails for hidden (collapsed) DOM elements.
**How to avoid:** Default-expand Providers + Index sections (D-09); `lark-sync-url-input` etc. are in collapsed Lark/Advanced — E2E must expand accordion before asserting, or check `toExist()` not `toBeDisplayed()`.
**Warning signs:** `settings.spec.ts` existing 3 tests fail after extraction without accordion interaction.

### Pitfall 2: `handleAgentAsk` migration breaks E2E `start_ask_e2e` path

**What goes wrong:** Moving `handleAgentAsk` logic into `useChat` accidentally removes the `e2eMode` + `jarvis-ask-done` listener branch, breaking `agent.spec.ts`.
**Why it happens:** The E2E branch in `handleAgentAsk` listens for `jarvis-ask-done` event — this must stay functional after migration.
**How to avoid:** Move handler wholesale first; verify E2E passes before any simplification. Keep `askHandleCount` and `e2eMode` wiring; they're E2E-critical.
**Warning signs:** `agent.spec.ts` times out on ask; `chat-session-ready` testid present but no response.

### Pitfall 3: `init_state()` move breaks E2E fixture seeding

**What goes wrong:** `init_state` calls `seed_e2e_fixture` after setup — if the call order changes during refactor, E2E mode starts with empty DB.
**Why it happens:** `seed_e2e_fixture` depends on `AppState` being fully initialized AND managed before seeding.
**How to avoid:** D-34 specifies startup order explicitly; replicate it exactly in the moved `state::init_state()` function. Verify by running `npm run test:e2e:local` after Rust changes.
**Warning signs:** `smoke.spec.ts` fails; fixture not seeded; index-status shows 0 sources.

### Pitfall 4: SyncStatusView `#[serde(default)]` missing causes frontend parse error

**What goes wrong:** Adding `last_scheduled_sync_error` + `last_scheduled_sync_error_at` to `SyncStatusView` without `#[serde(default)]` / `Option` causes deserialization failure on frontends reading from old binaries (during development iteration).
**Why it happens:** Rust `serde` requires all fields present unless `#[serde(default)]` is set.
**How to avoid:** Both new fields are `Option<T>` — serde will serialize them as `null` → frontend uses `?? null` safely.
**Warning signs:** `get_sync_status` invoke fails with "missing field" in console.

### Pitfall 5: `useJarvisConfig` re-renders on every App render

**What goes wrong:** Config refresh callbacks passed as props to `SettingsView` without `useCallback` memoization cause infinite render loops.
**Why it happens:** New function reference on every render → useEffect dependency array changes.
**How to avoid:** Wrap all `useJarvisConfig` callbacks in `useCallback` with correct deps; match pattern from `useChat.ts` (line 30+) exactly.
**Warning signs:** Browser console warns "Too many re-renders"; settings UI flickers.

### Pitfall 6: `commands/chat.rs` handler registration gap after moving `start_ask_e2e`

**What goes wrong:** `start_ask_e2e` and `is_e2e_mode_cmd` moved to `commands/chat.rs` but `generate_handler!` in `lib.rs` not updated → runtime "command not found" for E2E.
**Why it happens:** Manual registration list in `generate_handler!` not updated.
**How to avoid:** Move + update `commands/mod.rs` re-exports + `lib.rs` handler list atomically in one task; run `cargo build` to catch missing registration at compile time.
**Warning signs:** E2E driver hangs; `start_ask_e2e` invoke returns error.

### Pitfall 7: Accordion DOM hidden elements fail existing WdIO assertions

**What goes wrong:** `settings.spec.ts` line checking `toBeDisplayed()` on `rebuild-index` passes today with flat layout; fails when rebuild button is inside collapsed accordion.
**Why it happens:** WdIO `toBeDisplayed()` checks CSS visibility — collapsed sections are hidden.
**How to avoid:** Index section defaults **expanded** (D-09); `rebuild-index` must remain visible on load. Verify test immediately after extraction.
**Warning signs:** `it("exposes rebuild index control")` fails.

---

## Code Examples

### Current `sync_status_view` (to extend)

```rust
// src-tauri/src/sync_scheduler.rs lines 169-182 [VERIFIED: codebase]
pub fn sync_status_view(state: &AppState) -> SyncStatusView {
    let cfg = state.config();
    let last = state
        .store
        .get_meta("last_scheduled_sync_at")
        .ok()
        .flatten()
        .and_then(|s| s.parse().ok());
    SyncStatusView {
        last_scheduled_sync_at: last,
        scheduled_sync_enabled: cfg.sync.scheduled_sync_enabled,
        scheduled_sync_interval_minutes: cfg.sync.scheduled_sync_interval_minutes,
    }
}
```

### Current `run_scheduled_sync` error handling (gap to fix)

```rust
// src-tauri/src/sync_scheduler.rs line 68 [VERIFIED: codebase]
// PROBLEM: errors discarded
let _ = rt.block_on(run_scheduled_sync(&app, &state));
// FIX: capture error, write to meta
if let Err(e) = rt.block_on(run_scheduled_sync(&app, &state)) {
    let _ = state.store.set_meta("last_scheduled_sync_error", &e);
    let _ = state.store.set_meta("last_scheduled_sync_error_at", &unix_now().to_string());
}
```

### Current `SyncStatusView` IPC type (to extend)

```typescript
// src/types/ipc.ts [VERIFIED: codebase] — current shape, no error fields
export interface SyncStatusView {
  lastScheduledSyncAt?: number | null;
  scheduledSyncEnabled: boolean;
  scheduledSyncIntervalMinutes: number;
  // NEED TO ADD:
  // lastScheduledSyncError?: string | null;
  // lastScheduledSyncErrorAt?: number | null;
}
```

### Existing settings testids (must all be preserved)

```
settings-panel           ← main wrapper
index-status             ← index health div
rebuild-index            ← rebuild button  
reinit-rebuild-index     ← reinit+rebuild button (conditional)
cloud-api-key-input      ← password input
clear-api-key            ← clear button
lark-check-connection    ← Lark check button
lark-detect-cli          ← auto-detect button
lark-status-panel        ← Lark status display
lark-sync-url-input      ← manual doc URL input
lark-sync-submit         ← manual sync button
cursor-projects-root     ← Cursor root input
pipeline-mode-toggle     ← pipeline radio
router-mode-toggle       ← router radio
run-scheduled-sync       ← immediate sync button
```

### `useJarvisConfig` settings-local state inventory (from App.tsx)

```typescript
// [VERIFIED: codebase — src/App.tsx lines 113-142]
// All of these move into useJarvisConfig:
const [indexStatus, setIndexStatus] = useState<IndexStatusView | null>(null);
const [config, setConfig] = useState<AppConfig | null>(null);
const [larkDocToken, setLarkDocToken] = useState("");
const [larkSheetToken, setLarkSheetToken] = useState("");
const [larkMailId, setLarkMailId] = useState("");
const [larkChatId, setLarkChatId] = useState("");
const [larkStatus, setLarkStatus] = useState<LarkAuthStatus | null>(null);
const [larkSyncError, setLarkSyncError] = useState<string | null>(null);
const [hasApiKey, setHasApiKey] = useState(false);
const [apiKeyDraft, setApiKeyDraft] = useState("");
const [newWatchFolder, setNewWatchFolder] = useState("");
const [syncStatus, setSyncStatus] = useState<SyncStatusView | null>(null);
const [skills, setSkills] = useState<SkillItem[]>([]);
const [hooks, setHooks] = useState<HookItem[]>([]);
const [plugins, setPlugins] = useState<PluginItem[]>([]);
// Agent drafts (newAgentId, newAgentName, etc.) also move in
```

### Agent/chat state that stays in App.tsx or moves to useChat

```typescript
// [VERIFIED: codebase — stays App.tsx or moves to useChat]
// Move to useChat:
const [lastToolCalls, setLastToolCalls] = useState<ToolCallInfo[]>([]);
const [lastToolParseWarnings, setLastToolParseWarnings] = useState<string[]>([]);
const [lastOrchestrationSteps, setLastOrchestrationSteps] = useState<...>([]);
const [agentMode, setAgentMode] = useState(true);
const [e2eMode, setE2eMode] = useState(false);
const [askHandleCount, setAskHandleCount] = useState(0);
// handleAgentAsk, handleRagAsk, handleAsk → moved to useChat or new hook
```

### `lib.rs` seeds to extract (current location)

```rust
// src-tauri/src/lib.rs lines 96-165 [VERIFIED: codebase]
// Move to bootstrap.rs:
fn seed_skills_dir(dir: &PathBuf) -> Result<(), String> { ... }
fn seed_hooks_dir(dir: &PathBuf) -> Result<(), String> { ... }
fn seed_plugins_dir(dir: &PathBuf) -> Result<(), String> { ... }

// Move to events.rs:
pub(crate) fn emit_index_progress(app: &AppHandle, event: IndexProgressEvent) { ... }
pub(crate) fn emit_index_complete(app: &AppHandle, report: &RebuildReport) { ... }

// Move to state.rs (append):
fn init_state(app: &tauri::App) -> Result<AppState, String> { ... }

// Move to commands/chat.rs:
#[tauri::command]
fn is_e2e_mode_cmd() -> bool { ... }
#[tauri::command]
fn start_ask_e2e(...) -> Result<(), String> { ... }
// struct AskDonePayload
// ask_question command (or keep in lib.rs — minimal)
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Settings inline in App.tsx (~1000 lines) | `SettingsView` + `useJarvisConfig` | Phase 6 | FE-05 complete |
| Flat `AppConfig` in TS | Nested `EmbeddingConfig`/`SyncConfig`/`AgentConfig` in `src/types/config.ts` | Phase 6 | D-27/28/29 |
| Scheduler errors silently discarded | Error persisted to `store.meta` + surfaced in Settings | Phase 6 | QA-02 structural concern resolved |
| `emit_*` + `seed_*` + `init_state` in `lib.rs` | Extracted to `events.rs`, `bootstrap.rs`, `state.rs` | Phase 6 | SHELL-01 complete |
| `start_ask_e2e` in `lib.rs` | `commands/chat.rs` | Phase 6 | D-33; invoke name unchanged |
| AppState defined + initialized in lib.rs | Defined in `state.rs` (Phase 1); `init_state` body moves here Phase 6 | Phase 6 | SHELL-01 complete |

**Deprecated/outdated:**
- Inline Settings JSX in `App.tsx` — superseded by `SettingsView` extraction.
- Global `err` banner owned by App.tsx — moves to `AppShell.tsx`.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `AppState` is already in `state.rs` so SHELL-01 is partially complete; only `init_state()` body + emission/seed functions remain in `lib.rs` | Runtime Inventory | If `init_state` is already fully in `state.rs`, the extraction task is trivial; verified it's still in `lib.rs` [VERIFIED: codebase] |
| A2 | Motion/react `height: "auto"` animate works with `AnimatePresence` for accordion without additional config | Architecture Patterns | Accordion height collapses wrong; need `overflow: hidden` wrapper (included in example) |
| A3 | Playwright CLI (`@playwright/test`) is NOT yet in project devDependencies | Package Audit | If already installed, no npm action needed for D-42 |

**Note on A1:** Confirmed `init_state()` is at line ~167 in `lib.rs` [VERIFIED: codebase]. `AppState` struct is already in `state.rs` [VERIFIED]. Gap = `init_state()` body + seed functions + emit functions.

---

## Open Questions (RESOLVED)

1. **React context vs prop-drill for `busy`/`err` through AppShell**
   - What we know: D-07 says "React context or props drilled from App.tsx"; Claude's discretion.
   - What's unclear: Context adds `AppShell.Provider` wrapper complexity; prop-drill is simpler for 1 level.
   - Recommendation: Prop-drill — `AppShell` receives `{ busy, setBusy, err, setErr, view, setView, children }`; simplest and matches existing pattern. Use context only if 3+ levels of drilling emerge.

2. **`chat-error` testid duplication**
   - What we know: D-35 says Chat may retain `chat-error` for agent/RAG errors; global err banner in AppShell.
   - What's unclear: Whether `chat-error` (in ChatView) duplicates AppShell banner or is exclusive.
   - Recommendation: `chat-error` in ChatView shows only RAG/agent-specific failures from the last ask; AppShell `err` banner shows system-level errors (index, config, etc.). Clear semantic separation prevents double-banner.

3. **Playwright version for supplemental E2E**
   - What we know: D-42 requires `e2e/playwright/` nav smoke; NOT CI gate.
   - What's unclear: Whether `@playwright/test` should be added to `package.json` or used via `npx`.
   - Recommendation: Add `@playwright/test` to devDependencies (latest stable) — single explicit dep is cleaner than `npx`. Verify exact version at implementation.

---

## Environment Availability

Step 2.6: **SKIPPED** — no new external tool dependencies identified. Phase uses existing Rust stable + Node 22 + WebdriverIO + Tauri driver toolchain.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (workspace); Vitest 3; WebdriverIO 9 + Mocha |
| Config file | `vite.config.ts` (Vitest); `e2e/wdio.conf.ts` (WdIO) |
| Quick run command | `npm test` (Vitest only) |
| Full suite command | `cargo test --workspace && npm test && npm run test:e2e:local` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FE-05 | `useJarvisConfig` hook returns config and actions | unit | `npm test -- useJarvisConfig` | ❌ Wave 0 |
| FE-05 | TS nested config roundtrip (camelCase IPC compat) | unit (Vitest) | `npm test -- config` | ❌ Wave 0 (`src/types/config.test.ts`) |
| FE-05 | Settings accordion renders 6 sections | E2E | `npm run test:e2e:local -- --spec e2e/specs/settings.spec.ts` | ✅ spec exists, need extension |
| FE-05 | Preserved testids accessible in accordion | E2E | `npm run test:e2e:local -- --spec e2e/specs/settings.spec.ts` | ✅ (need expansion) |
| FE-05 | Sync error visible in sync section | E2E | `npm run test:e2e:local -- --spec e2e/specs/settings.spec.ts` | ❌ Wave 0 (new test) |
| QA-02 | Architecture checklist documented and pass | manual + doc | `06-VERIFICATION.md` pass | ❌ Wave 0 (new doc) |
| QA-02 | `cargo test --workspace` green after lib.rs slim | compile + test | `cargo test --workspace` | ✅ CI path |
| QA-02 | `npm run test:e2e:local` green after all changes | E2E | `npm run test:e2e:local` | ✅ CI path |
| QA-04 | No new user-facing features (refactor-only) | code review | Manual — check diff for new Tauri commands / new routes | manual |
| QA-04 | `npx tsc --noEmit` passes | compile | `npx tsc --noEmit` | ✅ already in CI path |
| SHELL-01 | `lib.rs` contains only `run()` + handler registration | compile + review | `cargo build -p tauri-app` + line count | ❌ Wave 0 (extraction) |
| FE-01 | `App.tsx` is view router + coordination only | compile + review | `npx tsc --noEmit` + line count | ❌ Wave 0 (extraction) |

### Sampling Rate

- **Per task commit:** `cargo test -p tauri-app` (Rust) or `npm test` (TS)
- **Per wave merge:** `cargo test --workspace && npm test && npx tsc --noEmit`
- **Phase gate:** `cargo test --workspace && npm test && npm run test:e2e:local` before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `src/hooks/useJarvisConfig.ts` + unit test — covers FE-05 hook API
- [ ] `src/types/config.ts` + `src/types/config.test.ts` — covers FE-05 D-29 roundtrip
- [ ] `src-tauri/src/bootstrap.rs` — covers SHELL-01 D-30
- [ ] `src-tauri/src/events.rs` — covers SHELL-01 D-31
- [ ] `e2e/specs/settings.spec.ts` extension — accordion sections + sync error
- [ ] `06-VERIFICATION.md` — QA-02 architecture checklist

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Local desktop app |
| V3 Session Management | no | Chat sessions local only |
| V4 Access Control | no | No new permission surface |
| V5 Input Validation | **yes (minor)** | Scheduled sync error string from Rust → UI is internal string, not user input; no XSS risk with React text rendering |
| V6 Cryptography | no | No crypto changes; keychain already handled in Phase 3 |

### Known Threat Patterns for Rust/Tauri settings stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Sync error string containing sensitive path info | Info Disclosure | Error strings are internal Rust error messages; don't log API keys in error paths |
| Accordion hidden testid DOM elements accessible to E2E | Tampering | WdIO selects by testid; collapsed accordion elements exist in DOM; use `toExist()` not `toBeDisplayed()` for hidden testids |
| React key prop leakage via config rendering | — | No risk; config values are user-controlled local data |

---

## Project Constraints (from .cursor/rules/)

- TDD required: failing tests first, then implementation; user-facing changes need E2E updates.
- E2E mandatory for user-facing features; deterministic mocks (`JARVIS_E2E=1`); no live LLM in CI.
- Surgical diffs; no scope beyond refactor; preserve all existing IPC signatures and `data-testid` contracts.
- Only `crates/store` opens SQLite; `set_meta`/`get_meta` are the correct persistence API.
- Quality gates: `cargo test --workspace`, `npm test`, `npm run test:e2e:local` after all changes.
- Stack unchanged: Rust stable, Tauri 2, React 19; no new framework dependencies.
- No barrel `src/views/index.ts` per Phase 1 D-20.
- Line count targets are **soft**: `App.tsx` <300, `lib.rs` <200; record actual count in VERIFICATION.

---

## Sources

### Primary (HIGH confidence)

- `src/App.tsx` — 2,034 lines; all settings state + handler inventory [VERIFIED: codebase]
- `src-tauri/src/lib.rs` — 315 lines; seed/emit/init_state/start_ask_e2e locations [VERIFIED: codebase]
- `src-tauri/src/state.rs` — AppState already defined here; init_state still in lib.rs [VERIFIED: codebase]
- `src-tauri/src/sync_scheduler.rs` — 182 lines; error swallowed at line 68; SyncStatusView fields [VERIFIED: codebase]
- `src/views/SettingsView.tsx` — 5-line stub passthrough [VERIFIED: codebase]
- `src/hooks/` — useChat, useLibrary, useMemory, useTasks (no useJarvisConfig) [VERIFIED: codebase]
- `src/types/ipc.ts` — flat AppConfig, SyncStatusView without error fields [VERIFIED: codebase]
- `e2e/specs/settings.spec.ts` — 3 existing tests [VERIFIED: codebase]
- `src-tauri/src/commands/` — agent.rs, chat.rs, config.rs, index.rs, lark.rs, library.rs, memory.rs, sync.rs [VERIFIED: codebase]
- `.planning/phases/06-settings-architecture-review/06-CONTEXT.md` — locked decisions D-01..D-44 [VERIFIED]
- `.planning/phases/06-settings-architecture-review/06-UI-SPEC.md` — accordion testids, component inventory [VERIFIED]
- `.planning/REQUIREMENTS.md` — FE-05, QA-02, QA-04 definitions [VERIFIED]
- `.planning/codebase/CONCERNS.md` — "Scheduled sync errors swallowed" concern [VERIFIED]
- `.planning/config.json` — `nyquist_validation: true` [VERIFIED]

### Secondary (MEDIUM confidence)

- `.planning/phases/05-agent-protocol/05-RESEARCH.md` — research format and pattern reference [VERIFIED]
- `.planning/phases/05-agent-protocol/05-04-SUMMARY.md` — Phase 5 completed state [VERIFIED]
- `src/hooks/useChat.ts` — controlled hook pattern template [VERIFIED: codebase]

### Tertiary (LOW confidence)

- None material — phase is pure codebase-driven refactor; no external library research needed.

---

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — no new deps; all patterns verified in existing codebase
- Architecture: **HIGH** — CONTEXT locked; all file locations verified with actual line counts
- Pitfalls: **HIGH** — regression risks confirmed from actual codebase state (sync error swallowed at exact line, accordion testid visibility pattern)

**Research date:** 2026-06-30
**Valid until:** 2026-07-30 (stable refactor domain; codebase unlikely to change before planning)
