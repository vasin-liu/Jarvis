# Roadmap: Jarvis

**Updated:** 2026-07-29

## Milestones

- ✅ **v1.9 Structural Refactor** — Phases 01–06 (shipped 2026-07-17) — [archive](./milestones/v1.9-ROADMAP.md)
- ✅ **v1.10 Wiki Compile Layer** — Phases 07–14 (shipped 2026-07-25) — [archive](./milestones/v1.10-ROADMAP.md)
- ✅ **v1.11 Related-docs + MCP** — Phases 15–19 (shipped 2026-07-29) — [archive](./milestones/v1.11-ROADMAP.md)
- 🚧 **v1.12 Release Hardening** — Phases 20–23 (in progress)

## Phases

<details>
<summary>✅ v1.9 Structural Refactor (Phases 01–06) — SHIPPED 2026-07-17</summary>

- [x] Phase 01–06 — 22 plans — completed 2026-07-17
- Details: [v1.9-ROADMAP.md](./milestones/v1.9-ROADMAP.md) · phases: [v1.9-phases/](./milestones/v1.9-phases/)

</details>

<details>
<summary>✅ v1.10 Wiki Compile Layer (Phases 07–14) — SHIPPED 2026-07-25</summary>

- [x] Phase 07: Wiki kind + config (3/3) — completed 2026-07-18
- [x] Phase 08: Deterministic Markdown renderer (2/2) — completed 2026-07-19
- [x] Phase 09: LLM wiki analysis (2/2) — completed 2026-07-19
- [x] Phase 10: Persist + index (2/2) — completed 2026-07-21
- [x] Phase 11: Library / Settings UI (2/2) — completed 2026-07-23
- [x] Phase 12: Obsidian zip export (2/2) — completed 2026-07-23
- [x] Phase 13: E2E + citation trust (2/2) — completed 2026-07-24
- [x] Phase 14: Tech debt closeout (3/3) — completed 2026-07-25

Details: [v1.10-ROADMAP.md](./milestones/v1.10-ROADMAP.md) · phases: [v1.10-phases/](./milestones/v1.10-phases/) · audit: [v1.10-MILESTONE-AUDIT.md](./milestones/v1.10-MILESTONE-AUDIT.md)

</details>

<details>
<summary>✅ v1.11 Related-docs + MCP (Phases 15–19) — SHIPPED 2026-07-29</summary>

- [x] Phase 15: Overlap scoring API (2/2) — completed 2026-07-25
- [x] Phase 16: Related-docs Library panel (4/4) — completed 2026-07-28
- [x] Phase 17: MCP transport + read-only scaffold (4/4) — completed 2026-07-28
- [x] Phase 18: MCP tools search + list_sources (4/4) — completed 2026-07-28
- [x] Phase 19: E2E + citation regression gate (3/3) — completed 2026-07-28

Details: [v1.11-ROADMAP.md](./milestones/v1.11-ROADMAP.md) · phases: [v1.11-phases/](./milestones/v1.11-phases/) · audit: [v1.11-MILESTONE-AUDIT.md](./milestones/v1.11-MILESTONE-AUDIT.md)

</details>

## v1.12 Release Hardening

**Goal:** Make FastEmbed deferred init observable and failure-visible, verify Windows release cold-start, then ship local **1.12.0** install package + changelog.

**Constraints:** Keep existing `DeferredEmbedder` + deferred `initial_scan`; no RAG/`RetrieverConfig` default changes; no GitHub Release; no auto-fallback embedder; TDD; reliability gate before version bump.

**Spec / plan:** `docs/superpowers/specs/2026-07-29-release-hardening-design.md` · `docs/superpowers/plans/2026-07-29-release-hardening.md`

### Phases

- [x] **Phase 20: DeferredEmbedder readiness API** - `ready_state` + testable timeout (BOOT-01, BOOT-04) (completed 2026-07-29)
- [x] **Phase 21: Readiness IPC** - AppState watch + `get_embedder_readiness` (BOOT-02) (completed 2026-07-29)
- [ ] **Phase 22: Settings readiness UI** - Pending/Ready/Failed + Vitest (BOOT-03)
- [ ] **Phase 23: Release gate + 1.12.0 package** - Smoke → bump → changelog → `tauri build` (SHIP-*, TRUST-*)

### Phase Details

### Phase 20: DeferredEmbedder readiness API

**Goal**: Callers can non-blockingly observe deferred FastEmbed init as Pending, Ready, or Failed, and failed/timed-out waits surface clear errors
**Depends on**: Nothing (v1.12 start)
**Requirements**: BOOT-01, BOOT-04
**Success Criteria** (what must be TRUE):

  1. `ready_state()` returns Pending before fulfill/fail, Ready after fulfill, Failed with message after fail
  2. `with_wait_timeout` allows short waits in tests; timeout while Pending returns a clear Init error
  3. Production `new()` still uses the existing ~300s wait default
  4. `cargo test -p embedder` green (including prior fulfill/fail tests)

**Plans**: TBD

### Phase 21: Readiness IPC

**Goal**: The Tauri shell exposes embedder readiness to the frontend without coupling UI to FastEmbed internals
**Depends on**: Phase 20
**Requirements**: BOOT-02
**Success Criteria** (what must be TRUE):

  1. FastEmbed cold-start path keeps an `Option<Arc<DeferredEmbedder>>` watch on `AppState`
  2. `get_embedder_readiness` returns `{ state, message }` for pending/ready/failed
  3. Non-deferred providers report Ready without a deferred watch
  4. Only `store` opens SQLite; `cargo check -p tauri-app` (+ readiness unit helpers) green

**Plans**: TBD

### Phase 22: Settings readiness UI

**Goal**: Users can see local embedder init status in Settings and get an actionable Failed hint
**Depends on**: Phase 21
**Requirements**: BOOT-03
**Success Criteria** (what must be TRUE):

  1. Settings shows Pending / Ready / Failed with stable `data-testid`s
  2. Failed state shows the error message and a hint to check model/cache or switch provider
  3. While Pending, UI refreshes readiness (poll) until Ready or Failed
  4. Vitest covers pending and failed rendering (`SettingsView.test.tsx`)

**Plans**: TBD
**UI hint**: yes

### Phase 23: Release gate + 1.12.0 package

**Goal**: Ship a trusted Windows **1.12.0** local package only after reliability smoke passes
**Depends on**: Phase 22
**Requirements**: SHIP-01, SHIP-02, SHIP-03, SHIP-04, TRUST-01, TRUST-02
**Success Criteria** (what must be TRUE):

  1. `docs/release/v1.12.0-smoke.md` exists and cold-start smoke is recorded PASS before bump
  2. `package.json` + `tauri.conf.json` are **1.12.0** only after smoke PASS
  3. `CHANGELOG.md` covers 1.9–1.11 summary + 1.12.0 hardening
  4. `tauri build` produces a Windows artifact that launches; Mock-path tests stay green; no CI FastEmbed download required

**Plans**: TBD

## Progress

| Phase | Milestone | Plans | Status | Completed |
|-------|-----------|-------|--------|-----------|
| 01–06 | v1.9 | 22/22 | Complete | 2026-07-17 |
| 07–14 | v1.10 | 18/18 | Complete | 2026-07-25 |
| 15–19 | v1.11 | 17/17 | Complete | 2026-07-28 |
| 20. DeferredEmbedder readiness API | v1.12 | 2/2 | Complete    | 2026-07-29 |
| 21. Readiness IPC | v1.12 | 2/2 | Complete    | 2026-07-29 |
| 22. Settings readiness UI | v1.12 | 0/? | Not started | - |
| 23. Release gate + 1.12.0 package | v1.12 | 0/? | Not started | - |

## Backlog (post-v1.12 / carryover)

- Bulk compile + `auto_on_insights` UX (WIKI-F01)
- Cross-corpus entity merge
- Dual `index.md` writers consolidation
- Unify Settings `reload_providers` with deferred cold-start (v1.12 debt)
- REL-F01 / REL-F02
- MCP-F01 / MCP-F02 / MCP-F03
- REL-02 affinity score threshold (D-06 deferred)
- GitHub Release automation; Linux/macOS packages

---

*Next:* `/gsd-plan-phase 20`
