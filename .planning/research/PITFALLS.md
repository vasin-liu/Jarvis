# Pitfalls — Brownfield Incremental Refactor (Jarvis v1.9.x)

> Research dimension: **Pitfalls**  
> Date: 2026-06-17  
> Sources: `.planning/PROJECT.md`, `.planning/codebase/CONCERNS.md`, `.planning/codebase/TESTING.md`, codebase map

This document catalogs critical mistakes common in incremental refactors of brownfield Tauri/Rust/React apps. Each pitfall is scoped to Jarvis v1.9.x structural work: splitting `App.tsx` / `lib.rs`, config/keychain hardening, SQLite evolution, IPC stability, and E2E regression safety.

**Phase key** (from `PROJECT.md` balanced phasing):

| Phase | Focus |
|-------|--------|
| **P1 — Frontend structure** | Extract views/hooks from `src/App.tsx` |
| **P2 — Tauri shell** | Split `src-tauri/src/lib.rs` into `commands/*` |
| **P3 — Config & secrets** | Nested `AppConfig`, OS keychain for API keys |
| **P4 — Agent protocol** | Replace XML+JSON tool parsing with structured calls |
| **P5 — Memory model** | Dedicated storage or strict `memory://` URIs |
| **All phases** | E2E gate, architecture review, shippable increments |

---

## 1. Big-bang extraction instead of vertical slices

**What goes wrong:** Moving all of Chat, Library, Settings, etc. out of `App.tsx` in one PR—or splitting all 40 Tauri commands at once—creates a massive diff that is impossible to review, bisect, or roll back. Merge conflicts freeze the team for days.

**Warning signs:**
- PR touches 2,000+ lines with no behavior change claim verified by tests
- Multiple views/commands move before any single path is E2E-verified
- "Cleanup" commits mixed with structural moves in the same branch

**Prevention:**
- One **vertical slice** per merge: e.g. extract `SettingsView` + its hooks + keep shared state in a thin parent; or move `commands/config.rs` only with zero signature changes
- Each slice must pass `cargo test --workspace`, `npm test`, and `npm run test:e2e:local`
- Use `git mv` + re-export shims so imports change minimally per slice

**Address in:** P1, P2 — planning gate before first extraction PR

---

## 2. Splitting UI without preserving `data-testid` contracts

**What goes wrong:** E2E specs (`e2e/specs/*.spec.ts`) select elements via stable `data-testid` attributes defined in `App.tsx`. Renaming, nesting, or dropping testids during component extraction breaks CI on Windows without any functional regression in the product.

**Warning signs:**
- E2E fails on `nav-*`, `chat-input`, `e2e-active`, `ask-handle-count` after a "pure refactor"
- Testids moved to child components that mount conditionally (race in `wdio.conf.ts` readiness waits)
- Duplicate testids after copy-paste into multiple views

**Prevention:**
- Treat `data-testid` values as **public API** — document in PR checklist; never rename without updating specs
- Move testids with the JSX they identify; keep `app-root`, `e2e-active`, `chat-session-ready` at stable layout boundaries
- Run `npm run test:e2e:local` before every frontend merge (policy in `.cursor/rules/e2e-required.mdc`)

**Address in:** P1 — every frontend extraction PR; **All phases** for any UI touch

---

## 3. Lifting state incorrectly when extracting views

**What goes wrong:** `App.tsx` holds intertwined state: `config`, chat sessions, index progress, agent orchestration, Lark sync status. Naive extraction either duplicates state (two sources of truth) or breaks prop-drilling/event chains so IPC fires twice or not at all.

**Warning signs:**
- Double `invoke("set_config")` on mount
- Child view has its own `useState` copy of `config` that diverges from parent
- `useEffect` dependency arrays change and re-trigger sync/index on navigation only
- Chat ask handle count (`ask-handle-count`) increments unexpectedly

**Prevention:**
- Extract **presentational** views first; keep IPC and config writes in one hook (`useJarvisConfig`) or parent container
- One writer per piece of persisted state; children receive callbacks, not direct `invoke`
- Add Vitest tests for extracted pure helpers before moving JSX (repo currently has almost no component tests—helpers in `src/lib/` are the pattern)

**Address in:** P1 — design hook boundaries before first view extraction

---

## 4. Circular dependencies after `lib.rs` module split

**What goes wrong:** Splitting `lib.rs` into `commands/chat.rs`, `commands/index.rs`, etc. introduces `use crate::commands::foo` cycles via shared `AppState`, or commands calling each other through re-exports. Rust compile errors or subtle initialization order bugs follow.

**Warning signs:**
- `mod commands` tree needs `pub use` gymnastics to break cycles
- `AppState` grows new `Arc<Mutex<...>>` fields to work around access
- `index_ops.rs` and new command modules duplicate orchestration (already a concern vs `sync_scheduler.rs`)

**Prevention:**
- Mirror **existing crate boundaries** (`store`, `rag`, `agent`, `lark`) — commands should delegate to crates, not to each other
- Keep `AppState` definition in one file (`state.rs` or `lib.rs`); commands take `State<'_, AppState>`
- `lib.rs` remains registration-only: `tauri::generate_handler![...]` + `run()`
- No command-to-command calls; shared logic stays in `index_ops.rs` or moves to a named service module

**Address in:** P2 — module layout design before first command file move

---

## 5. Changing Tauri command signatures during a "structural" refactor

**What goes wrong:** Renaming `forget_memory_cmd` → `forget_memory`, changing serde field names (`{ id }` → `{ memory_id }`), or switching return types from `Result<T, String>` to typed errors breaks the React frontend and any external scripts. TypeScript has no compile-time check against Tauri IPC.

**Warning signs:**
- Frontend `invoke` calls updated in same PR as Rust split
- Silent failures: command returns `Err` string that UI does not display
- `grep invoke` shows mismatched argument shapes between `src/` and `src-tauri/`

**Prevention:**
- **Freeze IPC contracts** during P1–P2; structural-only changes
- If a signature must change: add new command, deprecate old, update frontend in same release, document in changelog
- Consider thin typed wrappers in `src/lib/tauri.ts` (per CONCERNS.md recommendation) so renames happen once

**Address in:** P2 (freeze); P3–P5 if new payloads required — always paired frontend + E2E update

---

## 6. SQLite migration without version bump or idempotency test

**What goes wrong:** Jarvis uses inline migrations in `crates/store/src/schema.rs` (`meta.schema_version`, `migrate()` chain). Adding columns/tables without incrementing version, or writing non-idempotent `ALTER` logic, corrupts existing user `kb.sqlite` files on upgrade.

**Warning signs:**
- Migration code only tested on fresh `open_in_memory` DBs
- No `migrate_vN_to_vM_*` unit test mirroring existing `migrate_v1_to_v2` pattern
- `init_schema` and `migrate` diverge (new installs vs upgrades get different shapes)
- Memory refactor (P5) writes to new tables but old code path still reads `sources` only

**Prevention:**
- Every schema change: bump `schema_version`, add dedicated `#[test] fn migrate_vX_to_vY_*` with pre-migration fixture SQL
- Run migration tests in `cargo test -p store`
- **Only `crates/store` opens SQLite** — never add migrations in `src-tauri`
- For destructive changes (e.g. vec dim): explicit user action + backup UX, not silent migrate

**Address in:** P5 primarily; any phase that touches `store` schema

---

## 7. Breaking sqlite-vec / FTS5 / chunks triple-write consistency

**What goes wrong:** `insert_chunks` and `delete_chunks_for_source` must keep `chunks`, `vec_chunks`, and `chunks_fts` in sync (`CONCERNS.md`). Refactors that shortcut deletes or add new write paths create orphan FTS rows or vectors—search returns ghosts or misses hits.

**Warning signs:**
- RAG answers cite deleted sources
- `rebuild_index` fixes corruption (symptom of incremental inconsistency)
- Tests pass on in-memory DB but not after full index → delete → re-index cycle

**Prevention:**
- Never bypass `Store` methods for chunk writes
- Integration test: index → delete source → assert all three subsystems empty
- Treat `reinit_vectors` as last resort (drops all embeddings)—not a migration tool

**Address in:** P5 (memory table); **All phases** that touch indexing paths

---

## 8. Embedding dimension change treated as a migration

**What goes wrong:** `vec_chunks` dimension is baked into the virtual table DDL. Changing embedder model/dim requires `Store::reinit_vectors`—full re-index. Teams try to "migrate" vectors in place; sqlite-vec makes this painful and error-prone.

**Warning signs:**
- `DimMismatch` at runtime after config refactor
- `AppConfig::embedding_dim()` fallback masks unknown model names (`CONCERNS.md` bug)
- Settings UI allows save without triggering rebuild

**Prevention:**
- Validate dim at **provider build** time, not first embed call
- Config refactor (P3): keep dim fields coherent; document that dim change = user-initiated rebuild
- Do not conflate P3 config nesting with vec schema migration

**Address in:** P3 — config validation; not a silent migration

---

## 9. E2E mock path diverging from production path

**What goes wrong:** `JARVIS_E2E=1` branches in `src-tauri/src/e2e.rs` and scattered checks in `lib.rs` / Lark sync short-circuit real providers. Refactoring command modules can wire E2E mocks to a different code path than production—CI green, production broken.

**Warning signs:**
- `#[cfg(test)]` or `is_e2e_mode()` in the middle of business logic instead of at provider injection boundary
- Lark sync uses `lark_fixture_from_input` in E2E but production path untested
- New command forgets to call `apply_e2e_config` wiring

**Prevention:**
- Inject mocks at **provider factory** (`crates/config/src/providers.rs`), not inside command handlers
- After P2 split: single bootstrap in `lib::run()` sets `AppState` providers; E2E env only affects that layer
- Extend `src-tauri/src/e2e.rs` tests when adding fixtures

**Address in:** P2 — command extraction; **All phases**

---

## 10. Skipping E2E because "refactor only"

**What goes wrong:** Developers assume structural changes need no test updates. Jarvis policy (`.cursor/rules/e2e-required.mdc`) requires E2E for user-facing behavior; refactors that move Settings rebuild button or memory forget flow break journeys in `full-ui.spec.ts` without unit test signal.

**Warning signs:**
- `cargo test` green, `npm test` green, E2E not run locally
- PR template missing e2e checkbox
- Known gaps (tasks CRUD, insights, hooks/plugins per TESTING.md) mask regressions in untested areas

**Prevention:**
- **Hard gate:** `npm run test:e2e:local` before merge; CI Windows job is authoritative
- When extracting UI: run affected spec (`settings.spec.ts`, `memory.spec.ts`, etc.) not only smoke
- Fill critical E2E gaps when touching those areas (tasks, insights, scheduled sync)

**Address in:** **All phases** — non-negotiable per PROJECT.md success criteria

---

## 11. Keychain migration leaving plaintext behind or failing closed

**What goes wrong:** Moving `cloud_api_key` from `config.json` to OS keychain (Windows Credential Manager, macOS Keychain, Secret Service) commonly fails by: (a) leaving the key in JSON after "migration", (b) losing the key on first read error, (c) breaking headless/CI with no keyring, (d) different service/account names per platform causing re-prompt loops.

**Warning signs:**
- `config.json` still contains `cloud_api_key` after upgrade
- Settings shows empty API key but cloud chat works (or vice versa)
- E2E mode breaks because keyring unavailable in CI
- User backup/restores config file and expects key to transfer

**Prevention:**
- **One-time migration** on startup: if JSON has key and keyring empty → write keyring → strip key from JSON → save
- Never log or serialize keyring values; redact in errors
- `JARVIS_E2E=1`: bypass keyring with mock providers (already forces Mock chat/embedder)
- Document backup: "export key separately" or optional encrypted export
- Use consistent keyring entry identity (`service = "jarvis"`, `user = "cloud_api_key"` or per-provider)
- Test roundtrip: tempfile config with key → migrate → assert JSON clean + `get_password` works

**Address in:** P3 — dedicated migration PR with integration test; not bundled with unrelated AppConfig nesting

---

## 12. Nested `AppConfig` serde breaking existing installs

**What goes wrong:** Splitting flat `AppConfig` (~30+ fields) into nested structs (`EmbeddingConfig`, `SyncConfig`, `AgentConfig`) with serde flatten/rename breaks deserialization of existing `config.json` files if field names or defaults change.

**Warning signs:**
- `serde` unknown field errors on startup
- `null` vs missing field treated differently after refactor
- Default changes (`default_granted_plugin_permissions` returning `shell_exec`) alter behavior on load

**Prevention:**
- Use `#[serde(default)]`, `#[serde(alias = "old_name")]`, and flatten carefully
- Roundtrip test: load golden v1.8 `config.json` fixture → save → reload → assert equivalence
- Separate **shape refactor** from **security defaults** change (e.g. shell_exec default-deny is its own decision)

**Address in:** P3 — before keychain work or in same phase with compat tests

---

## 13. Agent protocol swap breaking Mock chat and E2E

**What goes wrong:** Replacing `<tool_call>{json}</tool_call>` parsing (`crates/agent/src/tools.rs`) with structured tool calls changes `MockChatModel` response format, agent loop in `run.rs`, and E2E agent specs. Partial migration leaves models emitting old format while parser expects new.

**Warning signs:**
- Agent mode E2E passes but real OpenAI path fails (or reverse)
- `MAX_TOOL_ROUNDS` hardcoded at 3 masks infinite retry loops
- Tool call UI (`data-testid="agent-tool-calls"`) never populates

**Prevention:**
- Update `MockChatModel` and `crates/agent` tests in same PR as parser change
- Feature-flag or version agent protocol during transition if needed
- E2E `agent.spec.ts` + extend coverage for tool-call UI (currently weak per TESTING.md)

**Address in:** P4 — isolated phase; do not mix with P1/P2 splits

---

## 14. Memory model migration causing title collisions and forget wrong row

**What goes wrong:** Today memory uses `SourceKind::Memory` with fuzzy title matching in `resolve_memory_id` (`crates/memory/src/learn.rs`). Introducing `memory://{uuid}` URIs or a `memories` table without migrating existing rows breaks forget/update or duplicates memories.

**Warning signs:**
- Library list shows duplicate memory entries
- `forget_memory_cmd` deletes wrong source
- E2E `memory.spec.ts` passes on fresh DB but fails on migrated user DB

**Prevention:**
- Migration script: assign stable URIs to existing memory sources; backfill table
- Deprecate title-only resolution with warning period
- Integration test with colliding titles before/after migrate

**Address in:** P5 — schema + memory crate together; E2E memory spec on migrated fixture DB

---

## 15. Opening SQLite outside `crates/store`

**What goes wrong:** Refactor convenience leads to `rusqlite::Connection` in `src-tauri` for "quick reads." Violates single-DB-owner rule; causes lock contention, missed migrations, and FTS/vec corruption.

**Warning signs:**
- `rusqlite` in `src-tauri/Cargo.toml` for non-test code
- `SQLITE_BUSY` under scheduler + UI (already a risk without `busy_timeout`)

**Prevention:**
- All persistence through `Store` API; new queries = new `Store` methods in P5 if needed
- Consider WAL + `busy_timeout` as separate reliability task (CONCERNS.md)

**Address in:** **All phases** — architecture review checkpoint

---

## 16. CI split hiding platform regressions

**What goes wrong:** `cargo test` runs on Ubuntu; E2E on Windows only. Path logic (`ingest` cwd heuristics, `lark` `.cmd` fallback), keyring, and WebDriver behavior differ. Refactor passes Rust CI but fails packaged Windows app.

**Warning signs:**
- Path-dependent bugs "only in production"
- `msedgedriver` / Edge version skew breaks E2E intermittently
- Keychain works on dev machine, fails in CI (expected—E2E must mock)

**Prevention:**
- Run Windows E2E before claiming phase complete
- Pin Edge/driver alignment in `e2e/README.md`; keep `msedgedriver.exe` gitignored
- Path tests with explicit `tempfile` on Windows-style paths where feasible

**Address in:** **All phases** — release verification

---

## 17. Architecture review skipped when "files just moved"

**What goes wrong:** MODULE boundaries drift from crate layout (`agent` logic in `commands/agent.rs` that re-implements `crates/agent`). Duplicate orchestration between `index_ops.rs` and `sync_scheduler.rs` worsens.

**Warning signs:**
- Business logic added to command handlers instead of library crates
- `index_ops.rs` not reduced after command extraction
- Frontend `invoke` scattered across views instead of `src/lib/tauri.ts`

**Prevention:**
- End-of-phase checklist: boundaries match `.planning/codebase/ARCHITECTURE.md` and `STRUCTURE.md`
- Commands = thin IPC adapters; crates = domain logic
- Log decision in PROJECT.md Key Decisions table

**Address in:** End of P1, P2, P3, P4, P5

---

## 18. Mixing small features with refactor slices

**What goes wrong:** PROJECT.md allows small features in touched files—but scope creep turns refactor PRs into feature + refactor, invalidating E2E signal and delaying rollback.

**Warning signs:**
- PR title says "extract LibraryView" but includes new Library bulk actions
- Requirements creep into Active without phase plan update

**Prevention:**
- One intent per PR: refactor **or** feature, not both unless feature is trivial (typo, testid)
- Update phase plan when deliberate feature work piggybacks

**Address in:** **All phases** — planning discipline

---

## Summary matrix

| Pitfall | Highest-risk phase |
|---------|-------------------|
| Big-bang extraction | P1, P2 |
| `data-testid` / E2E breakage | P1, All |
| State duplication on UI split | P1 |
| `lib.rs` circular deps | P2 |
| IPC signature drift | P2, P3–P5 |
| SQLite migration errors | P5 |
| FTS/vec/chunks inconsistency | P5, All |
| Vec dim "migration" | P3 |
| E2E mock divergence | P2, All |
| Skipping E2E | All |
| Keychain plaintext leftover | P3 |
| `AppConfig` serde break | P3 |
| Agent protocol partial swap | P4 |
| Memory title collision migrate | P5 |
| SQLite outside store | All |
| Windows/CI platform skew | All |
| Skipped architecture review | End of each phase |
| Refactor + feature scope creep | All |

---

## Recommended phase gates (checklist)

Before merging any v1.9.x phase:

1. [ ] Vertical slice only — reviewable diff, single primary intent
2. [ ] `cargo test --workspace` green
3. [ ] `npm test` green (if `src/` touched)
4. [ ] `npm run test:e2e:local` green (if any user-facing path touched)
5. [ ] IPC contract unchanged OR frontend + specs updated in same PR
6. [ ] No new `rusqlite` usage outside `crates/store`
7. [ ] Schema changes have `migrate_vX_to_vY` test
8. [ ] Secrets not in `config.json` after P3 migration path
9. [ ] Architecture review notes filed for phase
10. [ ] CONCERNS.md items touched are updated or ticketed

---

*Generated for GSD research — Pitfalls dimension. Refresh after first phase ships or when PLAN.md defines explicit phase IDs.*
