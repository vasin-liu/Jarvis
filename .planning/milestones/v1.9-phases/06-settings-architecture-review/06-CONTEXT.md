# Phase 6: Settings + Architecture Review - Context

**Gathered:** 2026-06-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Complete the v1.9.x structural refactor: **extract Settings** (`SettingsView` + `useJarvisConfig`), **surface scheduled sync failures**, **slim App.tsx / lib.rs**, and **architecture sign-off** — zero regression on user journeys; **QA-04 strict refactor-only** (no new product capabilities beyond locked UX improvements in this discussion).

**In scope:**
- `AppShell.tsx` — sidebar nav + global `err` banner + `busy` overlay; `App.tsx` = view router + cross-view coordination only
- `useAppEvents.ts` — global `index-progress` / `jarvis-ask-done` listeners; props into views
- Migrate residual Chat/agent handlers from `App.tsx` into `useChat` / `ChatView`
- `SettingsView.tsx` full extraction; `useJarvisConfig` owns config + all settings-local state
- Accordion Settings layout; nested TS `AppConfig` in `src/types/config.ts` mirroring Rust
- `sync_scheduler`: persist `last_scheduled_sync_error` + `last_scheduled_sync_error_at` in `store.meta`; extend `SyncStatusView`
- Lark manual URL inputs collapsed under **「高级 · 手动同步」** (testids preserved)
- Shell: `bootstrap.rs`, `events.rs`, `init_state` → `state.rs`, E2E cmds → `commands/chat.rs`
- E2E: extend `settings.spec.ts` + `full-ui.spec.ts`; introduce `e2e/playwright/` playwright-cli nav smoke (supplemental, **CI gate = WebdriverIO**)
- `06-VERIFICATION.md` — Phase 5-style checklist; mark **FE-01** + **SHELL-01** Complete; Known Remaining for non-structural CONCERNS items
- Target release **v1.9.6** on phase pass

**Out of scope (later / backlog):**
- Replacing WebdriverIO with Playwright in CI
- react-router or new frontend routing dependency
- Removing Lark manual sync entirely (collapsed, not deleted)
- CONCERNS.md non-structural items (IM pagination, PDF ingest, etc.) — document only
- v2.0.0 bump (ARCH-01)
- New user-facing features beyond accordion/sync-error/Lark-advanced UX locked here

</domain>

<decisions>
## Implementation Decisions

### App.tsx & AppShell (FE-01 completion)
- **D-01:** Create **`AppShell.tsx`**: sidebar navigation + global **err banner** + **busy** overlay.
- **D-02:** **`App.tsx` retains view switch only** (`chat` | `library` | `tasks` | `memory` | `settings`) + minimal cross-view coordination — not layout chrome.
- **D-03:** **Line counts are soft targets** (`App.tsx` <300, `lib.rs` <200): record in VERIFICATION with notes if missed; **not a hard gate**.
- **D-04:** Global event listeners → **`useAppEvents.ts`**; App subscribes and passes props (not per-view duplication).
- **D-05:** **Migrate Chat/agent handlers** (`handleAgentAsk`, tool-call state, etc.) from `App.tsx` into **`useChat` / `ChatView`** this phase.
- **D-06:** **No barrel** `src/views/index.ts` — direct imports per Phase 1 D-20.
- **D-07:** **Busy state**: `AppShell` receives `busy` + `setBusy` via React context or props drilled from `App.tsx` (App still coordinates cross-view busy).

### Settings Extraction (FE-05)
- **D-08:** **`useJarvisConfig` hook-owned state**: config + Lark tokens + api key draft + skills/hooks/plugins + agent CRUD drafts + sync status; App passes **`busy`/`err` only**.
- **D-09:** Settings layout = **accordion sections**; default **expanded: Providers + Index status**; rest collapsed.
- **D-10:** Section order: **Providers → Index → Sync (watch folders + scheduled sync) → Lark → Agent basics → Advanced (Agent CRUD + skills/hooks/plugins)**.
- **D-11:** **Index rebuild / reinit buttons stay in Settings** (`rebuild-index`, `reinit-rebuild-index` testids unchanged).
- **D-12:** **Watch folders** UI lives inside **「同步与监视」** accordion section.
- **D-13:** Agent: **basic agent selection** in main Settings; **CRUD + skills/hooks/plugins** under **「高级」** accordion.
- **D-14:** Config save = **immediate per-field** `set_config` (current behavior); no section/global Save button.
- **D-15:** Add **`settings-section-*`** testids for accordion; **preserve all existing** settings testids (`settings-panel`, `cloud-api-key-input`, etc.).

### Scheduled Sync Errors (QA-02 / S8)
- **D-16:** Persist **`last_scheduled_sync_error`** (string) + **`last_scheduled_sync_error_at`** (unix timestamp) in `store.meta` on scheduler failure.
- **D-17:** **Partial failure counts**: if `failed > 0` in scheduled run (even when `indexed > 0`), write error summary to meta.
- **D-18:** **Clear meta on next successful** scheduled sync.
- **D-19:** Display error **inline in 「定时同步」accordion** (not global Settings banner).
- **D-20:** **Keep「立即同步」** button; failures show in same section.
- **D-21:** Extend **`SyncStatusView`** with `lastScheduledSyncError` + `lastScheduledSyncErrorAt` (camelCase serde); frontend reads via existing `get_sync_status`.

### Lark Manual Sync (Phase 3 D-16 follow-through)
- **D-22:** Manual doc/sheet/mail/IM URL inputs → **collapsed「高级 · 手动同步」** with hint: *推荐使用定时同步 / lark-cli*.
- **D-23:** **Primary Lark UX**: detect connection + **立即同步** + scheduled sync toggles.
- **D-24:** **Preserve** `lark-sync-url-input` etc. testids (DOM may be hidden until accordion expanded).
- **D-25:** **Error separation**: manual Lark errors (`larkSyncError`) in Lark section; scheduled errors in sync section.
- **D-26:** **lark-cli path + identity** config stays in main Lark section (not advanced).

### TypeScript Config (deferred Phase 3)
- **D-27:** **`src/types/config.ts`**: nested `EmbeddingConfig` / `SyncConfig` / `AgentConfig` mirroring Rust; flat IPC compat via serde on Rust side.
- **D-28:** `useJarvisConfig` uses **nested config shape** internally; saves whole config via `set_config`.
- **D-29:** **Vitest roundtrip** test: nested TS types ↔ camelCase IPC payload consistency.

### Tauri Shell (SHELL-01 completion)
- **D-30:** **`bootstrap.rs`**: `seed_skills_dir`, `seed_hooks_dir`, `seed_plugins_dir`.
- **D-31:** **`events.rs`**: `emit_index_progress`, `emit_index_complete`.
- **D-32:** **`init_state()` → `state.rs`**; `lib.rs` = `run()` + `generate_handler!` registration.
- **D-33:** **`start_ask_e2e` + `is_e2e_mode_cmd` → `commands/chat.rs`**.
- **D-34:** Startup order: **Store → config+migration → keychain → memory URI migration → providers → watcher → scheduler**.

### Global Error UX
- **D-35:** Global **`err` banner in AppShell** (top); Chat may retain `chat-error` testid for E2E if needed for agent/RAG errors.

### Architecture Review (QA-02, QA-04)
- **D-36:** Checklist source: **CONCERNS.md + ROADMAP Phase 6 success criteria + REQUIREMENTS** traceability.
- **D-37:** **`06-VERIFICATION.md`**: Phase 5 table format + automated check boxes; mark **FE-01** and **SHELL-01** **Complete**.
- **D-38:** Structural CONCERNS **PASS**; non-structural items → **Known Remaining** table in VERIFICATION (do not block v1.9.6).
- **D-39:** **QA-04 strict**: only refactor, bugfix, tests, docs — accordion/sync-error/Lark-collapse are **in-scope UX**, not new features.
- **D-40:** Phase pass → **v1.9.6** tag target.

### E2E & Playwright
- **D-41:** **CI gate unchanged**: WebdriverIO + `tauri-driver`; extend **`settings.spec.ts`** (accordion, sync error panel, scheduled sync) + **`full-ui.spec.ts`** Settings journey.
- **D-42:** Introduce **`e2e/playwright/`** with playwright-cli **full-nav smoke + Settings focus**; **not required in CI** this phase.
- **D-43:** Playwright lands **in Phase 6** as supplemental tooling; **WDIO remains authoritative** for merge gate.

### i18n / Copy
- **D-44:** Settings copy: **concise 中文 technical tone** (consistent with existing UI).

### Claude's Discretion
- Exact accordion section IDs and `settings-section-*` testid suffixes.
- React context vs prop-drill for busy/err through AppShell.
- `SyncStatusView` error string formatting for partial failures.
- `e2e/playwright/` script layout and playwright-cli command wrappers.
- Whether `chat-error` duplicates AppShell err or only agent-specific errors.
- Minor hook internal organization within `useJarvisConfig`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope & requirements
- `.planning/ROADMAP.md` — Phase 6 goal, success criteria, key files
- `.planning/REQUIREMENTS.md` — FE-05, QA-02, QA-04; FE-01/SHELL-01 completion
- `.planning/PROJECT.md` — Core value, refactor constraints, success criteria
- `.planning/phases/03-library-tasks-keychain/03-CONTEXT.md` — Controlled views, keychain UX, Lark vision (D-14..D-19)
- `.planning/phases/05-agent-protocol/05-CONTEXT.md` — `start_ask_e2e` retention policy (superseded: move to chat.rs)
- `.planning/phases/01-scaffold-shell-foundation/01-CONTEXT.md` — No barrel exports, stub patterns

### Research & architecture
- `.planning/research/ARCHITECTURE.md` — S8 Settings + sync surfacing slice
- `.planning/research/SUMMARY.md` — Phase 6 ordering rationale
- `.planning/research/PITFALLS.md` — Per-phase risk checklist
- `.planning/codebase/CONCERNS.md` — Debt inventory + Known Remaining baseline
- `.planning/codebase/STRUCTURE.md` — Target views/hooks/commands layout
- `.planning/codebase/TESTING.md` — E2E spec map (`settings.spec.ts`)

### Engineering policy
- `AGENTS.md` — TDD, E2E mandatory, file boundaries
- `.cursor/rules/e2e-required.mdc` — E2E for user-facing changes
- `.cursor/rules/tdd-goal-driven.mdc` — Red-green-refactor
- `.cursor/rules/frontend-taste.mdc` — Settings UI (liquid glass, motion, a11y)

### Implementation targets (current state)
- `src/App.tsx` — ~1961 lines; Settings JSX inline ~1051–2026; Chat handlers still present
- `src/views/SettingsView.tsx` — Stub passthrough
- `src/hooks/` — `useChat`, `useLibrary`, `useTasks`, `useMemory` (no `useJarvisConfig` yet)
- `src/types/ipc.ts` — Flat `AppConfig` today
- `src-tauri/src/lib.rs` — ~292 lines; seeds, E2E cmd, emit helpers, setup
- `src-tauri/src/sync_scheduler.rs` — Swallows errors; `SyncStatusView` lacks error fields
- `src-tauri/src/commands/sync.rs` — `get_sync_status`, `run_scheduled_sync_cmd`
- `crates/store/src/store.rs` — `get_meta` / `set_meta`
- `e2e/specs/settings.spec.ts` — Index + API key tests to extend

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/views/ChatView.tsx` + `useChat.ts` — Target for migrated App chat handlers; controlled pattern reference.
- `src/views/LibraryView.tsx` / `TasksView.tsx` / `MemoryView.tsx` — Completed controlled extractions.
- `src/lib/tauri.ts` — Extend with settings/sync helpers; config IPC already partial.
- `src-tauri/src/commands/config.rs` — Config + API key commands; pattern for thin IPC.
- `src-tauri/src/state.rs` — `AppState`; destination for `init_state`.
- Phase 3 keychain: `clear_api_key`, masked placeholder, `cloud-api-key-input` testids.

### Established Patterns
- Controlled views: App/hook coordinates `busy`/`err`; views are presentational.
- Tauri `Result<T, String>` at IPC boundary; meta keys for operational state.
- E2E: `JARVIS_E2E=1` deterministic mocks; stable `data-testid` kebab-case.
- One-release deprecation pattern (Phase 4/5): log + collapse, not hard removal.

### Integration Points
- `sync_scheduler` background thread → must write meta on error; `get_sync_status` for Settings poll on mount/focus.
- `App.tsx` view router passes hooks' refresh fns into views after extraction.
- `generate_handler!` in `lib.rs` — register any moved commands without renames.
- `settings.spec.ts` + `full-ui.spec.ts` — primary regression gate; playwright-cli supplemental.

</code_context>

<specifics>
## Specific Ideas

- User communicates in **中文** for discuss; artifacts in English for downstream agents.
- **Soft line-count targets** — prioritize structural completion over arbitrary thresholds.
- **Playwright in Phase 6** as dev/QA supplement (`e2e/playwright/`), not CI replacement.
- Lark manual sync: **fold not delete** — honor Phase 3 vision while preserving E2E testids.
- Architecture sign-off = **structural debt closed**, remaining CONCERNS documented honestly.

</specifics>

<deferred>
## Deferred Ideas

- **Playwright as CI gate** — WDIO remains merge authority; revisit post-v1.9.6.
- **react-router** for view navigation — rejected this phase.
- **Complete removal of Lark manual URL inputs** — folded to advanced; removal later if unused.
- **Section-level Save button for Settings** — user chose immediate save.
- **Global Settings banner for sync errors** — user chose inline sync section only.
- **CONCERNS.md full closure** — IM pagination, PDF ingest, FastEmbed CI ignore, etc. → Known Remaining.
- **v2.0.0 / ARCH-01** — after refactor review complete.

</deferred>

---

*Phase: 6-Settings + Architecture Review*
*Context gathered: 2026-06-30*
